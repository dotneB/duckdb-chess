use super::types::GameRecord;
#[cfg(not(test))]
use libduckdb_sys::duckdb_create_time_tz;
use libduckdb_sys::{duckdb_date, duckdb_time_tz};

use chrono::{Datelike, NaiveDate, NaiveTime, Timelike};

#[cfg(not(test))]
#[inline]
fn create_time_tz(micros: i64, offset_seconds: i32) -> duckdb_time_tz {
    // SAFETY: Only called inside DuckDB (API initialized).
    unsafe { duckdb_create_time_tz(micros, offset_seconds) }
}
use pgn_reader::{Outcome, RawComment, RawTag, Reader, SanPlus, Skip, Visitor};
use std::fs::File;
use std::ops::ControlFlow;

/// Streaming PGN visitor (pgn-reader).
/// Spec: pgn-parsing - Visitor Pattern Implementation
///
/// Accumulates mainline movetext into a `String`, includes `{ ... }` comments
/// (whitespace-normalized). Result is captured separately via `outcome()` (or
/// the `Result` tag as fallback).
pub struct GameVisitor {
    headers: Vec<(String, String)>,
    movetext_buffer: String,
    move_count: u32,
    result_marker: Option<String>,
    pub current_game: Option<GameRecord>,
}

impl GameVisitor {
    pub fn new() -> Self {
        Self {
            headers: Vec::new(),
            movetext_buffer: String::new(),
            move_count: 0,
            result_marker: None,
            current_game: None,
        }
    }

    fn get_header(&self, key: &str) -> Option<String> {
        self.headers
            .iter()
            .find(|(k, _)| k == key)
            .map(|(_, v)| v.clone())
    }

    fn push_error(parse_error: &mut Option<String>, msg: String) {
        match parse_error {
            Some(existing) => {
                existing.push_str("; ");
                existing.push_str(&msg);
            }
            None => {
                *parse_error = Some(msg);
            }
        }
    }

    fn normalize_date_separators(s: &str) -> String {
        let s = s.trim();
        if s.contains('.') {
            s.replace('.', "-")
        } else {
            s.to_string()
        }
    }

    fn date_completeness_score(raw: &str) -> u8 {
        let s = raw.trim();
        if s.is_empty() {
            return 0;
        }

        let norm = Self::normalize_date_separators(s);
        let parts: Vec<&str> = norm.split('-').collect();
        if parts.len() != 3 {
            return 0;
        }

        let year_known = !parts[0].contains('?') && parts[0].parse::<i32>().ok().is_some();
        if !year_known {
            return 0;
        }

        let mut score = 1u8;
        let month_known = !parts[1].contains('?') && parts[1].parse::<u32>().ok().is_some();
        let day_known = !parts[2].contains('?') && parts[2].parse::<u32>().ok().is_some();
        if month_known {
            score += 1;
        }
        if day_known {
            score += 1;
        }
        score
    }

    fn rank_date_candidates(
        utc_date: Option<String>,
        date: Option<String>,
        event_date: Option<String>,
    ) -> Vec<(String, &'static str)> {
        // Rank by completeness first; if tied, prefer header precedence:
        // UTCDate -> Date -> EventDate
        let mut ranked: Vec<(u8, u8, String, &'static str)> = Vec::new();

        for (precedence, raw, label) in [
            (0u8, utc_date, "UTCDate"),
            (1u8, date, "UTCDate (from Date)"),
            (2u8, event_date, "UTCDate (from EventDate)"),
        ] {
            let Some(raw) = raw else { continue };
            let s = raw.trim();
            if s.is_empty() {
                continue;
            }

            let score = Self::date_completeness_score(s);
            ranked.push((score, precedence, raw, label));
        }

        ranked.sort_by(|left, right| right.0.cmp(&left.0).then_with(|| left.1.cmp(&right.1)));

        ranked
            .into_iter()
            .map(|(_, _, raw, label)| (raw, label))
            .collect()
    }

    fn parse_best_date_field(
        utc_date: Option<String>,
        date: Option<String>,
        event_date: Option<String>,
        parse_error: &mut Option<String>,
    ) -> Option<duckdb_date> {
        for (raw, label) in Self::rank_date_candidates(utc_date, date, event_date) {
            if let Some(parsed) = Self::parse_date_field(raw, label, parse_error) {
                return Some(parsed);
            }
        }

        None
    }

    fn parse_date_field(
        raw: String,
        label: &str,
        parse_error: &mut Option<String>,
    ) -> Option<duckdb_date> {
        let s = raw.trim();
        if s.is_empty() {
            return None;
        }

        let norm = Self::normalize_date_separators(s);
        let parts: Vec<&str> = norm.split('-').collect();
        if parts.len() != 3 {
            match NaiveDate::parse_from_str(&norm, "%Y-%m-%d") {
                Ok(_) => {
                    // Should not happen if split failed, but keep a consistent error message.
                    Self::push_error(parse_error, format!("Conversion error: {label}='{s}'"));
                }
                Err(e) => {
                    Self::push_error(
                        parse_error,
                        format!("Conversion error: {label}='{s}' (chrono: {e})"),
                    );
                }
            }
            return None;
        }

        // Unknown year => unknown date (NULL) without a conversion error.
        if parts[0].contains('?') {
            return None;
        }

        let year_s = parts[0];
        let month_s = if parts[1].contains('?') {
            "01"
        } else {
            parts[1]
        };
        let day_s = if parts[2].contains('?') {
            "01"
        } else {
            parts[2]
        };

        let full = format!("{year_s}-{month_s}-{day_s}");

        let date = match NaiveDate::parse_from_str(&full, "%Y-%m-%d") {
            Ok(v) => v,
            Err(e) => {
                Self::push_error(
                    parse_error,
                    format!("Conversion error: {label}='{s}' (chrono: {e})"),
                );
                return None;
            }
        };

        if date.year() <= 0 {
            Self::push_error(
                parse_error,
                format!("Conversion error: {label}='{s}' (chrono: year must be >= 1)"),
            );
            return None;
        }

        let epoch = match NaiveDate::from_ymd_opt(1970, 1, 1) {
            Some(v) => v,
            None => {
                // Should never happen.
                Self::push_error(
                    parse_error,
                    "Conversion error: failed to create epoch".into(),
                );
                return None;
            }
        };

        let days_i64 = date.signed_duration_since(epoch).num_days();
        let days: i32 = match i32::try_from(days_i64) {
            Ok(v) => v,
            Err(_) => {
                Self::push_error(
                    parse_error,
                    format!("Conversion error: {label}='{s}' (chrono: date out of range)"),
                );
                return None;
            }
        };

        Some(duckdb_date { days })
    }

    fn parse_uinteger_field(
        raw: Option<String>,
        label: &str,
        parse_error: &mut Option<String>,
    ) -> Option<u32> {
        let raw = raw?;
        let s = raw.trim();
        if s.is_empty() {
            return None;
        }
        match s.parse::<u32>() {
            Ok(v) => Some(v),
            Err(_) => {
                Self::push_error(parse_error, format!("Conversion error: {label}='{s}'"));
                None
            }
        }
    }

    fn parse_time_tz_field(
        raw: String,
        label: &str,
        parse_error: &mut Option<String>,
    ) -> Option<duckdb_time_tz> {
        let s = raw.trim();
        if s.is_empty() {
            return None;
        }

        // Formats supported:
        // - HH:MM:SS
        // - HH:MM:SSZ
        // - HH:MM:SS+HH:MM
        // - HH:MM:SS-HH:MM
        let (time_part, offset_seconds) = if let Some(stripped) = s.strip_suffix('Z') {
            (stripped, 0i32)
        } else if let Some((t, off)) = s.split_once('+') {
            (
                t,
                match Self::parse_tz_offset_seconds(off) {
                    Some(v) => v,
                    None => {
                        Self::push_error(parse_error, format!("Conversion error: {label}='{s}'"));
                        return None;
                    }
                },
            )
        } else if let Some((t, off)) = s.split_once('-') {
            (
                t,
                match Self::parse_tz_offset_seconds(off) {
                    Some(v) => -v,
                    None => {
                        Self::push_error(parse_error, format!("Conversion error: {label}='{s}'"));
                        return None;
                    }
                },
            )
        } else {
            (s, 0i32)
        };

        let time = match NaiveTime::parse_from_str(time_part, "%H:%M:%S") {
            Ok(v) => v,
            Err(e) => {
                Self::push_error(
                    parse_error,
                    format!("Conversion error: {label}='{s}' (chrono: {e})"),
                );
                return None;
            }
        };

        let micros = (time.num_seconds_from_midnight() as i64) * 1_000_000
            + (time.nanosecond() as i64) / 1_000;
        Some(Self::pack_time_tz(micros, offset_seconds))
    }

    fn parse_best_time_tz_field(
        utc_time: Option<String>,
        time: Option<String>,
        parse_error: &mut Option<String>,
    ) -> Option<duckdb_time_tz> {
        if let Some(raw) = utc_time
            && let Some(parsed) = Self::parse_time_tz_field(raw, "UTCTime", parse_error)
        {
            return Some(parsed);
        }

        if let Some(raw) = time
            && let Some(parsed) = Self::parse_time_tz_field(raw, "UTCTime (from Time)", parse_error)
        {
            return Some(parsed);
        }

        None
    }

    #[cfg(not(test))]
    fn pack_time_tz(micros: i64, offset_seconds: i32) -> duckdb_time_tz {
        create_time_tz(micros, offset_seconds)
    }

    #[cfg(test)]
    fn pack_time_tz(micros: i64, offset_seconds: i32) -> duckdb_time_tz {
        // Unit tests run without DuckDB initializing the C API.
        const OFFSET_SENTINEL_SECONDS: i32 = 16 * 60 * 60 - 1; // 15:59:59
        let encoded_offset = OFFSET_SENTINEL_SECONDS - offset_seconds;

        let micros_part = (micros as u64) & ((1u64 << 40) - 1);
        let offset_part = (encoded_offset as i64 as u64) & ((1u64 << 24) - 1);
        duckdb_time_tz {
            bits: (micros_part << 24) | offset_part,
        }
    }

    fn parse_tz_offset_seconds(s: &str) -> Option<i32> {
        let s = s.trim();
        let (hh, mm) = s.split_once(':')?;
        let hh: i32 = hh.parse().ok()?;
        let mm: i32 = mm.parse().ok()?;
        if !(0..=23).contains(&hh) || !(0..=59).contains(&mm) {
            return None;
        }
        Some(hh * 3600 + mm * 60)
    }

    fn finalize_game(&mut self) {
        let mut parse_error: Option<String> = None;

        let white_elo =
            Self::parse_uinteger_field(self.get_header("WhiteElo"), "WhiteElo", &mut parse_error);
        let black_elo =
            Self::parse_uinteger_field(self.get_header("BlackElo"), "BlackElo", &mut parse_error);

        let utc_date = Self::parse_best_date_field(
            self.get_header("UTCDate"),
            self.get_header("Date"),
            self.get_header("EventDate"),
            &mut parse_error,
        );
        let utc_time = Self::parse_best_time_tz_field(
            self.get_header("UTCTime"),
            self.get_header("Time"),
            &mut parse_error,
        );

        self.current_game = Some(GameRecord {
            event: self.get_header("Event"),
            site: self.get_header("Site"),
            source: self.get_header("Source"),
            white: self.get_header("White"),
            black: self.get_header("Black"),
            result: self
                .get_header("Result")
                .or_else(|| self.result_marker.clone()),
            white_title: self.get_header("WhiteTitle"),
            black_title: self.get_header("BlackTitle"),
            white_elo,
            black_elo,
            utc_date,
            utc_time,
            eco: self.get_header("ECO"),
            opening: self.get_header("Opening"),
            termination: self.get_header("Termination"),
            time_control: self.get_header("TimeControl"),
            movetext: self.movetext_buffer.trim().to_string(),
            parse_error,
        });
    }

    /// Spec: pgn-parsing - Error Message Capture
    pub fn finalize_game_with_error(&mut self, error_msg: String) {
        let mut parse_error: Option<String> = Some(error_msg);

        let white_elo =
            Self::parse_uinteger_field(self.get_header("WhiteElo"), "WhiteElo", &mut parse_error);
        let black_elo =
            Self::parse_uinteger_field(self.get_header("BlackElo"), "BlackElo", &mut parse_error);

        let utc_date = Self::parse_best_date_field(
            self.get_header("UTCDate"),
            self.get_header("Date"),
            self.get_header("EventDate"),
            &mut parse_error,
        );
        let utc_time = Self::parse_best_time_tz_field(
            self.get_header("UTCTime"),
            self.get_header("Time"),
            &mut parse_error,
        );

        self.current_game = Some(GameRecord {
            event: self.get_header("Event"),
            site: self.get_header("Site"),
            source: self.get_header("Source"),
            white: self.get_header("White"),
            black: self.get_header("Black"),
            result: self
                .get_header("Result")
                .or_else(|| self.result_marker.clone()),
            white_title: self.get_header("WhiteTitle"),
            black_title: self.get_header("BlackTitle"),
            white_elo,
            black_elo,
            utc_date,
            utc_time,
            eco: self.get_header("ECO"),
            opening: self.get_header("Opening"),
            termination: self.get_header("Termination"),
            time_control: self.get_header("TimeControl"),
            movetext: self.movetext_buffer.trim().to_string(),
            parse_error,
        });
    }
}

pub struct PgnReaderState {
    pub pgn_reader: Reader<File>,
    pub path_idx: usize,
    pub next_game_index: usize,
    pub record_buffer: GameRecord,
    pub visitor: GameVisitor,
}

impl PgnReaderState {
    pub fn new(file: File, path_idx: usize) -> Self {
        Self {
            pgn_reader: Reader::new(file),
            path_idx,
            next_game_index: 1,
            record_buffer: GameRecord::default(),
            visitor: GameVisitor::new(),
        }
    }
}

pub struct SharedState {
    pub next_path_idx: usize,
    pub available_readers: Vec<PgnReaderState>,
}

impl Visitor for GameVisitor {
    type Tags = ();
    type Movetext = String;
    type Output = ();

    fn begin_tags(&mut self) -> ControlFlow<Self::Output, Self::Tags> {
        self.headers.clear();
        self.movetext_buffer.clear();
        self.move_count = 0;
        self.result_marker = None;
        self.current_game = None;
        ControlFlow::Continue(())
    }

    fn tag(
        &mut self,
        _: &mut Self::Tags,
        key: &[u8],
        value: RawTag<'_>,
    ) -> ControlFlow<Self::Output> {
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
        if !movetext.is_empty() {
            movetext.push(' ');
        }

        if self.move_count.is_multiple_of(2) {
            movetext.push_str(&format!("{}. ", (self.move_count / 2) + 1));
        }

        movetext.push_str(&san.to_string());
        self.move_count += 1;
        ControlFlow::Continue(())
    }

    fn comment(
        &mut self,
        movetext: &mut Self::Movetext,
        comment: RawComment<'_>,
    ) -> ControlFlow<Self::Output> {
        let comment_str = String::from_utf8_lossy(comment.as_bytes());

        if !movetext.is_empty() {
            movetext.push(' ');
        }
        movetext.push_str(&format!("{{ {} }}", comment_str.trim()));

        ControlFlow::Continue(())
    }

    fn outcome(
        &mut self,
        _movetext: &mut Self::Movetext,
        outcome: Outcome,
    ) -> ControlFlow<Self::Output> {
        self.result_marker = Some(outcome.to_string());
        ControlFlow::Continue(())
    }

    fn end_game(&mut self, movetext: Self::Movetext) -> Self::Output {
        let marker = self
            .result_marker
            .clone()
            .or_else(|| self.get_header("Result"));
        self.result_marker = marker;

        self.movetext_buffer = movetext;
        self.finalize_game();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pgn_reader::Reader;

    #[test]
    fn test_visitor_basic_parsing() {
        let pgn = r#"[Event "Test Game"]
[Site "Internet"]
[Result "1-0"]
1. e4 e5 2. Nf3 1-0"#;

        let mut reader = Reader::new(pgn.as_bytes());
        let mut visitor = GameVisitor::new();

        reader.read_game(&mut visitor).unwrap();

        let game = visitor.current_game.expect("Should have parsed a game");
        assert_eq!(game.event.as_deref(), Some("Test Game"));
        assert_eq!(game.site.as_deref(), Some("Internet"));
        assert_eq!(game.result.as_deref(), Some("1-0"));
        assert_eq!(game.movetext, "1. e4 e5 2. Nf3");
    }

    #[test]
    fn test_visitor_with_comments() {
        let pgn = r#"[Event "Comment Test"]
1. e4 { best by test } e5 1-0"#;

        let mut reader = Reader::new(pgn.as_bytes());
        let mut visitor = GameVisitor::new();

        reader.read_game(&mut visitor).unwrap();

        let game = visitor.current_game.expect("Should have parsed a game");
        assert_eq!(game.movetext, "1. e4 { best by test } e5");
    }

    #[test]
    fn test_visitor_empty_movetext() {
        let pgn = r#"[Event "Empty"]
[Result "*"]
*"#;
        let mut reader = Reader::new(pgn.as_bytes());
        let mut visitor = GameVisitor::new();

        reader.read_game(&mut visitor).unwrap();

        let game = visitor.current_game.expect("Should have parsed a game");
        assert_eq!(game.movetext, "");
        assert_eq!(game.result.as_deref(), Some("*"));
    }

    #[test]
    fn test_visitor_numeric_fields() {
        let pgn = r#"[WhiteElo "2500"]
[BlackElo "2400"]
1. e4 1-0"#;

        let mut reader = Reader::new(pgn.as_bytes());
        let mut visitor = GameVisitor::new();

        reader.read_game(&mut visitor).unwrap();

        let game = visitor.current_game.expect("Should have parsed a game");
        assert_eq!(game.white_elo, Some(2500));
        assert_eq!(game.black_elo, Some(2400));
    }

    #[test]
    fn test_visitor_comment_before_first_move() {
        let pgn = r#"[Event "Comment Test"]
{ opening comment } 1. e4 e5"#;

        let mut reader = Reader::new(pgn.as_bytes());
        let mut visitor = GameVisitor::new();

        reader.read_game(&mut visitor).unwrap();

        let game = visitor.current_game.expect("Should have parsed a game");
        assert_eq!(game.movetext, "{ opening comment } 1. e4 e5");
    }

    #[test]
    fn test_visitor_multiple_comments() {
        let pgn = r#"[Event "Multiple Comments"]
1. e4 { first } e5 { second } 2. Nf3 { third }"#;

        let mut reader = Reader::new(pgn.as_bytes());
        let mut visitor = GameVisitor::new();

        reader.read_game(&mut visitor).unwrap();

        let game = visitor.current_game.expect("Should have parsed a game");
        assert_eq!(
            game.movetext,
            "1. e4 { first } e5 { second } 2. Nf3 { third }"
        );
    }

    #[test]
    fn test_visitor_lichess_annotations() {
        let pgn = r#"[Event "Lichess Annotations"]
1. d4 { [%eval 0.25] [%clk 1:30:43] } Nf6 { [%eval 0.22] [%clk 1:30:42] }"#;

        let mut reader = Reader::new(pgn.as_bytes());
        let mut visitor = GameVisitor::new();

        reader.read_game(&mut visitor).unwrap();

        let game = visitor.current_game.expect("Should have parsed a game");
        assert_eq!(
            game.movetext,
            "1. d4 { [%eval 0.25] [%clk 1:30:43] } Nf6 { [%eval 0.22] [%clk 1:30:42] }"
        );
    }
}
