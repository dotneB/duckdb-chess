extern crate duckdb;
extern crate duckdb_loadable_macros;
extern crate libduckdb_sys;

use duckdb::{
    core::{DataChunkHandle, Inserter, LogicalTypeHandle, LogicalTypeId},
    vtab::{BindInfo, InitInfo, TableFunctionInfo, VTab},
    Connection, Result,
};
use duckdb_loadable_macros::duckdb_entrypoint_c_api;
use libduckdb_sys as ffi;
use pgn_reader::{Visitor, Skip, SanPlus, Reader, RawTag, RawComment};
use std::{
    error::Error,
    ffi::CString,
    fs::File,
    io::{BufRead, BufReader},
    ops::ControlFlow,
    path::PathBuf,
    sync::atomic::{AtomicBool, Ordering},
    sync::Mutex,
};

/// Stores parsed game data from PGN - matches Lichess dataset schema
#[derive(Debug, Clone, Default)]
pub struct GameRecord {
    // Core game info
    pub event: Option<String>,
    pub site: Option<String>,
    pub white: Option<String>,
    pub black: Option<String>,
    pub result: Option<String>,

    // Player info
    pub white_title: Option<String>,
    pub black_title: Option<String>,
    pub white_elo: Option<i32>,
    pub black_elo: Option<i32>,

    // Date/Time
    pub utc_date: Option<String>,
    pub utc_time: Option<String>,

    // Opening info
    pub eco: Option<String>,
    pub opening: Option<String>,

    // Game details
    pub termination: Option<String>,
    pub time_control: Option<String>,

    // Movetext
    pub movetext: String,

    // Parse diagnostics
    /// Spec: data-schema - Parse Error Column
    /// Contains NULL for successfully parsed games or error message for failed games
    pub parse_error: Option<String>,
}

/// Visitor implementation for pgn-reader
/// Spec: pgn-parsing - Visitor Pattern Implementation
/// Implements streaming PGN parsing using the pgn-reader library's Visitor trait
struct GameVisitor {
    headers: Vec<(String, String)>,
    movetext_buffer: String,
    move_count: u32,
    current_game: Option<GameRecord>,
}

impl GameVisitor {
    fn new() -> Self {
        Self {
            headers: Vec::new(),
            movetext_buffer: String::new(),
            move_count: 0,
            current_game: None,
        }
    }

    fn get_header(&self, key: &str) -> Option<String> {
        self.headers
            .iter()
            .find(|(k, _)| k == key)
            .map(|(_, v)| v.clone())
    }

    fn finalize_game(&mut self) {
        // Helper to parse integer values
        let parse_elo = |s: &str| s.parse::<i32>().ok();

        self.current_game = Some(GameRecord {
            event: self.get_header("Event"),
            site: self.get_header("Site"),
            white: self.get_header("White"),
            black: self.get_header("Black"),
            result: self.get_header("Result"),
            white_title: self.get_header("WhiteTitle"),
            black_title: self.get_header("BlackTitle"),
            white_elo: self.get_header("WhiteElo").and_then(|s| parse_elo(&s)),
            black_elo: self.get_header("BlackElo").and_then(|s| parse_elo(&s)),
            utc_date: self.get_header("UTCDate").or_else(|| self.get_header("Date")),
            utc_time: self.get_header("UTCTime").or_else(|| self.get_header("Time")),
            eco: self.get_header("ECO"),
            opening: self.get_header("Opening"),
            termination: self.get_header("Termination"),
            time_control: self.get_header("TimeControl"),
            movetext: self.movetext_buffer.trim().to_string(),
            parse_error: None,
        });
    }

    /// Spec: pgn-parsing - Error Message Capture
    /// Creates a partial GameRecord with available headers and a parse error message
    fn finalize_game_with_error(&mut self, error_msg: String) {
        let parse_elo = |s: &str| s.parse::<i32>().ok();

        self.current_game = Some(GameRecord {
            event: self.get_header("Event"),
            site: self.get_header("Site"),
            white: self.get_header("White"),
            black: self.get_header("Black"),
            result: self.get_header("Result"),
            white_title: self.get_header("WhiteTitle"),
            black_title: self.get_header("BlackTitle"),
            white_elo: self.get_header("WhiteElo").and_then(|s| parse_elo(&s)),
            black_elo: self.get_header("BlackElo").and_then(|s| parse_elo(&s)),
            utc_date: self.get_header("UTCDate").or_else(|| self.get_header("Date")),
            utc_time: self.get_header("UTCTime").or_else(|| self.get_header("Time")),
            eco: self.get_header("ECO"),
            opening: self.get_header("Opening"),
            termination: self.get_header("Termination"),
            time_control: self.get_header("TimeControl"),
            movetext: self.movetext_buffer.trim().to_string(),
            parse_error: Some(error_msg),
        });
    }
}

struct PgnReaderState {
    reader: BufReader<File>,
    path_idx: usize,
    game_buffer: String,
    record_buffer: GameRecord,
    line_buffer: Vec<u8>,
    visitor: GameVisitor,
}

impl PgnReaderState {
    fn new(reader: BufReader<File>, path_idx: usize) -> Self {
        Self {
            reader,
            path_idx,
            game_buffer: String::new(),
            record_buffer: GameRecord::default(),
            line_buffer: Vec::new(),
            visitor: GameVisitor::new(),
        }
    }
}

struct SharedState {
    next_path_idx: usize,
    available_readers: Vec<PgnReaderState>,
}

impl Visitor for GameVisitor {

    type Tags = ();
    type Movetext = String;
    type Output = ();

    fn begin_tags(&mut self) -> ControlFlow<Self::Output, Self::Tags> {
        self.headers.clear();
        self.movetext_buffer.clear();
        self.move_count = 0;
        self.current_game = None;
        ControlFlow::Continue(())
    }

    fn tag(&mut self, _: &mut Self::Tags, key: &[u8], value: RawTag<'_>) -> ControlFlow<Self::Output> {
        let key_str = String::from_utf8_lossy(key).to_string();
        let value_str = String::from_utf8_lossy(value.as_bytes()).to_string();
        self.headers.push((key_str, value_str));
        ControlFlow::Continue(())
    }

    fn begin_movetext(&mut self, _: Self::Tags) -> ControlFlow<Self::Output, Self::Movetext> {
        ControlFlow::Continue(String::new())
    }

    fn begin_variation(&mut self, _: &mut Self::Movetext) -> ControlFlow<Self::Output, Skip> {
        ControlFlow::Continue(Skip(true))
    }

    fn san(&mut self, movetext: &mut Self::Movetext, san: SanPlus) -> ControlFlow<Self::Output> {
        if self.move_count > 0 {
            movetext.push(' ');
        }
        if self.move_count % 2 == 0 {
            // White's move
            movetext.push_str(&format!("{}. ", (self.move_count / 2) + 1));
        }
        movetext.push_str(&san.to_string());
        self.move_count += 1;
        ControlFlow::Continue(())
    }

    fn comment(&mut self, movetext: &mut Self::Movetext, comment: RawComment<'_>) -> ControlFlow<Self::Output> {
        let comment_str = String::from_utf8_lossy(comment.as_bytes());
        movetext.push_str(&format!(" {{ {} }}", comment_str.trim()));
        ControlFlow::Continue(())
    }

    fn end_game(&mut self, movetext: Self::Movetext) -> Self::Output {
        self.movetext_buffer = movetext;
        self.finalize_game();
    }
}

#[repr(C)]
struct ReadPgnBindData {
    paths: Vec<PathBuf>,
}

#[repr(C)]
struct ReadPgnInitData {
    state: Mutex<SharedState>,
}

struct ReadPgnVTab;

impl VTab for ReadPgnVTab {
    type InitData = ReadPgnInitData;
    type BindData = ReadPgnBindData;

    fn bind(bind: &BindInfo) -> Result<Self::BindData, Box<dyn std::error::Error>> {
        let pattern = bind.get_parameter(0).to_string();

        // Spec: pgn-parsing - PGN File Reading
        // Expand glob pattern to get list of files (single file or glob pattern)
        let paths: Vec<PathBuf> = if pattern.contains('*') || pattern.contains('?') {
            // It's a glob pattern
            glob::glob(&pattern)?
                .filter_map(|entry| entry.ok())
                .collect()
        } else {
            // It's a single file path
            vec![PathBuf::from(pattern)]
        };

        // Spec: data-schema - Lichess Schema Compatibility
        // Extended Lichess dataset schema (16 base columns + 1 diagnostic column = 17 total)
        bind.add_result_column("Event", LogicalTypeHandle::from(LogicalTypeId::Varchar));
        bind.add_result_column("Site", LogicalTypeHandle::from(LogicalTypeId::Varchar));
        bind.add_result_column("White", LogicalTypeHandle::from(LogicalTypeId::Varchar));
        bind.add_result_column("Black", LogicalTypeHandle::from(LogicalTypeId::Varchar));
        bind.add_result_column("Result", LogicalTypeHandle::from(LogicalTypeId::Varchar));
        bind.add_result_column("WhiteTitle", LogicalTypeHandle::from(LogicalTypeId::Varchar));
        bind.add_result_column("BlackTitle", LogicalTypeHandle::from(LogicalTypeId::Varchar));
        bind.add_result_column("WhiteElo", LogicalTypeHandle::from(LogicalTypeId::Varchar));
        bind.add_result_column("BlackElo", LogicalTypeHandle::from(LogicalTypeId::Varchar));
        bind.add_result_column("UTCDate", LogicalTypeHandle::from(LogicalTypeId::Varchar));
        bind.add_result_column("UTCTime", LogicalTypeHandle::from(LogicalTypeId::Varchar));
        bind.add_result_column("ECO", LogicalTypeHandle::from(LogicalTypeId::Varchar));
        bind.add_result_column("Opening", LogicalTypeHandle::from(LogicalTypeId::Varchar));
        bind.add_result_column("Termination", LogicalTypeHandle::from(LogicalTypeId::Varchar));
        bind.add_result_column("TimeControl", LogicalTypeHandle::from(LogicalTypeId::Varchar));
        bind.add_result_column("movetext", LogicalTypeHandle::from(LogicalTypeId::Varchar));
        
        // Spec: data-schema - Parse Error Column
        // 17th column: diagnostic information about parsing failures (VARCHAR, nullable)
        bind.add_result_column("parse_error", LogicalTypeHandle::from(LogicalTypeId::Varchar));

        Ok(ReadPgnBindData { paths })
    }

    fn init(_: &InitInfo) -> Result<Self::InitData, Box<dyn std::error::Error>> {
        Ok(ReadPgnInitData {
            state: Mutex::new(SharedState {
                next_path_idx: 0,
                available_readers: Vec::new(),
            }),
        })
    }

    fn func(func: &TableFunctionInfo<Self>, output: &mut DataChunkHandle) -> Result<(), Box<dyn std::error::Error>> {
        let init_data = func.get_init_data();
        let bind_data = func.get_bind_data();

        let mut event_vec = output.flat_vector(0);
        let mut site_vec = output.flat_vector(1);
        let mut white_vec = output.flat_vector(2);
        let mut black_vec = output.flat_vector(3);
        let mut result_vec = output.flat_vector(4);
        let mut white_title_vec = output.flat_vector(5);
        let mut black_title_vec = output.flat_vector(6);
        let mut white_elo_vec = output.flat_vector(7);
        let mut black_elo_vec = output.flat_vector(8);
        let mut utc_date_vec = output.flat_vector(9);
        let mut utc_time_vec = output.flat_vector(10);
        let mut eco_vec = output.flat_vector(11);
        let mut opening_vec = output.flat_vector(12);
        let mut termination_vec = output.flat_vector(13);
        let mut time_control_vec = output.flat_vector(14);
        let mut movetext_vec = output.flat_vector(15);
        let mut parse_error_vec = output.flat_vector(16);

        let mut count = 0;
        let mut current_reader_state: Option<PgnReaderState> = None;

        while count < 2048 {
            // Acquire work
            if current_reader_state.is_none() {
                let mut state = init_data.state.lock().unwrap();
                
                if let Some(reader) = state.available_readers.pop() {
                    current_reader_state = Some(reader);
                } else if state.next_path_idx < bind_data.paths.len() {
                    let path_idx = state.next_path_idx;
                    state.next_path_idx += 1;
                    
                    // Unlock early to allow parallelism
                    drop(state);

                    let path = &bind_data.paths[path_idx];
                    match File::open(path) {
                        Ok(file) => {
                            current_reader_state = Some(PgnReaderState::new(BufReader::new(file), path_idx));
                        },
                        Err(e) => {
                            let err_msg = format!("Failed to open file '{}': {}", path.display(), e);
                            // If we only have one path (likely explicit single file), fail hard.
                            // If we have multiple (glob result), warn and skip.
                            if bind_data.paths.len() == 1 {
                                return Err(err_msg.into());
                            } else {
                                eprintln!("WARNING: {}", err_msg);
                                continue;
                            }
                        }
                    }
                } else {
                    // No more work
                    break;
                }
            }

            // Process using current reader
            if let Some(mut reader) = current_reader_state.take() {
                let mut game_found = false;
                
                // Read lines loop
                loop {
                    reader.line_buffer.clear();
                    match reader.reader.read_until(b'\n', &mut reader.line_buffer) {
                        Ok(0) => break, // EOF
                        Ok(_) => {
                            let mut line = String::from_utf8_lossy(&reader.line_buffer).into_owned();
                            if line.ends_with('\n') { line.pop(); if line.ends_with('\r') { line.pop(); } }

                            if line.starts_with("[Event ") {
                                if !reader.game_buffer.is_empty() {
                                    // Parse previous game
                                    let game_bytes = reader.game_buffer.as_bytes();
                                    let mut game_reader = Reader::new(game_bytes);
                                    
                                    // Reset visitor for new game
                                    reader.visitor.begin_tags(); // Manually clear (although read_game calls it)
                                    // Actually read_game calls begin_tags.
                                    
                                    // We need to handle the fact that visitor state is persistent in PgnReaderState
                                    // read_game calls visitor methods.
                                    
                                    match game_reader.read_game(&mut reader.visitor) {
                                        Ok(Some(_)) => {
                                            if let Some(game) = reader.visitor.current_game.take() {
                                                reader.record_buffer = game;
                                                game_found = true;
                                            }
                                        }
                                        Ok(None) => {}
                                        Err(e) => {
                                            let error_msg = format!("Error parsing game in file '{}': {}", 
                                                bind_data.paths[reader.path_idx].display(), e);
                                            reader.visitor.finalize_game_with_error(error_msg);
                                            if let Some(game) = reader.visitor.current_game.take() {
                                                reader.record_buffer = game;
                                                game_found = true;
                                            }
                                        }
                                    }
                                }

                                reader.game_buffer.clear();
                                reader.game_buffer.push_str(&line);
                                reader.game_buffer.push('\n');
                                
                                if game_found {
                                    break;
                                }
                            } else {
                                reader.game_buffer.push_str(&line);
                                reader.game_buffer.push('\n');
                            }
                        }
                        Err(e) => {
                            eprintln!("WARNING: Error reading file: {}", e);
                            break; // Treat as EOF/Error
                        }
                    }
                }

                // Handle EOF (last game in file)
                if !game_found && !reader.game_buffer.is_empty() {
                     // Try parse last game
                     let game_bytes = reader.game_buffer.as_bytes();
                     let mut game_reader = Reader::new(game_bytes);
                     match game_reader.read_game(&mut reader.visitor) {
                        Ok(Some(_)) => {
                            if let Some(game) = reader.visitor.current_game.take() {
                                reader.record_buffer = game;
                                game_found = true;
                            }
                        }
                         Ok(None) => {}
                         Err(e) => {
                             let error_msg = format!("Error parsing last game in file '{}': {}", 
                                 bind_data.paths[reader.path_idx].display(), e);
                             reader.visitor.finalize_game_with_error(error_msg);
                             if let Some(game) = reader.visitor.current_game.take() {
                                 reader.record_buffer = game;
                                 game_found = true;
                             }
                         }
                     }
                     reader.game_buffer.clear(); // Consumed
                }

                if game_found {
                    // Write to DuckDB
                    let game = &reader.record_buffer;
                    let i = count;
                    
                    if let Some(ref val) = game.event { event_vec.insert(i, CString::new(val.as_str())?); } else { event_vec.set_null(i); }
                    if let Some(ref val) = game.site { site_vec.insert(i, CString::new(val.as_str())?); } else { site_vec.set_null(i); }
                    if let Some(ref val) = game.white { white_vec.insert(i, CString::new(val.as_str())?); } else { white_vec.set_null(i); }
                    if let Some(ref val) = game.black { black_vec.insert(i, CString::new(val.as_str())?); } else { black_vec.set_null(i); }
                    if let Some(ref val) = game.result { result_vec.insert(i, CString::new(val.as_str())?); } else { result_vec.set_null(i); }
                    if let Some(ref val) = game.white_title { white_title_vec.insert(i, CString::new(val.as_str())?); } else { white_title_vec.set_null(i); }
                    if let Some(ref val) = game.black_title { black_title_vec.insert(i, CString::new(val.as_str())?); } else { black_title_vec.set_null(i); }
                    if let Some(val) = game.white_elo { white_elo_vec.insert(i, CString::new(val.to_string())?); } else { white_elo_vec.set_null(i); }
                    if let Some(val) = game.black_elo { black_elo_vec.insert(i, CString::new(val.to_string())?); } else { black_elo_vec.set_null(i); }
                    if let Some(ref val) = game.utc_date { utc_date_vec.insert(i, CString::new(val.as_str())?); } else { utc_date_vec.set_null(i); }
                    if let Some(ref val) = game.utc_time { utc_time_vec.insert(i, CString::new(val.as_str())?); } else { utc_time_vec.set_null(i); }
                    if let Some(ref val) = game.eco { eco_vec.insert(i, CString::new(val.as_str())?); } else { eco_vec.set_null(i); }
                    if let Some(ref val) = game.opening { opening_vec.insert(i, CString::new(val.as_str())?); } else { opening_vec.set_null(i); }
                    if let Some(ref val) = game.termination { termination_vec.insert(i, CString::new(val.as_str())?); } else { termination_vec.set_null(i); }
                    if let Some(ref val) = game.time_control { time_control_vec.insert(i, CString::new(val.as_str())?); } else { time_control_vec.set_null(i); }
                    
                    movetext_vec.insert(i, CString::new(game.movetext.as_str())?);
                    
                    if let Some(ref val) = game.parse_error { parse_error_vec.insert(i, CString::new(val.as_str())?); } else { parse_error_vec.set_null(i); }

                    count += 1;
                    current_reader_state = Some(reader); // Return reader to local var for next iteration
                } else {
                    // Reader finished (EOF)
                    // It will be dropped here (current_reader_state remains None)
                    // Loop continues to acquire next reader
                }
            }
        }

        // Return reader to pool if we stopped due to count limit
        if let Some(reader) = current_reader_state {
            let mut state = init_data.state.lock().unwrap();
            state.available_readers.push(reader);
        }

        output.set_len(count);
        Ok(())
    }

    fn parameters() -> Option<Vec<LogicalTypeHandle>> {
        // Only declare first parameter as required
        // Additional parameters can still be passed and will be detected via get_parameter_count()
        Some(vec![
            LogicalTypeHandle::from(LogicalTypeId::Varchar), // path pattern (required)
        ])
    }
}

const EXTENSION_NAME: &str = "read_pgn";

/// Spec: annotation-filtering - Movetext Annotation Removal
/// Removes curly brace annotations from chess movetext while preserving move structure
/// Spec: annotation-filtering - Nested Annotation Handling (tracks brace depth)
/// Spec: annotation-filtering - Whitespace Normalization (collapses spaces and trims)
fn filter_movetext_annotations(movetext: &str) -> String {
    let mut result = String::new();
    let mut in_annotation = false;
    let mut brace_depth = 0;
    let mut prev_was_space = false;

    for ch in movetext.chars() {
        match ch {
            '{' => {
                in_annotation = true;
                brace_depth += 1;
            }
            '}' => {
                brace_depth -= 1;
                if brace_depth == 0 {
                    in_annotation = false;
                    // Mark that we should skip next space if any
                    prev_was_space = true;
                }
            }
            ' ' if !in_annotation => {
                if !prev_was_space && !result.is_empty() {
                    result.push(' ');
                    prev_was_space = true;
                }
            }
            _ if !in_annotation => {
                prev_was_space = false;
                result.push(ch);
            }
            _ => {}
        }
    }

    result.trim().to_string()
}

#[repr(C)]
struct FilterMovetextBindData {
    movetext: String,
}

#[repr(C)]
struct FilterMovetextInitData {
    done: AtomicBool,
}

struct FilterMovetextVTab;

impl VTab for FilterMovetextVTab {
    type InitData = FilterMovetextInitData;
    type BindData = FilterMovetextBindData;

    fn bind(bind: &BindInfo) -> Result<Self::BindData, Box<dyn std::error::Error>> {
        let movetext = bind.get_parameter(0).to_string();
        
        bind.add_result_column("filtered_movetext", LogicalTypeHandle::from(LogicalTypeId::Varchar));

        Ok(FilterMovetextBindData { movetext })
    }

    fn init(_: &InitInfo) -> Result<Self::InitData, Box<dyn std::error::Error>> {
        Ok(FilterMovetextInitData {
            done: AtomicBool::new(false),
        })
    }

    fn func(func: &TableFunctionInfo<Self>, output: &mut DataChunkHandle) -> Result<(), Box<dyn std::error::Error>> {
        let init_data = func.get_init_data();
        let bind_data = func.get_bind_data();

        if init_data.done.swap(true, Ordering::Relaxed) {
            output.set_len(0);
            return Ok(());
        }

        let filtered = filter_movetext_annotations(&bind_data.movetext);
        let result_vec = output.flat_vector(0);
        result_vec.insert(0, CString::new(filtered)?);
        output.set_len(1);

        Ok(())
    }

    fn parameters() -> Option<Vec<LogicalTypeHandle>> {
        Some(vec![
            LogicalTypeHandle::from(LogicalTypeId::Varchar),
        ])
    }
}

#[duckdb_entrypoint_c_api(ext_name = "duckdb_chess", min_duckdb_version = "v1.0.0")]
pub unsafe fn extension_entrypoint(con: Connection) -> Result<(), Box<dyn Error>> {
    con.register_table_function::<ReadPgnVTab>(EXTENSION_NAME)?;
    con.register_table_function::<FilterMovetextVTab>("filter_movetext_annotations")?;
    Ok(())
}
