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
    /// Contains NULL for successfully parsed games or error message for failed games
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
        });
    }

    /// Creates a partial GameRecord with available headers and a parse error message
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
    done: AtomicBool,
    games: std::sync::Mutex<Option<Vec<GameRecord>>>,
    offset: std::sync::atomic::AtomicUsize,
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
        
        // 17th column: diagnostic information about parsing failures (VARCHAR, nullable)

        Ok(ReadPgnBindData { paths })
    }

    fn init(_: &InitInfo) -> Result<Self::InitData, Box<dyn std::error::Error>> {
        Ok(ReadPgnInitData {
            done: AtomicBool::new(false),
            games: std::sync::Mutex::new(None),
            offset: std::sync::atomic::AtomicUsize::new(0),
        })
    }

    fn func(func: &TableFunctionInfo<Self>, output: &mut DataChunkHandle) -> Result<(), Box<dyn std::error::Error>> {
        let init_data = func.get_init_data();
        let bind_data = func.get_bind_data();

        // Spec: pgn-parsing - Thread Safety
        // Parse all PGN files on first call (atomic flag ensures single initialization)
        if !init_data.done.swap(true, Ordering::Relaxed) {
            let mut games = Vec::new();
            let mut visitor = GameVisitor::new();

            // Spec: pgn-parsing - PGN File Reading (Glob pattern parsing)
            // Iterate over all paths (from glob expansion or single file)
            for path in &bind_data.paths {
                let file = File::open(path).map_err(|e| {
                    format!("Failed to open file '{}': {}", path.display(), e)
                })?;
                let mut reader = BufReader::new(file);

                // Spec: pgn-parsing - Game Boundary Detection
                // Split file into individual games and parse each separately
                let mut current_game_text = String::new();
                let mut game_number = 0;
                let mut in_game = false;
                let mut buffer = Vec::new();

                for line_result in reader.lines() {
                    let line = match line_result {
                        Ok(l) => l,
                        Err(e) => {
                            eprintln!("WARNING: Error reading line in file '{}': {} (skipped)", path.display(), e);
                            continue;
                        }
                    };

                    // Check if we're starting a new game (starts with [Event header)
                    if line.starts_with("[Event ") {
                        // If we have accumulated a previous game, try to parse it
                        if in_game && !current_game_text.is_empty() {
                            game_number += 1;
                            // Try to parse the accumulated game
                            let game_bytes = current_game_text.as_bytes();
                            let mut game_reader = Reader::new(game_bytes);
                            match game_reader.read_game(&mut visitor) {
                                Ok(Some(_)) => {
                                    if let Some(game) = visitor.current_game.take() {
                                        games.push(game);
                                    }
                                }
                                Ok(None) => {}
                                Err(e) => {
                                    // Spec: pgn-parsing - Malformed Game Handling
                                    eprintln!("WARNING: Error parsing game #{} in file '{}': {}", 
                                                game_number, path.display(), e);
                                }
                            }
                        }

                        // Start new game
                        current_game_text.clear();
                        current_game_text.push_str(&line);
                        current_game_text.push('\n');
                        in_game = true;
                    } else if in_game {
                        // Continue accumulating current game
                        current_game_text.push_str(&line);
                        current_game_text.push('\n');
                    }
                }
                
                // Don't forget the last game in the file
                if in_game && !current_game_text.is_empty() {
                    game_number += 1;
                    let game_bytes = current_game_text.as_bytes();
                    let mut game_reader = Reader::new(game_bytes);
                    match game_reader.read_game(&mut visitor) {
                        Ok(Some(_)) => {
                            if let Some(game) = visitor.current_game.take() {
                                games.push(game);
                            }
                        }
                        Ok(None) => {}
                        Err(e) => {
                            // Spec: pgn-parsing - Malformed Game Handling
                            // Capture partial game data with error message instead of skipping
                            eprintln!("WARNING: Error parsing game #{} in file '{}': {}", 
                                game_number, path.display(), e);
                        }
                    }
                }
            }

            *init_data.games.lock().unwrap() = Some(games);
        }

        // Output games in chunks
        let current_offset = init_data.offset.load(Ordering::Relaxed);

        let games_guard = init_data.games.lock().unwrap();
        let games = match games_guard.as_ref() {
            Some(g) => g,
            None => {
                output.set_len(0);
                return Ok(());
            }
        };

        // Check if we've output all games
        if current_offset >= games.len() {
            output.set_len(0);
            return Ok(());
        }

        // Spec: pgn-parsing - Chunked Output
        // Output up to 2048 games per chunk to manage memory efficiently
        let remaining = games.len() - current_offset;
        let count = remaining.min(2048);

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
        let movetext_vec = output.flat_vector(15);

        for (i, game) in games.iter().skip(current_offset).take(count).enumerate() {
            // Spec: data-schema - NULL Value Handling
            // Insert strings with proper NULL handling (NULL for missing headers, not empty strings)
            if let Some(ref val) = game.event {
                event_vec.insert(i, CString::new(val.as_str())?);
            } else {
                event_vec.set_null(i);
            }
            
            if let Some(ref val) = game.site {
                site_vec.insert(i, CString::new(val.as_str())?);
            } else {
                site_vec.set_null(i);
            }
            
            if let Some(ref val) = game.white {
                white_vec.insert(i, CString::new(val.as_str())?);
            } else {
                white_vec.set_null(i);
            }
            
            if let Some(ref val) = game.black {
                black_vec.insert(i, CString::new(val.as_str())?);
            } else {
                black_vec.set_null(i);
            }
            
            if let Some(ref val) = game.result {
                result_vec.insert(i, CString::new(val.as_str())?);
            } else {
                result_vec.set_null(i);
            }
            
            if let Some(ref val) = game.white_title {
                white_title_vec.insert(i, CString::new(val.as_str())?);
            } else {
                white_title_vec.set_null(i);
            }
            
            if let Some(ref val) = game.black_title {
                black_title_vec.insert(i, CString::new(val.as_str())?);
            } else {
                black_title_vec.set_null(i);
            }

            // Insert ELO ratings as strings (storing integers as VARCHAR)
            if let Some(elo) = game.white_elo {
                white_elo_vec.insert(i, CString::new(elo.to_string())?);
            } else {
                white_elo_vec.set_null(i);
            }
            
            if let Some(elo) = game.black_elo {
                black_elo_vec.insert(i, CString::new(elo.to_string())?);
            } else {
                black_elo_vec.set_null(i);
            }

            // Insert date/time
            if let Some(ref val) = game.utc_date {
                utc_date_vec.insert(i, CString::new(val.as_str())?);
            } else {
                utc_date_vec.set_null(i);
            }
            
            if let Some(ref val) = game.utc_time {
                utc_time_vec.insert(i, CString::new(val.as_str())?);
            } else {
                utc_time_vec.set_null(i);
            }

            // Insert opening info
            if let Some(ref val) = game.eco {
                eco_vec.insert(i, CString::new(val.as_str())?);
            } else {
                eco_vec.set_null(i);
            }
            
            if let Some(ref val) = game.opening {
                opening_vec.insert(i, CString::new(val.as_str())?);
            } else {
                opening_vec.set_null(i);
            }

            // Insert game details
            if let Some(ref val) = game.termination {
                termination_vec.insert(i, CString::new(val.as_str())?);
            } else {
                termination_vec.set_null(i);
            }
            
            if let Some(ref val) = game.time_control {
                time_control_vec.insert(i, CString::new(val.as_str())?);
            } else {
                time_control_vec.set_null(i);
            }

            // Spec: data-schema - Movetext Column (Movetext always present)
            // Insert movetext (always present, never NULL - empty string if no moves)
            movetext_vec.insert(i, CString::new(game.movetext.as_str())?);

            } else {
            }
        }

        output.set_len(count);

        // Update offset for next call
        init_data.offset.fetch_add(count, Ordering::Relaxed);

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
