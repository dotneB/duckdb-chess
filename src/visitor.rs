use crate::types::GameRecord;
use pgn_reader::{RawComment, RawTag, SanPlus, Skip, Visitor};
use std::fs::File;
use std::io::BufReader;
use std::ops::ControlFlow;

/// Visitor implementation for pgn-reader
/// Spec: pgn-parsing - Visitor Pattern Implementation
/// Implements streaming PGN parsing using the pgn-reader library's Visitor trait
pub struct GameVisitor {
    headers: Vec<(String, String)>,
    movetext_buffer: String,
    move_count: u32,
    pub current_game: Option<GameRecord>,
}

impl GameVisitor {
    pub fn new() -> Self {
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
    pub fn finalize_game_with_error(&mut self, error_msg: String) {
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

pub struct PgnReaderState {
    pub reader: BufReader<File>,
    pub path_idx: usize,
    pub game_buffer: String,
    pub record_buffer: GameRecord,
    pub line_buffer: Vec<u8>,
    pub visitor: GameVisitor,
}

impl PgnReaderState {
    pub fn new(reader: BufReader<File>, path_idx: usize) -> Self {
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

    fn comment(
        &mut self,
        movetext: &mut Self::Movetext,
        comment: RawComment<'_>,
    ) -> ControlFlow<Self::Output> {
        let comment_str = String::from_utf8_lossy(comment.as_bytes());
        movetext.push_str(&format!(" {{ {} }}", comment_str.trim()));
        ControlFlow::Continue(())
    }

    fn end_game(&mut self, movetext: Self::Movetext) -> Self::Output {
        self.movetext_buffer = movetext;
        self.finalize_game();
    }
}
