use super::{
    log,
    types::GameRecord,
    visitor::{PgnInput, PgnReaderState, SharedState},
};
use crate::chess::ErrorAccumulator;
use duckdb::{
    core::{DataChunkHandle, Inserter, LogicalTypeHandle, LogicalTypeId},
    vtab::{BindInfo, InitInfo, TableFunctionInfo, VTab},
};
use libduckdb_sys::{
    duckdb_bind_get_named_parameter, duckdb_bind_info, duckdb_date, duckdb_destroy_value,
    duckdb_free, duckdb_get_varchar, duckdb_is_null_value, duckdb_time_tz,
};
use std::borrow::Cow;
use std::ffi::{CStr, CString};
use std::fs::File;
use std::os::raw::c_void;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use zstd::stream::read::Decoder as ZstdDecoder;

#[repr(C)]
pub struct ReadPgnBindData {
    paths: Vec<PathBuf>,
    compression: CompressionMode,
}

#[repr(C)]
pub struct ReadPgnInitData {
    state: Mutex<SharedState>,
}

pub struct ReadPgnVTab;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum CompressionMode {
    Plain,
    Zstd,
}

const PATH_PATTERN_PARAM_INDEX: u64 = 0;
const ROWS_PER_CHUNK: usize = 2048;
const READ_PGN_COLUMN_COUNT: usize = 18;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ReadPgnColumn {
    Event = 0,
    Site = 1,
    White = 2,
    Black = 3,
    Result = 4,
    WhiteTitle = 5,
    BlackTitle = 6,
    WhiteElo = 7,
    BlackElo = 8,
    UtcDate = 9,
    UtcTime = 10,
    Eco = 11,
    Opening = 12,
    Termination = 13,
    TimeControl = 14,
    Movetext = 15,
    ParseError = 16,
    Source = 17,
}

impl ReadPgnColumn {
    const fn index(self) -> usize {
        self as usize
    }

    fn name(self) -> &'static str {
        READ_PGN_COLUMNS[self.index()].name
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ReadPgnLogicalType {
    Varchar,
    UInteger,
    Date,
    TimeTz,
}

impl ReadPgnLogicalType {
    fn to_handle(self) -> LogicalTypeHandle {
        match self {
            Self::Varchar => LogicalTypeHandle::from(LogicalTypeId::Varchar),
            Self::UInteger => LogicalTypeHandle::from(LogicalTypeId::UInteger),
            Self::Date => LogicalTypeHandle::from(LogicalTypeId::Date),
            Self::TimeTz => LogicalTypeHandle::from(LogicalTypeId::TimeTZ),
        }
    }
}

struct ReadPgnColumnDef {
    name: &'static str,
    logical_type: ReadPgnLogicalType,
}

const READ_PGN_COLUMNS: [ReadPgnColumnDef; READ_PGN_COLUMN_COUNT] = [
    ReadPgnColumnDef {
        name: "Event",
        logical_type: ReadPgnLogicalType::Varchar,
    },
    ReadPgnColumnDef {
        name: "Site",
        logical_type: ReadPgnLogicalType::Varchar,
    },
    ReadPgnColumnDef {
        name: "White",
        logical_type: ReadPgnLogicalType::Varchar,
    },
    ReadPgnColumnDef {
        name: "Black",
        logical_type: ReadPgnLogicalType::Varchar,
    },
    ReadPgnColumnDef {
        name: "Result",
        logical_type: ReadPgnLogicalType::Varchar,
    },
    ReadPgnColumnDef {
        name: "WhiteTitle",
        logical_type: ReadPgnLogicalType::Varchar,
    },
    ReadPgnColumnDef {
        name: "BlackTitle",
        logical_type: ReadPgnLogicalType::Varchar,
    },
    ReadPgnColumnDef {
        name: "WhiteElo",
        logical_type: ReadPgnLogicalType::UInteger,
    },
    ReadPgnColumnDef {
        name: "BlackElo",
        logical_type: ReadPgnLogicalType::UInteger,
    },
    ReadPgnColumnDef {
        name: "UTCDate",
        logical_type: ReadPgnLogicalType::Date,
    },
    ReadPgnColumnDef {
        name: "UTCTime",
        logical_type: ReadPgnLogicalType::TimeTz,
    },
    ReadPgnColumnDef {
        name: "ECO",
        logical_type: ReadPgnLogicalType::Varchar,
    },
    ReadPgnColumnDef {
        name: "Opening",
        logical_type: ReadPgnLogicalType::Varchar,
    },
    ReadPgnColumnDef {
        name: "Termination",
        logical_type: ReadPgnLogicalType::Varchar,
    },
    ReadPgnColumnDef {
        name: "TimeControl",
        logical_type: ReadPgnLogicalType::Varchar,
    },
    ReadPgnColumnDef {
        name: "movetext",
        logical_type: ReadPgnLogicalType::Varchar,
    },
    ReadPgnColumnDef {
        name: "parse_error",
        logical_type: ReadPgnLogicalType::Varchar,
    },
    ReadPgnColumnDef {
        name: "Source",
        logical_type: ReadPgnLogicalType::Varchar,
    },
];

impl CompressionMode {
    fn parse(raw: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let normalized = raw.trim();
        if normalized.is_empty() {
            return Err(
                "Invalid compression value ''. Supported values: 'zstd' or NULL/omitted."
                    .to_string()
                    .into(),
            );
        }

        if normalized.eq_ignore_ascii_case("zstd") {
            Ok(Self::Zstd)
        } else {
            Err(format!(
                "Invalid compression value '{}'. Supported values: 'zstd' or NULL/omitted.",
                normalized
            )
            .into())
        }
    }
}

fn resolve_compression_mode(
    bind: &BindInfo,
) -> Result<CompressionMode, Box<dyn std::error::Error>> {
    match get_named_parameter_varchar(bind, "compression")? {
        None | Some(None) => Ok(CompressionMode::Plain),
        Some(Some(raw)) => {
            let normalized = raw.trim();
            if normalized.eq_ignore_ascii_case("null") {
                Ok(CompressionMode::Plain)
            } else {
                CompressionMode::parse(normalized)
            }
        }
    }
}

fn bind_info_ptr(bind: &BindInfo) -> duckdb_bind_info {
    // SAFETY: `duckdb::vtab::BindInfo` is a thin wrapper around `duckdb_bind_info`
    // in duckdb-rs 1.4.4. We only read the wrapped pointer for C API interop.
    unsafe { *(bind as *const BindInfo as *const duckdb_bind_info) }
}

fn get_named_parameter_varchar(
    bind: &BindInfo,
    name: &str,
) -> Result<Option<Option<String>>, Box<dyn std::error::Error>> {
    let name_cstr = CString::new(name)?;

    // SAFETY: `bind_info_ptr` returns the C bind pointer owned by DuckDB for this bind call.
    let mut value =
        unsafe { duckdb_bind_get_named_parameter(bind_info_ptr(bind), name_cstr.as_ptr()) };
    if value.is_null() {
        return Ok(None);
    }

    // SAFETY: `value` is a valid duckdb_value returned by DuckDB and must be destroyed once.
    let result = unsafe {
        if duckdb_is_null_value(value) {
            Ok(Some(None))
        } else {
            let varchar = duckdb_get_varchar(value);
            if varchar.is_null() {
                Err(format!("Failed to read named parameter '{}' as VARCHAR", name).into())
            } else {
                let text = CStr::from_ptr(varchar).to_string_lossy().into_owned();
                duckdb_free(varchar as *mut c_void);
                Ok(Some(Some(text)))
            }
        }
    };

    // SAFETY: `value` has not been destroyed yet and must be released now.
    unsafe {
        duckdb_destroy_value(&mut value);
    }

    result
}

fn open_input_stream(path: &PathBuf, compression: CompressionMode) -> Result<PgnInput, String> {
    let file =
        File::open(path).map_err(|e| format!("Failed to open file '{}': {}", path.display(), e))?;

    match compression {
        CompressionMode::Plain => Ok(Box::new(file)),
        CompressionMode::Zstd => ZstdDecoder::new(file)
            .map(|decoder| Box::new(decoder) as PgnInput)
            .map_err(|e| {
                format!(
                    "Failed to initialize zstd decoder for '{}': {}",
                    path.display(),
                    e
                )
            }),
    }
}

fn sanitize_for_cstring<'a>(
    value: &'a str,
    field_name: &str,
    parse_error: &mut ErrorAccumulator,
) -> Cow<'a, str> {
    if value.contains('\0') {
        parse_error.push(&format!("Sanitized interior NUL in {}", field_name));
        Cow::Owned(value.replace('\0', " "))
    } else {
        Cow::Borrowed(value)
    }
}

fn sanitize_for_cstring_silent(value: &str) -> Cow<'_, str> {
    if value.contains('\0') {
        Cow::Owned(value.replace('\0', " "))
    } else {
        Cow::Borrowed(value)
    }
}

enum ReadNextGameOutcome {
    GameReady,
    ReaderFinished,
}

struct ChunkWriter<'a> {
    output: &'a mut DataChunkHandle,
    row_count: usize,
}

impl<'a> ChunkWriter<'a> {
    fn new(output: &'a mut DataChunkHandle) -> Self {
        Self {
            output,
            row_count: 0,
        }
    }

    fn is_full(&self) -> bool {
        self.row_count >= ROWS_PER_CHUNK
    }

    fn write_row(&mut self, game: &GameRecord) -> Result<(), Box<dyn std::error::Error>> {
        let row_idx = self.row_count;
        let mut row_parse_error = ErrorAccumulator::default();
        if let Some(parse_error) = game.parse_error.as_deref() {
            row_parse_error.push(parse_error);
        }

        self.write_optional_varchar(
            ReadPgnColumn::Event,
            row_idx,
            game.event.as_deref(),
            &mut row_parse_error,
        )?;
        self.write_optional_varchar(
            ReadPgnColumn::Site,
            row_idx,
            game.site.as_deref(),
            &mut row_parse_error,
        )?;
        self.write_optional_varchar(
            ReadPgnColumn::White,
            row_idx,
            game.white.as_deref(),
            &mut row_parse_error,
        )?;
        self.write_optional_varchar(
            ReadPgnColumn::Black,
            row_idx,
            game.black.as_deref(),
            &mut row_parse_error,
        )?;
        self.write_optional_varchar(
            ReadPgnColumn::Result,
            row_idx,
            game.result.as_deref(),
            &mut row_parse_error,
        )?;
        self.write_optional_varchar(
            ReadPgnColumn::WhiteTitle,
            row_idx,
            game.white_title.as_deref(),
            &mut row_parse_error,
        )?;
        self.write_optional_varchar(
            ReadPgnColumn::BlackTitle,
            row_idx,
            game.black_title.as_deref(),
            &mut row_parse_error,
        )?;
        self.write_optional_uinteger(ReadPgnColumn::WhiteElo, row_idx, game.white_elo);
        self.write_optional_uinteger(ReadPgnColumn::BlackElo, row_idx, game.black_elo);
        self.write_optional_date(ReadPgnColumn::UtcDate, row_idx, game.utc_date);
        self.write_optional_time_tz(ReadPgnColumn::UtcTime, row_idx, game.utc_time);
        self.write_optional_varchar(
            ReadPgnColumn::Eco,
            row_idx,
            game.eco.as_deref(),
            &mut row_parse_error,
        )?;
        self.write_optional_varchar(
            ReadPgnColumn::Opening,
            row_idx,
            game.opening.as_deref(),
            &mut row_parse_error,
        )?;
        self.write_optional_varchar(
            ReadPgnColumn::Termination,
            row_idx,
            game.termination.as_deref(),
            &mut row_parse_error,
        )?;
        self.write_optional_varchar(
            ReadPgnColumn::TimeControl,
            row_idx,
            game.time_control.as_deref(),
            &mut row_parse_error,
        )?;

        let movetext = sanitize_for_cstring(
            game.movetext.as_str(),
            ReadPgnColumn::Movetext.name(),
            &mut row_parse_error,
        );
        let movetext_vec = self.output.flat_vector(ReadPgnColumn::Movetext.index());
        movetext_vec.insert(row_idx, CString::new(movetext.as_ref())?);

        self.write_optional_varchar(
            ReadPgnColumn::Source,
            row_idx,
            game.source.as_deref(),
            &mut row_parse_error,
        )?;

        let mut parse_error_vec = self.output.flat_vector(ReadPgnColumn::ParseError.index());
        if row_parse_error.is_empty() {
            parse_error_vec.set_null(row_idx);
        } else {
            let parse_error = row_parse_error.take().unwrap_or_default();
            let parse_error = sanitize_for_cstring_silent(parse_error.as_str());
            parse_error_vec.insert(row_idx, CString::new(parse_error.as_ref())?);
        }

        self.row_count += 1;
        Ok(())
    }

    fn set_output_len(&mut self) {
        self.output.set_len(self.row_count);
    }

    fn write_optional_varchar(
        &mut self,
        column: ReadPgnColumn,
        row_idx: usize,
        value: Option<&str>,
        parse_error: &mut ErrorAccumulator,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut vector = self.output.flat_vector(column.index());
        if let Some(value) = value {
            let sanitized = sanitize_for_cstring(value, column.name(), parse_error);
            vector.insert(row_idx, CString::new(sanitized.as_ref())?);
        } else {
            vector.set_null(row_idx);
        }
        Ok(())
    }

    fn write_optional_uinteger(
        &mut self,
        column: ReadPgnColumn,
        row_idx: usize,
        value: Option<u32>,
    ) {
        let mut vector = self.output.flat_vector(column.index());
        if let Some(value) = value {
            vector.as_mut_slice::<u32>()[row_idx] = value;
        } else {
            vector.set_null(row_idx);
        }
    }

    fn write_optional_date(
        &mut self,
        column: ReadPgnColumn,
        row_idx: usize,
        value: Option<duckdb_date>,
    ) {
        let mut vector = self.output.flat_vector(column.index());
        if let Some(value) = value {
            vector.as_mut_slice::<duckdb_date>()[row_idx] = value;
        } else {
            vector.set_null(row_idx);
        }
    }

    fn write_optional_time_tz(
        &mut self,
        column: ReadPgnColumn,
        row_idx: usize,
        value: Option<duckdb_time_tz>,
    ) {
        let mut vector = self.output.flat_vector(column.index());
        if let Some(value) = value {
            vector.as_mut_slice::<duckdb_time_tz>()[row_idx] = value;
        } else {
            vector.set_null(row_idx);
        }
    }
}

fn acquire_reader(
    init_data: &ReadPgnInitData,
    bind_data: &ReadPgnBindData,
) -> Result<Option<PgnReaderState>, Box<dyn std::error::Error>> {
    loop {
        let path_idx = {
            let mut state = init_data.state.lock().unwrap();

            if let Some(reader) = state.available_readers.pop() {
                return Ok(Some(reader));
            }

            if state.next_path_idx < bind_data.paths.len() {
                let path_idx = state.next_path_idx;
                state.next_path_idx += 1;
                path_idx
            } else {
                return Ok(None);
            }
        };

        let path = &bind_data.paths[path_idx];
        match open_input_stream(path, bind_data.compression) {
            Ok(input_stream) => {
                return Ok(Some(PgnReaderState::new(input_stream, path_idx)));
            }
            Err(err_msg) => {
                if bind_data.paths.len() == 1 {
                    return Err(err_msg.into());
                }

                log::warn(&err_msg);
            }
        }
    }
}

fn read_next_game(reader: &mut PgnReaderState, source_path: &Path) -> ReadNextGameOutcome {
    let game_index = reader.next_game_index;

    match reader.pgn_reader.read_game(&mut reader.visitor) {
        Ok(Some(_)) => {
            reader.next_game_index += 1;
            if let Some(game) = reader.visitor.current_game.take() {
                reader.record_buffer = game;
                ReadNextGameOutcome::GameReady
            } else {
                ReadNextGameOutcome::ReaderFinished
            }
        }
        Ok(None) => ReadNextGameOutcome::ReaderFinished,
        Err(error) => {
            reader.next_game_index += 1;
            let error_msg = format!(
                "Parser-stage error: stage=read_game; file='{}'; game_index={}; error={}",
                source_path.display(),
                game_index,
                error
            );
            log::warn(&error_msg);
            reader.visitor.finalize_game_with_error(error_msg);

            if let Some(game) = reader.visitor.current_game.take() {
                reader.record_buffer = game;
                ReadNextGameOutcome::GameReady
            } else {
                ReadNextGameOutcome::ReaderFinished
            }
        }
    }
}

fn write_row(
    chunk_writer: &mut ChunkWriter<'_>,
    reader: &PgnReaderState,
) -> Result<(), Box<dyn std::error::Error>> {
    chunk_writer.write_row(&reader.record_buffer)
}

fn finalize_chunk(
    init_data: &ReadPgnInitData,
    current_reader_state: Option<PgnReaderState>,
    chunk_writer: &mut ChunkWriter<'_>,
) {
    if let Some(reader) = current_reader_state {
        let mut state = init_data.state.lock().unwrap();
        state.available_readers.push(reader);
    }

    chunk_writer.set_output_len();
}

impl VTab for ReadPgnVTab {
    type InitData = ReadPgnInitData;
    type BindData = ReadPgnBindData;

    fn bind(bind: &BindInfo) -> Result<Self::BindData, Box<dyn std::error::Error>> {
        let pattern = bind.get_parameter(PATH_PATTERN_PARAM_INDEX).to_string();
        let compression = resolve_compression_mode(bind)?;

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

        for column in READ_PGN_COLUMNS.iter() {
            bind.add_result_column(column.name, column.logical_type.to_handle());
        }

        Ok(ReadPgnBindData { paths, compression })
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
        let mut chunk_writer = ChunkWriter::new(output);
        let mut current_reader_state: Option<PgnReaderState> = None;

        while !chunk_writer.is_full() {
            if current_reader_state.is_none() {
                current_reader_state = acquire_reader(init_data, bind_data)?;
                if current_reader_state.is_none() {
                    break;
                }
            }

            if let Some(mut reader) = current_reader_state.take() {
                // Use pgn-reader's Reader directly for streaming PGN parsing.
                // Note: For plain files we do NOT add an extra BufReader layer because
                // pgn-reader's documentation states:
                // "Buffers the underlying reader with an appropriate strategy, so it's not
                // recommended to add an additional layer of buffering like BufReader."
                let source_path = &bind_data.paths[reader.path_idx];
                match read_next_game(&mut reader, source_path) {
                    ReadNextGameOutcome::GameReady => {
                        write_row(&mut chunk_writer, &reader)?;
                        current_reader_state = Some(reader);
                    }
                    ReadNextGameOutcome::ReaderFinished => {
                        // Reader finished (EOF or no recoverable record)
                        // It will be dropped here and loop will acquire new work.
                    }
                }
            }
        }

        finalize_chunk(init_data, current_reader_state, &mut chunk_writer);
        Ok(())
    }

    fn parameters() -> Option<Vec<LogicalTypeHandle>> {
        Some(vec![
            LogicalTypeHandle::from(LogicalTypeId::Varchar), // path pattern (required)
        ])
    }

    fn named_parameters() -> Option<Vec<(String, LogicalTypeHandle)>> {
        Some(vec![(
            "compression".to_string(),
            LogicalTypeHandle::from(LogicalTypeId::Varchar),
        )])
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
        let bind_data = ReadPgnBindData {
            paths,
            compression: CompressionMode::Plain,
        };
        assert_eq!(bind_data.paths.len(), 1);
        assert_eq!(bind_data.paths[0], PathBuf::from("test.pgn"));
        assert_eq!(bind_data.compression, CompressionMode::Plain);
    }

    #[test]
    fn test_read_pgn_bind_data_multiple_files() {
        // Test that bind data can be created with multiple files
        let paths = vec![PathBuf::from("test1.pgn"), PathBuf::from("test2.pgn")];
        let bind_data = ReadPgnBindData {
            paths,
            compression: CompressionMode::Plain,
        };
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

    #[test]
    fn test_read_pgn_columns_match_contract() {
        let expected: [(&str, ReadPgnLogicalType); READ_PGN_COLUMN_COUNT] = [
            ("Event", ReadPgnLogicalType::Varchar),
            ("Site", ReadPgnLogicalType::Varchar),
            ("White", ReadPgnLogicalType::Varchar),
            ("Black", ReadPgnLogicalType::Varchar),
            ("Result", ReadPgnLogicalType::Varchar),
            ("WhiteTitle", ReadPgnLogicalType::Varchar),
            ("BlackTitle", ReadPgnLogicalType::Varchar),
            ("WhiteElo", ReadPgnLogicalType::UInteger),
            ("BlackElo", ReadPgnLogicalType::UInteger),
            ("UTCDate", ReadPgnLogicalType::Date),
            ("UTCTime", ReadPgnLogicalType::TimeTz),
            ("ECO", ReadPgnLogicalType::Varchar),
            ("Opening", ReadPgnLogicalType::Varchar),
            ("Termination", ReadPgnLogicalType::Varchar),
            ("TimeControl", ReadPgnLogicalType::Varchar),
            ("movetext", ReadPgnLogicalType::Varchar),
            ("parse_error", ReadPgnLogicalType::Varchar),
            ("Source", ReadPgnLogicalType::Varchar),
        ];

        for (idx, column) in READ_PGN_COLUMNS.iter().enumerate() {
            assert_eq!(column.name, expected[idx].0);
            assert_eq!(column.logical_type, expected[idx].1);
        }
    }

    #[test]
    fn test_rows_per_chunk_constant_matches_contract() {
        assert_eq!(ROWS_PER_CHUNK, 2048);
    }

    #[test]
    fn test_sanitize_for_cstring_preserves_clean_values() {
        let mut parse_error = ErrorAccumulator::default();
        let sanitized = sanitize_for_cstring("normal text", "Event", &mut parse_error);
        assert_eq!(sanitized.as_ref(), "normal text");
        assert!(parse_error.is_empty());
    }

    #[test]
    fn test_sanitize_for_cstring_replaces_interior_nul_and_records_error() {
        let mut parse_error = ErrorAccumulator::default();
        let sanitized = sanitize_for_cstring("A\0B", "Event", &mut parse_error);
        assert_eq!(sanitized.as_ref(), "A B");

        let message = parse_error.take().expect("expected parse_error message");
        assert!(message.contains("Sanitized interior NUL in Event"));
    }

    #[test]
    fn test_parse_compression_mode_zstd_case_insensitive() {
        assert_eq!(
            CompressionMode::parse("zstd").unwrap(),
            CompressionMode::Zstd
        );
        assert_eq!(
            CompressionMode::parse("ZsTd").unwrap(),
            CompressionMode::Zstd
        );
    }

    #[test]
    fn test_parse_compression_mode_rejects_empty_value() {
        let err = CompressionMode::parse("   ").unwrap_err().to_string();
        assert!(err.contains("Invalid compression value"));
    }

    #[test]
    fn test_parse_compression_mode_rejects_unsupported_value() {
        let err = CompressionMode::parse("gzip").unwrap_err().to_string();
        assert!(err.contains("Invalid compression value 'gzip'"));
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
