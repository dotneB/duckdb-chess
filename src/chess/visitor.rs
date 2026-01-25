use super::types::GameRecord;
use libduckdb_sys::{duckdb_date, duckdb_time_tz};
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

    fn is_unknown_date(s: &str) -> bool {
        // Lichess uses "????.??.??" for unknown dates.
        s.contains('?')
    }

    // Howard Hinnant's days-from-civil algorithm (proleptic Gregorian).
    // Returns days since 1970-01-01.
    fn days_from_civil(year: i32, month: u32, day: u32) -> i32 {
        let y = year - if month <= 2 { 1 } else { 0 };
        let era = if y >= 0 { y } else { y - 399 } / 400;
        let yoe = y - era * 400;
        let m = month as i32;
        let doy = (153 * (m + if m > 2 { -3 } else { 9 }) + 2) / 5 + day as i32 - 1;
        let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
        // 719468 is days from civil 0000-03-01 to 1970-01-01.
        era * 146097 + doe - 719468
    }

    fn is_leap_year(year: i32) -> bool {
        (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
    }

    fn days_in_month(year: i32, month: u32) -> u32 {
        match month {
            1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
            4 | 6 | 9 | 11 => 30,
            2 => {
                if Self::is_leap_year(year) {
                    29
                } else {
                    28
                }
            }
            _ => 0,
        }
    }

    fn parse_date_field(
        raw: Option<String>,
        label: &str,
        parse_error: &mut Option<String>,
    ) -> Option<duckdb_date> {
        let raw = raw?;
        let s = raw.trim();
        if s.is_empty() {
            return None;
        }
        if Self::is_unknown_date(s) {
            return None;
        }

        let parts: Vec<&str> = if s.contains('.') {
            s.split('.').collect()
        } else if s.contains('-') {
            s.split('-').collect()
        } else {
            vec![]
        };

        if parts.len() != 3 {
            Self::push_error(parse_error, format!("Conversion error: {label}='{s}'"));
            return None;
        }

        let year: i32 = match parts[0].parse() {
            Ok(v) => v,
            Err(_) => {
                Self::push_error(parse_error, format!("Conversion error: {label}='{s}'"));
                return None;
            }
        };
        let month: u32 = match parts[1].parse() {
            Ok(v) => v,
            Err(_) => {
                Self::push_error(parse_error, format!("Conversion error: {label}='{s}'"));
                return None;
            }
        };
        let day: u32 = match parts[2].parse() {
            Ok(v) => v,
            Err(_) => {
                Self::push_error(parse_error, format!("Conversion error: {label}='{s}'"));
                return None;
            }
        };

        if year <= 0 || !(1..=12).contains(&month) {
            Self::push_error(parse_error, format!("Conversion error: {label}='{s}'"));
            return None;
        }
        let dim = Self::days_in_month(year, month);
        if day < 1 || day > dim {
            Self::push_error(parse_error, format!("Conversion error: {label}='{s}'"));
            return None;
        }

        Some(duckdb_date {
            days: Self::days_from_civil(year, month, day),
        })
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
        raw: Option<String>,
        label: &str,
        parse_error: &mut Option<String>,
    ) -> Option<duckdb_time_tz> {
        let raw = raw?;
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

        let parts: Vec<&str> = time_part.split(':').collect();
        if parts.len() != 3 {
            Self::push_error(parse_error, format!("Conversion error: {label}='{s}'"));
            return None;
        }
        let hh: i64 = match parts[0].parse() {
            Ok(v) => v,
            Err(_) => {
                Self::push_error(parse_error, format!("Conversion error: {label}='{s}'"));
                return None;
            }
        };
        let mm: i64 = match parts[1].parse() {
            Ok(v) => v,
            Err(_) => {
                Self::push_error(parse_error, format!("Conversion error: {label}='{s}'"));
                return None;
            }
        };
        let ss: i64 = match parts[2].parse() {
            Ok(v) => v,
            Err(_) => {
                Self::push_error(parse_error, format!("Conversion error: {label}='{s}'"));
                return None;
            }
        };

        if !(0..=23).contains(&hh) || !(0..=59).contains(&mm) || !(0..=59).contains(&ss) {
            Self::push_error(parse_error, format!("Conversion error: {label}='{s}'"));
            return None;
        }

        let micros = (hh * 3600 + mm * 60 + ss) * 1_000_000;
        Some(Self::pack_time_tz(micros, offset_seconds))
    }

    fn pack_time_tz(micros: i64, offset_seconds: i32) -> duckdb_time_tz {
        // TIME_TZ is stored as 40 bits for micros, and 24 bits for offset.
        // Negative offsets are stored as 24-bit two's complement.
        // DuckDB packs the micros in the high bits.
        let micros_part = (micros as u64) & ((1u64 << 40) - 1);
        let offset_part = (offset_seconds as i64 as u64) & ((1u64 << 24) - 1);
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

        let utc_date_raw = self
            .get_header("UTCDate")
            .or_else(|| self.get_header("Date"));
        let utc_date_label = if self.get_header("UTCDate").is_some() {
            "UTCDate"
        } else {
            "UTCDate (from Date)"
        };
        let utc_date = Self::parse_date_field(utc_date_raw, utc_date_label, &mut parse_error);

        let utc_time_raw = self
            .get_header("UTCTime")
            .or_else(|| self.get_header("Time"));
        let utc_time_label = if self.get_header("UTCTime").is_some() {
            "UTCTime"
        } else {
            "UTCTime (from Time)"
        };
        let utc_time = Self::parse_time_tz_field(utc_time_raw, utc_time_label, &mut parse_error);

        self.current_game = Some(GameRecord {
            event: self.get_header("Event"),
            site: self.get_header("Site"),
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

        let utc_date_raw = self
            .get_header("UTCDate")
            .or_else(|| self.get_header("Date"));
        let utc_date_label = if self.get_header("UTCDate").is_some() {
            "UTCDate"
        } else {
            "UTCDate (from Date)"
        };
        let utc_date = Self::parse_date_field(utc_date_raw, utc_date_label, &mut parse_error);

        let utc_time_raw = self
            .get_header("UTCTime")
            .or_else(|| self.get_header("Time"));
        let utc_time_label = if self.get_header("UTCTime").is_some() {
            "UTCTime"
        } else {
            "UTCTime (from Time)"
        };
        let utc_time = Self::parse_time_tz_field(utc_time_raw, utc_time_label, &mut parse_error);

        self.current_game = Some(GameRecord {
            event: self.get_header("Event"),
            site: self.get_header("Site"),
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
    pub record_buffer: GameRecord,
    pub visitor: GameVisitor,
}

impl PgnReaderState {
    pub fn new(file: File, path_idx: usize) -> Self {
        Self {
            pgn_reader: Reader::new(file),
            path_idx,
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
