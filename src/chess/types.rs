/// Stores parsed game data from PGN - matches Lichess dataset schema
use libduckdb_sys::{duckdb_date, duckdb_time_tz};

#[derive(Debug, Clone, Default)]
pub struct GameRecord {
    // Core game info
    pub event: Option<String>,
    pub site: Option<String>,
    pub source: Option<String>,
    pub white: Option<String>,
    pub black: Option<String>,
    pub result: Option<String>,

    // Player info
    pub white_title: Option<String>,
    pub black_title: Option<String>,
    pub white_elo: Option<u32>,
    pub black_elo: Option<u32>,

    // Date/Time
    pub utc_date: Option<duckdb_date>,
    pub utc_time: Option<duckdb_time_tz>,

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
