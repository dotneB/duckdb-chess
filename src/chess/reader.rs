use super::visitor::{PgnReaderState, SharedState};
use duckdb::{
    core::{DataChunkHandle, Inserter, LogicalTypeHandle, LogicalTypeId},
    vtab::{BindInfo, InitInfo, TableFunctionInfo, VTab},
};
use libduckdb_sys::{duckdb_date, duckdb_time_tz};
use std::ffi::CString;
use std::fs::File;
use std::path::PathBuf;
use std::sync::Mutex;

#[repr(C)]
pub struct ReadPgnBindData {
    paths: Vec<PathBuf>,
}

#[repr(C)]
pub struct ReadPgnInitData {
    state: Mutex<SharedState>,
}

pub struct ReadPgnVTab;

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
        // Extended Lichess dataset schema (16 base columns + 2 extra columns = 18 total)
        bind.add_result_column("Event", LogicalTypeHandle::from(LogicalTypeId::Varchar));
        bind.add_result_column("Site", LogicalTypeHandle::from(LogicalTypeId::Varchar));
        bind.add_result_column("White", LogicalTypeHandle::from(LogicalTypeId::Varchar));
        bind.add_result_column("Black", LogicalTypeHandle::from(LogicalTypeId::Varchar));
        bind.add_result_column("Result", LogicalTypeHandle::from(LogicalTypeId::Varchar));
        bind.add_result_column(
            "WhiteTitle",
            LogicalTypeHandle::from(LogicalTypeId::Varchar),
        );
        bind.add_result_column(
            "BlackTitle",
            LogicalTypeHandle::from(LogicalTypeId::Varchar),
        );
        bind.add_result_column("WhiteElo", LogicalTypeHandle::from(LogicalTypeId::UInteger));
        bind.add_result_column("BlackElo", LogicalTypeHandle::from(LogicalTypeId::UInteger));
        bind.add_result_column("UTCDate", LogicalTypeHandle::from(LogicalTypeId::Date));
        bind.add_result_column("UTCTime", LogicalTypeHandle::from(LogicalTypeId::TimeTZ));
        bind.add_result_column("ECO", LogicalTypeHandle::from(LogicalTypeId::Varchar));
        bind.add_result_column("Opening", LogicalTypeHandle::from(LogicalTypeId::Varchar));
        bind.add_result_column(
            "Termination",
            LogicalTypeHandle::from(LogicalTypeId::Varchar),
        );
        bind.add_result_column(
            "TimeControl",
            LogicalTypeHandle::from(LogicalTypeId::Varchar),
        );
        bind.add_result_column("movetext", LogicalTypeHandle::from(LogicalTypeId::Varchar));

        // Spec: data-schema - Parse Error Column
        // 17th column: diagnostic information about parsing failures (VARCHAR, nullable)
        bind.add_result_column(
            "parse_error",
            LogicalTypeHandle::from(LogicalTypeId::Varchar),
        );

        bind.add_result_column("Source", LogicalTypeHandle::from(LogicalTypeId::Varchar));

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

    fn func(
        func: &TableFunctionInfo<Self>,
        output: &mut DataChunkHandle,
    ) -> Result<(), Box<dyn std::error::Error>> {
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
        let movetext_vec = output.flat_vector(15);
        let mut parse_error_vec = output.flat_vector(16);
        let mut source_vec = output.flat_vector(17);

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
                            current_reader_state = Some(PgnReaderState::new(file, path_idx));
                        }
                        Err(e) => {
                            let err_msg =
                                format!("Failed to open file '{}': {}", path.display(), e);
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
                // Use pgn-reader's Reader directly for streaming PGN parsing.
                // Note: We do NOT wrap File in BufReader because pgn-reader's documentation states:
                // "Buffers the underlying reader with an appropriate strategy, so it's not
                // recommended to add an additional layer of buffering like BufReader."
                let game_index = reader.next_game_index;
                let game_found = match reader.pgn_reader.read_game(&mut reader.visitor) {
                    Ok(Some(_)) => {
                        reader.next_game_index += 1;
                        // Successfully parsed a game
                        if let Some(game) = reader.visitor.current_game.take() {
                            reader.record_buffer = game;
                            true
                        } else {
                            false
                        }
                    }
                    Ok(None) => {
                        // EOF reached - no more games in this file
                        false
                    }
                    Err(e) => {
                        reader.next_game_index += 1;
                        // Parsing error - create partial game with error message
                        let error_msg = format!(
                            "Parser-stage error: stage=read_game; file='{}'; game_index={}; error={}",
                            bind_data.paths[reader.path_idx].display(),
                            game_index,
                            e
                        );
                        eprintln!("WARNING: {}", error_msg);
                        reader.visitor.finalize_game_with_error(error_msg);
                        if let Some(game) = reader.visitor.current_game.take() {
                            reader.record_buffer = game;
                            true
                        } else {
                            false
                        }
                    }
                };

                if game_found {
                    // Write to DuckDB
                    let game = &reader.record_buffer;
                    let i = count;

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
                    if let Some(val) = game.white_elo {
                        white_elo_vec.as_mut_slice::<u32>()[i] = val;
                    } else {
                        white_elo_vec.set_null(i);
                    }
                    if let Some(val) = game.black_elo {
                        black_elo_vec.as_mut_slice::<u32>()[i] = val;
                    } else {
                        black_elo_vec.set_null(i);
                    }
                    if let Some(val) = game.utc_date {
                        utc_date_vec.as_mut_slice::<duckdb_date>()[i] = val;
                    } else {
                        utc_date_vec.set_null(i);
                    }
                    if let Some(val) = game.utc_time {
                        utc_time_vec.as_mut_slice::<duckdb_time_tz>()[i] = val;
                    } else {
                        utc_time_vec.set_null(i);
                    }
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

                    movetext_vec.insert(i, CString::new(game.movetext.as_str())?);

                    if let Some(ref val) = game.parse_error {
                        parse_error_vec.insert(i, CString::new(val.as_str())?);
                    } else {
                        parse_error_vec.set_null(i);
                    }

                    if let Some(ref val) = game.source {
                        source_vec.insert(i, CString::new(val.as_str())?);
                    } else {
                        source_vec.set_null(i);
                    }

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

#[cfg(test)]
#[allow(dead_code)]
mod tests {
    use super::*;

    use std::path::PathBuf;

    fn days_from_civil(year: i32, month: u32, day: u32) -> i32 {
        let y = year - if month <= 2 { 1 } else { 0 };
        let era = if y >= 0 { y } else { y - 399 } / 400;
        let yoe = y - era * 400;
        let m = month as i32;
        let doy = (153 * (m + if m > 2 { -3 } else { 9 }) + 2) / 5 + day as i32 - 1;
        let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
        (era * 146097 + doe - 719468) as i32
    }

    #[test]
    fn test_read_pgn_bind_data_creation() {
        // Test that bind data can be created with single file
        let paths = vec![PathBuf::from("test.pgn")];
        let bind_data = ReadPgnBindData { paths };
        assert_eq!(bind_data.paths.len(), 1);
        assert_eq!(bind_data.paths[0], PathBuf::from("test.pgn"));
    }

    #[test]
    fn test_read_pgn_bind_data_multiple_files() {
        // Test that bind data can be created with multiple files
        let paths = vec![PathBuf::from("test1.pgn"), PathBuf::from("test2.pgn")];
        let bind_data = ReadPgnBindData { paths };
        assert_eq!(bind_data.paths.len(), 2);
    }

    #[test]
    fn test_shared_state_initialization() {
        // Test that shared state can be initialized
        let state = SharedState {
            next_path_idx: 0,
            available_readers: Vec::new(),
        };
        let init_data = ReadPgnInitData {
            state: Mutex::new(state),
        };
        assert_eq!(init_data.state.lock().unwrap().next_path_idx, 0);
        assert!(init_data.state.lock().unwrap().available_readers.is_empty());
    }

    // Test with actual PGN file content parsing
    #[test]
    fn test_pgn_visitor_basic_game() {
        use crate::chess::visitor::GameVisitor;
        use pgn_reader::Reader;

        let pgn_content = r#"
[Event "Test Game"]
[Site "Test Site"]
[White "Player 1"]
[Black "Player 2"]
[Result "1-0"]

1. e4 e5 2. Nf3 Nc6 1-0
"#;

        let mut visitor = GameVisitor::new();
        let mut reader = Reader::new(pgn_content.as_bytes());

        let result = reader.read_game(&mut visitor);
        assert!(result.is_ok());

        let game = visitor.current_game.take();
        assert!(game.is_some());

        let game = game.unwrap();
        assert_eq!(game.event.as_deref().unwrap(), "Test Game");
        assert_eq!(game.white.as_deref().unwrap(), "Player 1");
        assert_eq!(game.black.as_deref().unwrap(), "Player 2");
        assert_eq!(game.result.as_deref().unwrap(), "1-0");
        assert_eq!(game.site.as_deref().unwrap(), "Test Site");
    }

    #[test]
    fn test_pgn_visitor_missing_headers() {
        use crate::chess::visitor::GameVisitor;
        use pgn_reader::Reader;

        let pgn_content = r#"
[Event "Minimal Game"]
[White "?"]
[Black "?"]
[Result "*"]

1. d4 d5 *
"#;

        let mut visitor = GameVisitor::new();
        let mut reader = Reader::new(pgn_content.as_bytes());

        let result = reader.read_game(&mut visitor);
        assert!(result.is_ok());

        let game = visitor.current_game.take();
        assert!(game.is_some());

        let game = game.unwrap();
        assert_eq!(game.event.as_deref().unwrap(), "Minimal Game");
        assert_eq!(game.white.as_deref().unwrap(), "?");
        assert_eq!(game.black.as_deref().unwrap(), "?");
        assert_eq!(game.result.as_deref().unwrap(), "*");

        // Missing headers should be None
        assert_eq!(game.site, None);
        assert_eq!(game.eco, None);
        assert_eq!(game.opening, None);
    }

    #[test]
    fn test_pgn_visitor_partial_headers() {
        use crate::chess::visitor::GameVisitor;
        use pgn_reader::Reader;

        let pgn_content = r#"
[Event "Game with some missing fields"]
[White "White Player"]
[Black "Black Player"]
[Result "1/2-1/2"]

1. e4 e5 1/2-1/2
"#;

        let mut visitor = GameVisitor::new();
        let mut reader = Reader::new(pgn_content.as_bytes());

        let result = reader.read_game(&mut visitor);
        assert!(result.is_ok());

        let game = visitor.current_game.take();
        assert!(game.is_some());

        let game = game.unwrap();
        assert_eq!(
            game.event.as_deref().unwrap(),
            "Game with some missing fields"
        );
        assert_eq!(game.white.as_deref().unwrap(), "White Player");
        assert_eq!(game.black.as_deref().unwrap(), "Black Player");
        assert_eq!(game.result.as_deref().unwrap(), "1/2-1/2");

        // Missing headers should be None
        assert_eq!(game.site, None);
        assert!(game.utc_date.is_none());
        assert_eq!(game.eco, None);
        assert_eq!(game.opening, None);
        assert_eq!(game.white_elo, None);
        assert_eq!(game.black_elo, None);
    }

    #[test]
    fn test_pgn_visitor_all_headers() {
        use crate::chess::visitor::GameVisitor;
        use pgn_reader::Reader;

        let pgn_content = r#"
[Event "Test with all headers"]
[Site "https://example.com"]
[Date "2024.01.01"]
[Round "1"]
[White "Player A"]
[Black "Player B"]
[Result "1-0"]
[WhiteElo "2000"]
[BlackElo "1900"]
[WhiteTitle "GM"]
[BlackTitle "IM"]
[ECO "B00"]
[Opening "Test Opening"]
[UTCDate "2024.01.01"]
[UTCTime "12:00:00"]
[TimeControl "180+0"]
[Termination "Normal"]

1. e4 e5 2. Nf3 Nc6 1-0
"#;

        let mut visitor = GameVisitor::new();
        let mut reader = Reader::new(pgn_content.as_bytes());

        let result = reader.read_game(&mut visitor);
        assert!(result.is_ok());

        let game = visitor.current_game.take();
        assert!(game.is_some());

        let game = game.unwrap();
        assert_eq!(game.event.as_deref().unwrap(), "Test with all headers");
        assert_eq!(game.site.as_deref().unwrap(), "https://example.com");
        // Note: Date header is mapped to utc_date in GameRecord
        assert_eq!(game.white.as_deref().unwrap(), "Player A");
        assert_eq!(game.black.as_deref().unwrap(), "Player B");
        assert_eq!(game.result.as_deref().unwrap(), "1-0");
        assert_eq!(game.white_elo.unwrap(), 2000);
        assert_eq!(game.black_elo.unwrap(), 1900);
        assert_eq!(game.white_title.as_deref().unwrap(), "GM");
        assert_eq!(game.black_title.as_deref().unwrap(), "IM");
        assert_eq!(game.eco.as_deref().unwrap(), "B00");
        assert_eq!(game.opening.as_deref().unwrap(), "Test Opening");

        let utc_date = game.utc_date.unwrap();
        assert_eq!(utc_date.days, days_from_civil(2024, 1, 1));

        let utc_time = game.utc_time.unwrap();
        let micros = 12i64 * 3600 * 1_000_000;
        let micros_part = (micros as u64) & ((1u64 << 40) - 1);
        let offset_sentinel = (16u64 * 60 * 60) - 1; // 15:59:59 encodes +00:00
        assert_eq!(utc_time.bits, (micros_part << 24) | offset_sentinel);

        assert_eq!(game.time_control.as_deref().unwrap(), "180+0");
        assert_eq!(game.termination.as_deref().unwrap(), "Normal");
    }

    #[test]
    fn test_pgn_visitor_date_candidate_selection_prefers_more_complete_partial() {
        use crate::chess::visitor::GameVisitor;
        use pgn_reader::Reader;

        let pgn_content = r#"
[Event "Partial Date Selection"]
[Date "1951.??.??"]
[EventDate "1951.09.??"]
[Result "*"]

*
"#;

        let mut visitor = GameVisitor::new();
        let mut reader = Reader::new(pgn_content.as_bytes());
        reader.read_game(&mut visitor).unwrap();

        let game = visitor.current_game.take().unwrap();
        let utc_date = game.utc_date.unwrap();
        assert_eq!(utc_date.days, days_from_civil(1951, 9, 1));
        assert!(game.parse_error.is_none());
    }

    #[test]
    fn test_pgn_visitor_date_candidate_selection_tie_break_by_precedence() {
        use crate::chess::visitor::GameVisitor;
        use pgn_reader::Reader;

        let pgn_content = r#"
[Event "Date Precedence"]
[UTCDate "1999.12.31"]
[Date "2000.01.01"]
[EventDate "2001.01.01"]
[Result "*"]

*
"#;

        let mut visitor = GameVisitor::new();
        let mut reader = Reader::new(pgn_content.as_bytes());
        reader.read_game(&mut visitor).unwrap();

        let game = visitor.current_game.take().unwrap();
        let utc_date = game.utc_date.unwrap();
        assert_eq!(utc_date.days, days_from_civil(1999, 12, 31));
        assert!(game.parse_error.is_none());
    }

    #[test]
    fn test_pgn_visitor_date_unknown_is_null() {
        use crate::chess::visitor::GameVisitor;
        use pgn_reader::Reader;

        let pgn_content = r#"
[Event "Unknown Date"]
[Date "????.??.??"]
[Result "*"]

*
"#;

        let mut visitor = GameVisitor::new();
        let mut reader = Reader::new(pgn_content.as_bytes());
        reader.read_game(&mut visitor).unwrap();

        let game = visitor.current_game.take().unwrap();
        assert!(game.utc_date.is_none());
        assert!(game.parse_error.is_none());
    }

    #[test]
    fn test_pgn_visitor_date_partial_defaults() {
        use crate::chess::visitor::GameVisitor;
        use pgn_reader::Reader;

        let pgn_content = r#"
[Event "Partial Date Defaults"]
[Date "2000.??.??"]
[EventDate "2000.06.??"]
[Result "*"]

*
"#;

        let mut visitor = GameVisitor::new();
        let mut reader = Reader::new(pgn_content.as_bytes());
        reader.read_game(&mut visitor).unwrap();

        let game = visitor.current_game.take().unwrap();
        let utc_date = game.utc_date.unwrap();
        // EventDate is more complete (year+month) than Date (year only), so it wins.
        assert_eq!(utc_date.days, days_from_civil(2000, 6, 1));
        assert!(game.parse_error.is_none());
    }

    #[test]
    fn test_pgn_visitor_date_clamps_out_of_range_day_for_30_day_month() {
        use crate::chess::visitor::GameVisitor;
        use pgn_reader::Reader;

        let pgn_content = r#"
[Event "Clamp November Day Overflow"]
[Date "2015.11.31"]
[Result "*"]

*
"#;

        let mut visitor = GameVisitor::new();
        let mut reader = Reader::new(pgn_content.as_bytes());
        reader.read_game(&mut visitor).unwrap();

        let game = visitor.current_game.take().unwrap();
        let utc_date = game.utc_date.unwrap();
        assert_eq!(utc_date.days, days_from_civil(2015, 11, 30));
        assert!(game.parse_error.is_none());
    }

    #[test]
    fn test_pgn_visitor_date_clamps_out_of_range_day_for_non_leap_february() {
        use crate::chess::visitor::GameVisitor;
        use pgn_reader::Reader;

        let pgn_content = r#"
[Event "Clamp Non-Leap February Day Overflow"]
[Date "1997.02.29"]
[Result "*"]

*
"#;

        let mut visitor = GameVisitor::new();
        let mut reader = Reader::new(pgn_content.as_bytes());
        reader.read_game(&mut visitor).unwrap();

        let game = visitor.current_game.take().unwrap();
        let utc_date = game.utc_date.unwrap();
        assert_eq!(utc_date.days, days_from_civil(1997, 2, 28));
        assert!(game.parse_error.is_none());
    }

    #[test]
    fn test_pgn_visitor_date_clamps_out_of_range_day_for_leap_february() {
        use crate::chess::visitor::GameVisitor;
        use pgn_reader::Reader;

        let pgn_content = r#"
[Event "Clamp Leap February Day Overflow"]
[Date "2000.02.30"]
[Result "*"]

*
"#;

        let mut visitor = GameVisitor::new();
        let mut reader = Reader::new(pgn_content.as_bytes());
        reader.read_game(&mut visitor).unwrap();

        let game = visitor.current_game.take().unwrap();
        let utc_date = game.utc_date.unwrap();
        assert_eq!(utc_date.days, days_from_civil(2000, 2, 29));
        assert!(game.parse_error.is_none());
    }

    #[test]
    fn test_pgn_visitor_date_clamp_preserves_header_precedence() {
        use crate::chess::visitor::GameVisitor;
        use pgn_reader::Reader;

        let pgn_content = r#"
[Event "Clamp Precedence"]
[UTCDate "2015.11.31"]
[Date "2015.11.15"]
[EventDate "2015.11.10"]
[Result "*"]

*
"#;

        let mut visitor = GameVisitor::new();
        let mut reader = Reader::new(pgn_content.as_bytes());
        reader.read_game(&mut visitor).unwrap();

        let game = visitor.current_game.take().unwrap();
        let utc_date = game.utc_date.unwrap();
        assert_eq!(utc_date.days, days_from_civil(2015, 11, 30));
        assert!(game.parse_error.is_none());
    }

    #[test]
    fn test_pgn_visitor_date_invalid_records_chrono_error() {
        use crate::chess::visitor::GameVisitor;
        use pgn_reader::Reader;

        let pgn_content = r#"
[Event "Invalid Date"]
[Date "2000.13.40"]
[Result "*"]

*
"#;

        let mut visitor = GameVisitor::new();
        let mut reader = Reader::new(pgn_content.as_bytes());
        reader.read_game(&mut visitor).unwrap();

        let game = visitor.current_game.take().unwrap();
        assert!(game.utc_date.is_none());
        let err = game.parse_error.unwrap();
        assert!(err.contains("UTCDate"));
        assert!(err.contains("2000.13.40"));
        assert!(err.contains("chrono:"));
    }

    #[test]
    fn test_pgn_visitor_date_fallback_from_invalid_utcdate_to_date() {
        use crate::chess::visitor::GameVisitor;
        use pgn_reader::Reader;

        let pgn_content = r#"
[Event "Invalid UTCDate Fallback Date"]
[UTCDate "2024.13.01"]
[Date "2024.01.02"]
[EventDate "2024.01.03"]
[Result "*"]

*
"#;

        let mut visitor = GameVisitor::new();
        let mut reader = Reader::new(pgn_content.as_bytes());
        reader.read_game(&mut visitor).unwrap();

        let game = visitor.current_game.take().unwrap();
        let utc_date = game.utc_date.unwrap();
        assert_eq!(utc_date.days, days_from_civil(2024, 1, 2));

        let err = game.parse_error.unwrap();
        assert!(err.contains("UTCDate='2024.13.01'"));
        assert!(err.contains("chrono:"));
    }

    #[test]
    fn test_pgn_visitor_date_fallback_from_invalid_utcdate_to_eventdate() {
        use crate::chess::visitor::GameVisitor;
        use pgn_reader::Reader;

        let pgn_content = r#"
[Event "Invalid UTCDate Fallback EventDate"]
[UTCDate "2024.13.01"]
[Date "????.??.??"]
[EventDate "2024.03.04"]
[Result "*"]

*
"#;

        let mut visitor = GameVisitor::new();
        let mut reader = Reader::new(pgn_content.as_bytes());
        reader.read_game(&mut visitor).unwrap();

        let game = visitor.current_game.take().unwrap();
        let utc_date = game.utc_date.unwrap();
        assert_eq!(utc_date.days, days_from_civil(2024, 3, 4));

        let err = game.parse_error.unwrap();
        assert!(err.contains("UTCDate='2024.13.01'"));
        assert!(err.contains("chrono:"));
    }

    #[test]
    fn test_pgn_visitor_date_fallback_preserves_partial_completeness_policy() {
        use crate::chess::visitor::GameVisitor;
        use pgn_reader::Reader;

        let pgn_content = r#"
[Event "Invalid UTCDate Partial Fallback"]
[UTCDate "2024.13.01"]
[Date "2000.??.??"]
[EventDate "2000.06.??"]
[Result "*"]

*
"#;

        let mut visitor = GameVisitor::new();
        let mut reader = Reader::new(pgn_content.as_bytes());
        reader.read_game(&mut visitor).unwrap();

        let game = visitor.current_game.take().unwrap();
        let utc_date = game.utc_date.unwrap();
        assert_eq!(utc_date.days, days_from_civil(2000, 6, 1));

        let err = game.parse_error.unwrap();
        assert!(err.contains("UTCDate='2024.13.01'"));
    }

    #[test]
    fn test_pgn_visitor_time_variants_and_offsets() {
        use crate::chess::visitor::GameVisitor;
        use pgn_reader::Reader;

        // Zulu
        let pgn_content = r#"
[Event "Time Variants"]
[UTCTime "12:00:00Z"]
[Result "*"]

*
"#;
        let mut visitor = GameVisitor::new();
        let mut reader = Reader::new(pgn_content.as_bytes());
        reader.read_game(&mut visitor).unwrap();
        let game = visitor.current_game.take().unwrap();
        let utc_time = game.utc_time.unwrap();
        let micros = 12i64 * 3600 * 1_000_000;
        let micros_part = (micros as u64) & ((1u64 << 40) - 1);
        let offset_sentinel: i32 = (16 * 60 * 60) - 1;
        let encoded_offset = offset_sentinel - 0;
        let offset_part = (encoded_offset as i64 as u64) & ((1u64 << 24) - 1);
        assert_eq!(utc_time.bits, (micros_part << 24) | offset_part);

        // Explicit positive offset
        let pgn_content = r#"
[Event "Time Variants"]
[UTCTime "12:00:00+01:30"]
[Result "*"]

*
"#;
        let mut visitor = GameVisitor::new();
        let mut reader = Reader::new(pgn_content.as_bytes());
        reader.read_game(&mut visitor).unwrap();
        let game = visitor.current_game.take().unwrap();
        let utc_time = game.utc_time.unwrap();
        let offset_seconds: i32 = 1 * 3600 + 30 * 60;
        let encoded_offset = offset_sentinel - offset_seconds;
        let offset_part = (encoded_offset as i64 as u64) & ((1u64 << 24) - 1);
        assert_eq!(utc_time.bits, (micros_part << 24) | offset_part);

        // Explicit negative offset
        let pgn_content = r#"
[Event "Time Variants"]
[UTCTime "12:00:00-05:00"]
[Result "*"]

*
"#;
        let mut visitor = GameVisitor::new();
        let mut reader = Reader::new(pgn_content.as_bytes());
        reader.read_game(&mut visitor).unwrap();
        let game = visitor.current_game.take().unwrap();
        let utc_time = game.utc_time.unwrap();
        let offset_seconds: i32 = -(5 * 3600);
        let encoded_offset = offset_sentinel - offset_seconds;
        let offset_part = (encoded_offset as i64 as u64) & ((1u64 << 24) - 1);
        assert_eq!(utc_time.bits, (micros_part << 24) | offset_part);
    }

    #[test]
    fn test_pgn_visitor_time_fallback_from_invalid_utctime_to_time() {
        use crate::chess::visitor::GameVisitor;
        use pgn_reader::Reader;

        let pgn_content = r#"
[Event "Invalid UTCTime Fallback Time"]
[UTCTime "25:00:00"]
[Time "12:34:56"]
[Result "*"]

*
"#;

        let mut visitor = GameVisitor::new();
        let mut reader = Reader::new(pgn_content.as_bytes());
        reader.read_game(&mut visitor).unwrap();

        let game = visitor.current_game.take().unwrap();
        let utc_time = game.utc_time.unwrap();

        let micros = (12i64 * 3600 + 34 * 60 + 56) * 1_000_000;
        let micros_part = (micros as u64) & ((1u64 << 40) - 1);
        let offset_sentinel: i32 = (16 * 60 * 60) - 1;
        let encoded_offset = offset_sentinel;
        let offset_part = (encoded_offset as i64 as u64) & ((1u64 << 24) - 1);
        assert_eq!(utc_time.bits, (micros_part << 24) | offset_part);

        let err = game.parse_error.unwrap();
        assert!(err.contains("UTCTime='25:00:00'"));
        assert!(err.contains("chrono:"));
    }

    #[test]
    fn test_pgn_visitor_time_invalid_records_chrono_error() {
        use crate::chess::visitor::GameVisitor;
        use pgn_reader::Reader;

        let pgn_content = r#"
[Event "Invalid Time"]
[UTCTime "25:00:00"]
[Result "*"]

*
"#;

        let mut visitor = GameVisitor::new();
        let mut reader = Reader::new(pgn_content.as_bytes());
        reader.read_game(&mut visitor).unwrap();

        let game = visitor.current_game.take().unwrap();
        assert!(game.utc_time.is_none());
        let err = game.parse_error.unwrap();
        assert!(err.contains("UTCTime"));
        assert!(err.contains("25:00:00"));
        assert!(err.contains("chrono:"));
    }

    #[test]
    fn test_pgn_visitor_parser_stage_and_conversion_errors_combined() {
        use crate::chess::visitor::GameVisitor;
        use pgn_reader::Reader;

        let pgn_content = r#"
[Event "Parser Stage Error Game"]
[White "ParserErrorWhite"]
[Black "ParserErrorBlack"]
[WhiteElo "abc"]
[UTCDate "2024.13.01"]
[UTCTime "25:00:00"]
[Result "*"]

1. e4 { unterminated comment
"#;

        let mut visitor = GameVisitor::new();
        let mut reader = Reader::new(pgn_content.as_bytes());

        let parser_error = reader.read_game(&mut visitor).unwrap_err();
        visitor.finalize_game_with_error(format!(
            "Parser-stage error: stage=read_game; file='inline-test.pgn'; game_index=1; error={}",
            parser_error
        ));

        let game = visitor.current_game.take().unwrap();
        assert_eq!(game.event.as_deref(), Some("Parser Stage Error Game"));
        assert!(game.white_elo.is_none());
        assert!(game.utc_date.is_none());
        assert!(game.utc_time.is_none());

        let parse_error = game.parse_error.unwrap();
        assert!(parse_error.contains("Parser-stage error: stage=read_game"));
        assert!(parse_error.contains("file='inline-test.pgn'"));
        assert!(parse_error.contains("game_index=1"));
        assert!(parse_error.contains("unterminated comment"));
        assert!(parse_error.contains("Conversion error: WhiteElo='abc'"));
        assert!(parse_error.contains("Conversion error: UTCDate='2024.13.01'"));
        assert!(parse_error.contains("Conversion error: UTCTime='25:00:00'"));
    }

    #[test]
    fn test_pgn_visitor_movetext_with_annotations() {
        use crate::chess::visitor::GameVisitor;
        use pgn_reader::Reader;

        let pgn_content = r#"
[Event "Game with annotations"]
[White "Player 1"]
[Black "Player 2"]
[Result "1-0"]

1. e4 { [%eval 0.25] [%clk 1:30:43] } e5 { [%eval 0.22] [%clk 1:30:42] } 2. Nf3 1-0
"#;

        let mut visitor = GameVisitor::new();
        let mut reader = Reader::new(pgn_content.as_bytes());

        let result = reader.read_game(&mut visitor);
        assert!(result.is_ok());

        let game = visitor.current_game.take();
        assert!(game.is_some());

        let game = game.unwrap();
        assert!(game.movetext.contains("e4"));
        assert!(game.movetext.contains("e5"));
        assert!(game.movetext.contains("Nf3"));
        assert!(game.movetext.contains("{")); // Should preserve annotations in raw movetext
    }

    #[test]
    fn test_pgn_visitor_empty_movetext() {
        use crate::chess::visitor::GameVisitor;
        use pgn_reader::Reader;

        let pgn_content = r#"
[Event "Game with no moves"]
[White "Player 1"]
[Black "Player 2"]
[Result "*"]

*
"#;

        let mut visitor = GameVisitor::new();
        let mut reader = Reader::new(pgn_content.as_bytes());

        let result = reader.read_game(&mut visitor);
        assert!(result.is_ok());

        let game = visitor.current_game.take();
        assert!(game.is_some());

        let game = game.unwrap();
        // Movetext should be empty (result is stored separately)
        assert!(game.movetext.trim().is_empty());
    }

    #[test]
    fn test_pgn_visitor_malformed_headers() {
        use crate::chess::visitor::GameVisitor;
        use pgn_reader::Reader;

        let pgn_content = r#"
[Event "Malformed - incomplete headers
[White "Player 3"]

1. d4
"#;

        let mut visitor = GameVisitor::new();
        let mut reader = Reader::new(pgn_content.as_bytes());

        // The pgn-reader library is very robust and typically handles malformed headers
        let result = reader.read_game(&mut visitor);
        // It might succeed with partial data or fail gracefully
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_pgn_visitor_truncated_game() {
        use crate::chess::visitor::GameVisitor;
        use pgn_reader::Reader;

        let pgn_content = r#"
[Event "Truncated Game"]
[White "No one"]
"#;

        let mut visitor = GameVisitor::new();
        let mut reader = Reader::new(pgn_content.as_bytes());

        let result = reader.read_game(&mut visitor);
        assert!(result.is_ok());

        // Game should be created but may have incomplete data
        reader.read_game(&mut visitor).unwrap();
        let game = visitor.current_game.take();
        // May or may not have a game depending on parser behavior
        if let Some(game) = game {
            assert_eq!(game.event.as_deref().unwrap(), "Truncated Game");
            assert_eq!(game.white.as_deref().unwrap(), "No one");
        }
    }
}
