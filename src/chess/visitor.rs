use super::types::GameRecord;
use crate::chess::ErrorAccumulator;
#[cfg(not(test))]
use libduckdb_sys::duckdb_create_time_tz;
use libduckdb_sys::{duckdb_date, duckdb_time_tz};

use chrono::{Datelike, NaiveDate, NaiveTime, Timelike};
use pgn_reader::{Outcome, RawComment, RawTag, Reader, SanPlus, Skip, Visitor};
use std::fmt::Write;
use std::io::Read;
use std::mem;
use std::ops::ControlFlow;
use std::sync::LazyLock;

static EPOCH: LazyLock<NaiveDate> = LazyLock::new(|| NaiveDate::from_ymd_opt(1970, 1, 1).unwrap());

#[macro_export]
macro_rules! pgn_visitor_skip_variations {
    () => {
        fn nag(&mut self, _: &mut Self::Movetext, _: Nag) -> ControlFlow<Self::Output> {
            ControlFlow::Continue(())
        }

        fn comment(
            &mut self,
            _: &mut Self::Movetext,
            _: RawComment<'_>,
        ) -> ControlFlow<Self::Output> {
            ControlFlow::Continue(())
        }

        fn partial_comment(
            &mut self,
            _: &mut Self::Movetext,
            _: RawComment<'_>,
        ) -> ControlFlow<Self::Output> {
            ControlFlow::Continue(())
        }

        fn begin_variation(&mut self, _: &mut Self::Movetext) -> ControlFlow<Self::Output, Skip> {
            ControlFlow::Continue(Skip(true))
        }
    };
}

#[cfg(not(test))]
#[inline]
fn create_time_tz(micros: i64, offset_seconds: i32) -> duckdb_time_tz {
    // SAFETY: Only called inside DuckDB (API initialized).
    unsafe { duckdb_create_time_tz(micros, offset_seconds) }
}

/// Streaming PGN visitor (pgn-reader).
/// Spec: pgn-parsing - Visitor Pattern Implementation
///
/// Accumulates mainline movetext into a `String`, includes `{ ... }` comments
/// (whitespace-normalized). Result is captured separately via `outcome()` (or
/// the `Result` tag as fallback).
pub struct GameVisitor {
    headers: HeaderFields,
    movetext_buffer: String,
    move_count: u32,
    result_marker: Option<String>,
    parse_error: ErrorAccumulator,
    pub current_game: Option<GameRecord>,
}

#[derive(Default)]
struct HeaderFields {
    event: String,
    site: String,
    source: String,
    white: String,
    black: String,
    result: String,
    white_title: String,
    black_title: String,
    white_elo: String,
    black_elo: String,
    utc_date: String,
    date: String,
    event_date: String,
    utc_time: String,
    time: String,
    eco: String,
    opening: String,
    termination: String,
    time_control: String,
}

impl HeaderFields {
    fn clear(&mut self) {
        *self = Self::default();
    }

    fn opt_take(field: &mut String) -> Option<String> {
        if field.is_empty() {
            None
        } else {
            Some(mem::take(field))
        }
    }

    fn set_known_tag(&mut self, key: &[u8], value: RawTag<'_>) {
        let slot: &mut String = match key {
            b"Event" => &mut self.event,
            b"Site" => &mut self.site,
            b"Source" => &mut self.source,
            b"White" => &mut self.white,
            b"Black" => &mut self.black,
            b"Result" => &mut self.result,
            b"WhiteTitle" => &mut self.white_title,
            b"BlackTitle" => &mut self.black_title,
            b"WhiteElo" => &mut self.white_elo,
            b"BlackElo" => &mut self.black_elo,
            b"UTCDate" => &mut self.utc_date,
            b"Date" => &mut self.date,
            b"EventDate" => &mut self.event_date,
            b"UTCTime" => &mut self.utc_time,
            b"Time" => &mut self.time,
            b"ECO" => &mut self.eco,
            b"Opening" => &mut self.opening,
            b"Termination" => &mut self.termination,
            b"TimeControl" => &mut self.time_control,
            _ => return,
        };

        if !slot.is_empty() {
            return;
        }

        let bytes = value.as_bytes();
        if bytes.is_empty() {
            return;
        }

        *slot = String::from_utf8_lossy(bytes).into_owned();
    }
}

impl GameVisitor {
    pub fn new() -> Self {
        Self {
            headers: HeaderFields::default(),
            movetext_buffer: String::new(),
            move_count: 0,
            result_marker: None,
            parse_error: ErrorAccumulator::default(),
            current_game: None,
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

    fn last_day_of_month(year: i32, month: u32) -> Option<u32> {
        let first_day_next_month = if month == 12 {
            let next_year = year.checked_add(1)?;
            NaiveDate::from_ymd_opt(next_year, 1, 1)?
        } else {
            let next_month = month.checked_add(1)?;
            NaiveDate::from_ymd_opt(year, next_month, 1)?
        };

        first_day_next_month.pred_opt().map(|d| d.day())
    }

    fn rank_date_candidates<'a>(
        utc_date: Option<&'a str>,
        date: Option<&'a str>,
        event_date: Option<&'a str>,
    ) -> Vec<(&'a str, &'static str)> {
        // Rank by completeness first; if tied, prefer header precedence:
        // UTCDate -> Date -> EventDate
        let mut ranked: Vec<(u8, u8, &'a str, &'static str)> = Vec::new();

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
        utc_date: Option<&str>,
        date: Option<&str>,
        event_date: Option<&str>,
        parse_error: &mut ErrorAccumulator,
    ) -> Option<duckdb_date> {
        for (raw, label) in Self::rank_date_candidates(utc_date, date, event_date) {
            if let Some(parsed) = Self::parse_date_field(raw, label, parse_error) {
                return Some(parsed);
            }
        }

        None
    }

    fn parse_date_field(
        raw: &str,
        label: &str,
        parse_error: &mut ErrorAccumulator,
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
                    parse_error.push(&format!("Conversion error: {label}='{s}'"));
                }
                Err(e) => {
                    parse_error.push(&format!("Conversion error: {label}='{s}' (chrono: {e})"));
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

        let year = match year_s.parse::<i32>() {
            Ok(v) => v,
            Err(e) => {
                parse_error.push(&format!("Conversion error: {label}='{s}' (chrono: {e})"));
                return None;
            }
        };
        let month = match month_s.parse::<u32>() {
            Ok(v) => v,
            Err(e) => {
                parse_error.push(&format!("Conversion error: {label}='{s}' (chrono: {e})"));
                return None;
            }
        };
        let mut day = match day_s.parse::<u32>() {
            Ok(v) => v,
            Err(e) => {
                parse_error.push(&format!("Conversion error: {label}='{s}' (chrono: {e})"));
                return None;
            }
        };

        let Some(last_day) = Self::last_day_of_month(year, month) else {
            parse_error.push(&format!(
                "Conversion error: {label}='{s}' (chrono: input is out of range)"
            ));
            return None;
        };

        if day > last_day {
            day = last_day;
        }

        let date = match NaiveDate::from_ymd_opt(year, month, day) {
            Some(v) => v,
            None => {
                parse_error.push(&format!(
                    "Conversion error: {label}='{s}' (chrono: input is out of range)"
                ));
                return None;
            }
        };

        if date.year() <= 0 {
            parse_error.push(&format!(
                "Conversion error: {label}='{s}' (chrono: year must be >= 1)"
            ));
            return None;
        }

        let days_i64 = date.signed_duration_since(*EPOCH).num_days();
        let days: i32 = match i32::try_from(days_i64) {
            Ok(v) => v,
            Err(_) => {
                parse_error.push(&format!(
                    "Conversion error: {label}='{s}' (chrono: date out of range)"
                ));
                return None;
            }
        };

        Some(duckdb_date { days })
    }

    fn parse_uinteger_field(
        raw: Option<&str>,
        label: &str,
        parse_error: &mut ErrorAccumulator,
    ) -> Option<u32> {
        let s = raw?.trim();
        if s.is_empty() {
            return None;
        }
        match s.parse::<u32>() {
            Ok(v) => Some(v),
            Err(_) => {
                parse_error.push(&format!("Conversion error: {label}='{s}'"));
                None
            }
        }
    }

    fn parse_time_tz_field(
        raw: &str,
        label: &str,
        parse_error: &mut ErrorAccumulator,
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
                        parse_error.push(&format!("Conversion error: {label}='{s}'"));
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
                        parse_error.push(&format!("Conversion error: {label}='{s}'"));
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
                parse_error.push(&format!("Conversion error: {label}='{s}' (chrono: {e})"));
                return None;
            }
        };

        let micros = (time.num_seconds_from_midnight() as i64) * 1_000_000
            + (time.nanosecond() as i64) / 1_000;
        Some(Self::pack_time_tz(micros, offset_seconds))
    }

    fn parse_best_time_tz_field(
        utc_time: Option<&str>,
        time: Option<&str>,
        parse_error: &mut ErrorAccumulator,
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

    fn build_game_record(&mut self) {
        let white_elo = Self::parse_uinteger_field(
            (!self.headers.white_elo.is_empty()).then_some(self.headers.white_elo.as_str()),
            "WhiteElo",
            &mut self.parse_error,
        );
        let black_elo = Self::parse_uinteger_field(
            (!self.headers.black_elo.is_empty()).then_some(self.headers.black_elo.as_str()),
            "BlackElo",
            &mut self.parse_error,
        );

        let utc_date = Self::parse_best_date_field(
            (!self.headers.utc_date.is_empty()).then_some(self.headers.utc_date.as_str()),
            (!self.headers.date.is_empty()).then_some(self.headers.date.as_str()),
            (!self.headers.event_date.is_empty()).then_some(self.headers.event_date.as_str()),
            &mut self.parse_error,
        );
        let utc_time = Self::parse_best_time_tz_field(
            (!self.headers.utc_time.is_empty()).then_some(self.headers.utc_time.as_str()),
            (!self.headers.time.is_empty()).then_some(self.headers.time.as_str()),
            &mut self.parse_error,
        );

        let movetext = {
            let needs_trim = {
                let trimmed = self.movetext_buffer.trim();
                trimmed.len() != self.movetext_buffer.len()
            };

            if needs_trim {
                let trimmed = self.movetext_buffer.trim().to_string();
                let _ = mem::take(&mut self.movetext_buffer);
                trimmed
            } else {
                mem::take(&mut self.movetext_buffer)
            }
        };

        self.current_game = Some(GameRecord {
            event: HeaderFields::opt_take(&mut self.headers.event),
            site: HeaderFields::opt_take(&mut self.headers.site),
            source: HeaderFields::opt_take(&mut self.headers.source),
            white: HeaderFields::opt_take(&mut self.headers.white),
            black: HeaderFields::opt_take(&mut self.headers.black),
            result: HeaderFields::opt_take(&mut self.headers.result)
                .or_else(|| self.result_marker.take()),
            white_title: HeaderFields::opt_take(&mut self.headers.white_title),
            black_title: HeaderFields::opt_take(&mut self.headers.black_title),
            white_elo,
            black_elo,
            utc_date,
            utc_time,
            eco: HeaderFields::opt_take(&mut self.headers.eco),
            opening: HeaderFields::opt_take(&mut self.headers.opening),
            termination: HeaderFields::opt_take(&mut self.headers.termination),
            time_control: HeaderFields::opt_take(&mut self.headers.time_control),
            movetext,
            parse_error: self.parse_error.take(),
        });
    }

    fn finalize_game(&mut self) {
        self.build_game_record();
    }

    /// Spec: pgn-parsing - Error Message Capture
    pub fn finalize_game_with_error(&mut self, error_msg: String) {
        self.parse_error.push(&error_msg);
        self.build_game_record();
    }
}

pub type PgnInput = Box<dyn Read + Send>;

pub struct PgnReaderState {
    pub pgn_reader: Reader<PgnInput>,
    pub path_idx: usize,
    pub next_game_index: usize,
    pub record_buffer: GameRecord,
    pub visitor: GameVisitor,
}

impl PgnReaderState {
    pub fn new(input: PgnInput, path_idx: usize) -> Self {
        Self {
            pgn_reader: Reader::new(input),
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
        self.parse_error = ErrorAccumulator::default();
        self.current_game = None;
        ControlFlow::Continue(())
    }

    fn tag(
        &mut self,
        _: &mut Self::Tags,
        key: &[u8],
        value: RawTag<'_>,
    ) -> ControlFlow<Self::Output> {
        self.headers.set_known_tag(key, value);
        ControlFlow::Continue(())
    }

    fn begin_movetext(&mut self, _: Self::Tags) -> ControlFlow<Self::Output, Self::Movetext> {
        ControlFlow::Continue(String::with_capacity(256))
    }

    fn begin_variation(&mut self, _: &mut Self::Movetext) -> ControlFlow<Self::Output, Skip> {
        ControlFlow::Continue(Skip(true))
    }

    fn san(&mut self, movetext: &mut Self::Movetext, san: SanPlus) -> ControlFlow<Self::Output> {
        if !movetext.is_empty() {
            movetext.push(' ');
        }

        if self.move_count.is_multiple_of(2) {
            let _ = write!(movetext, "{}. ", (self.move_count / 2) + 1);
        }

        let _ = write!(movetext, "{}", san);
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
        movetext.push('{');
        movetext.push(' ');
        movetext.push_str(comment_str.trim());
        movetext.push(' ');
        movetext.push('}');

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
            .take()
            .or_else(|| HeaderFields::opt_take(&mut self.headers.result));
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
    fn test_visitor_unknown_headers_are_ignored() {
        let pgn = r#"[Event "Known"]
[SomeRandomTag "noise"]
[Site "Somewhere"]
1. e4 1-0"#;

        let mut reader = Reader::new(pgn.as_bytes());
        let mut visitor = GameVisitor::new();

        reader.read_game(&mut visitor).unwrap();

        let game = visitor.current_game.expect("Should have parsed a game");
        assert_eq!(game.event.as_deref(), Some("Known"));
        assert_eq!(game.site.as_deref(), Some("Somewhere"));
        assert_eq!(game.result.as_deref(), Some("1-0"));
    }

    #[test]
    fn test_visitor_duplicate_headers_preserve_first_value() {
        let pgn = r#"[Event "First Event"]
[Event "Second Event"]
[WhiteElo "2000"]
[WhiteElo "2500"]
1. e4 1-0"#;

        let mut reader = Reader::new(pgn.as_bytes());
        let mut visitor = GameVisitor::new();

        reader.read_game(&mut visitor).unwrap();

        let game = visitor.current_game.expect("Should have parsed a game");
        assert_eq!(game.event.as_deref(), Some("First Event"));
        assert_eq!(game.white_elo, Some(2000));
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
    fn test_visitor_movetext_finalization_trims_surrounding_whitespace() {
        let mut visitor = GameVisitor::new();
        visitor.movetext_buffer = "  1. e4 e5  ".to_string();

        visitor.build_game_record();

        let game = visitor.current_game.expect("Should have built a record");
        assert_eq!(game.movetext, "1. e4 e5");
    }

    #[test]
    fn test_visitor_error_finalization_trims_movetext_and_sets_parse_error() {
        let mut visitor = GameVisitor::new();
        visitor.movetext_buffer = "  1. e4  ".to_string();

        visitor.finalize_game_with_error("boom".to_string());

        let game = visitor.current_game.expect("Should have built a record");
        assert_eq!(game.movetext, "1. e4");
        assert_eq!(game.parse_error.as_deref(), Some("boom"));
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
