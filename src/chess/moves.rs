use ::duckdb::{
    Result,
    core::{DataChunkHandle, LogicalTypeHandle, LogicalTypeId},
    vscalar::{ScalarFunctionSignature, VScalar},
    vtab::arrow::WritableVector,
};
use pgn_reader::{Nag, RawComment, Reader, SanPlus as PgnSanPlus, Skip, Visitor};
use shakmaty::{Chess, EnPassantMode, Position, fen::Fen, san::SanPlus, zobrist::Zobrist64};
use smallvec::SmallVec;
use std::error::Error;
use std::fmt::Write;
use std::io;
use std::ops::ControlFlow;

use super::duckdb_impl::scalar::{
    VarcharNullBehavior, VarcharOutput, invoke_binary_varchar_varchar_to_bool_nullable,
    invoke_unary_varchar_optional_i64_to_varchar, invoke_unary_varchar_to_i64_default,
    invoke_unary_varchar_to_u64_nullable, invoke_unary_varchar_to_varchar,
};
use super::log;
use crate::chess::filter::parse_movetext_mainline;
use crate::pgn_visitor_skip_variations;

type MoveList = SmallVec<[String; 128]>;

pub struct ChessMovesJsonScalar;

impl VScalar for ChessMovesJsonScalar {
    type State = ();

    unsafe fn invoke(
        _state: &Self::State,
        input: &mut DataChunkHandle,
        output: &mut dyn WritableVector,
    ) -> Result<(), Box<dyn Error>> {
        let mut logged_error = false;

        invoke_unary_varchar_optional_i64_to_varchar(
            input,
            output,
            VarcharNullBehavior::Static("[]"),
            |movetext, max_ply| match process_moves_with_limit(movetext, max_ply) {
                Ok(json) => Ok(VarcharOutput::Value(json)),
                Err(e) => {
                    if !logged_error {
                        logged_error = true;
                        log::error(format!("Error processing moves: {e}"));
                    }
                    Ok(VarcharOutput::Value("[]".to_string()))
                }
            },
        )
    }

    fn signatures() -> Vec<ScalarFunctionSignature> {
        vec![
            ScalarFunctionSignature::exact(
                vec![LogicalTypeHandle::from(LogicalTypeId::Varchar)],
                LogicalTypeHandle::from(LogicalTypeId::Varchar),
            ),
            ScalarFunctionSignature::exact(
                vec![
                    LogicalTypeHandle::from(LogicalTypeId::Varchar),
                    LogicalTypeHandle::from(LogicalTypeId::Bigint),
                ],
                LogicalTypeHandle::from(LogicalTypeId::Varchar),
            ),
        ]
    }
}

fn process_moves_with_limit(
    movetext: &str,
    max_ply: Option<i64>,
) -> Result<String, Box<dyn Error>> {
    if movetext.trim().is_empty() {
        return Ok("[]".to_string());
    }

    if let Some(max_ply) = max_ply
        && max_ply <= 0
    {
        return Ok("[]".to_string());
    }

    let max_ply_limit = max_ply.and_then(|v| usize::try_from(v).ok());
    let mut reader = Reader::new(io::Cursor::new(movetext.as_bytes()));
    let mut visitor = MovesJsonVisitor::new(max_ply_limit);

    let _ = reader.read_game(&mut visitor);
    Ok(visitor.finish())
}

struct MovesJsonVisitor {
    position: Chess,
    json: String,
    first: bool,
    ply: usize,
    max_ply: Option<usize>,
}

impl MovesJsonVisitor {
    fn new(max_ply: Option<usize>) -> Self {
        let mut visitor = Self {
            position: Chess::default(),
            json: String::new(),
            first: true,
            ply: 0,
            max_ply,
        };
        visitor.reset();
        visitor
    }

    fn reset(&mut self) {
        self.position = Chess::default();
        self.json.clear();
        self.json.push('[');
        self.first = true;
        self.ply = 0;
    }

    fn finish(mut self) -> String {
        self.json.push(']');
        self.json
    }
}

impl Visitor for MovesJsonVisitor {
    type Tags = ();
    type Movetext = ();
    type Output = ();

    fn begin_tags(&mut self) -> ControlFlow<Self::Output, Self::Tags> {
        self.reset();
        ControlFlow::Continue(())
    }

    fn begin_movetext(&mut self, _tags: Self::Tags) -> ControlFlow<Self::Output, Self::Movetext> {
        ControlFlow::Continue(())
    }

    fn san(
        &mut self,
        _movetext: &mut Self::Movetext,
        san_plus: PgnSanPlus,
    ) -> ControlFlow<Self::Output> {
        if let Some(max_ply) = self.max_ply
            && self.ply >= max_ply
        {
            return ControlFlow::Break(());
        }

        let next_move = match san_plus.san.to_move(&self.position) {
            Ok(next_move) => next_move,
            Err(_) => return ControlFlow::Break(()),
        };

        self.position.play_unchecked(next_move);
        self.ply += 1;

        if !self.first {
            self.json.push(',');
        }
        self.first = false;

        let fen = duckdb_fen(&self.position);
        let epd = fen_str_to_epd(&fen).unwrap_or_default();

        let _ = write!(
            self.json,
            r#"{{"ply":{},"move":"{}","fen":"{}","epd":"{}"}}"#,
            self.ply, san_plus, fen, epd
        );

        ControlFlow::Continue(())
    }

    pgn_visitor_skip_variations!();

    fn end_game(&mut self, _movetext: Self::Movetext) -> Self::Output {}
}

fn duckdb_fen(pos: &Chess) -> String {
    let fen = Fen::from_position(pos, EnPassantMode::Always);
    fen.to_string()
}

fn fen_str_to_epd(fen: &str) -> Option<String> {
    let mut fields = fen.split_whitespace();
    let board = fields.next()?;
    let side = fields.next()?;
    let castling = fields.next()?;
    let ep = fields.next()?;
    Some(format!("{} {} {} {}", board, side, castling, ep))
}

fn fen_to_epd(fen: &str) -> Option<String> {
    let fen = fen.trim();
    if fen.is_empty() {
        return None;
    }

    let parsed: Fen = fen.parse().ok()?;
    fen_str_to_epd(&parsed.to_string())
}

// Spec: move-analysis - FEN to EPD
pub struct ChessFenEpdScalar;

impl VScalar for ChessFenEpdScalar {
    type State = ();

    unsafe fn invoke(
        _state: &Self::State,
        input: &mut DataChunkHandle,
        output: &mut dyn WritableVector,
    ) -> Result<(), Box<dyn Error>> {
        invoke_unary_varchar_to_varchar(input, output, VarcharNullBehavior::Null, |fen| {
            Ok(match fen_to_epd(fen) {
                Some(epd) => VarcharOutput::Value(epd),
                None => VarcharOutput::Null,
            })
        })
    }

    fn signatures() -> Vec<ScalarFunctionSignature> {
        vec![ScalarFunctionSignature::exact(
            vec![LogicalTypeHandle::from(LogicalTypeId::Varchar)],
            LogicalTypeHandle::from(LogicalTypeId::Varchar),
        )]
    }
}

// Spec: move-analysis - Ply Count
pub struct ChessPlyCountScalar;

impl VScalar for ChessPlyCountScalar {
    type State = ();

    unsafe fn invoke(
        _state: &Self::State,
        input: &mut DataChunkHandle,
        output: &mut dyn WritableVector,
    ) -> Result<(), Box<dyn Error>> {
        invoke_unary_varchar_to_i64_default(input, output, 0, ply_count)
    }

    fn signatures() -> Vec<ScalarFunctionSignature> {
        vec![ScalarFunctionSignature::exact(
            vec![LogicalTypeHandle::from(LogicalTypeId::Varchar)],
            LogicalTypeHandle::from(LogicalTypeId::Bigint),
        )]
    }
}

fn ply_count(movetext: &str) -> i64 {
    if movetext.trim().is_empty() {
        return 0;
    }

    let mut reader = Reader::new(io::Cursor::new(movetext.as_bytes()));
    let mut visitor = PlyCountVisitor::default();

    match reader.read_game(&mut visitor) {
        Ok(Some(())) => visitor.count as i64,
        Ok(None) | Err(_) => 0,
    }
}

#[derive(Default)]
struct PlyCountVisitor {
    count: usize,
}

impl Visitor for PlyCountVisitor {
    type Tags = ();
    type Movetext = ();
    type Output = ();

    fn begin_tags(&mut self) -> ControlFlow<Self::Output, Self::Tags> {
        self.count = 0;
        ControlFlow::Continue(())
    }

    fn begin_movetext(&mut self, _tags: Self::Tags) -> ControlFlow<Self::Output, Self::Movetext> {
        ControlFlow::Continue(())
    }

    fn san(
        &mut self,
        _movetext: &mut Self::Movetext,
        _san_plus: PgnSanPlus,
    ) -> ControlFlow<Self::Output> {
        self.count += 1;
        ControlFlow::Continue(())
    }

    pgn_visitor_skip_variations!();

    fn end_game(&mut self, _movetext: Self::Movetext) -> Self::Output {}
}

// Spec: move-analysis - Moves Hashing
pub struct ChessMovesHashScalar;

fn zobrist_hash_of_position(pos: &Chess) -> u64 {
    let Zobrist64(v) = pos.zobrist_hash::<Zobrist64>(EnPassantMode::Legal);
    v
}

#[derive(Default)]
struct ZobristHashVisitor {
    pos: Chess,
    hash: u64,
}

impl ZobristHashVisitor {
    fn init(&mut self) {
        self.pos = Chess::default();
        self.hash = zobrist_hash_of_position(&self.pos);
    }
}

impl Visitor for ZobristHashVisitor {
    type Tags = ();
    type Movetext = ();
    type Output = ();

    fn begin_tags(&mut self) -> ControlFlow<Self::Output, Self::Tags> {
        self.init();
        ControlFlow::Continue(())
    }

    fn begin_movetext(&mut self, _tags: Self::Tags) -> ControlFlow<Self::Output, Self::Movetext> {
        ControlFlow::Continue(())
    }

    fn san(
        &mut self,
        _movetext: &mut Self::Movetext,
        san_plus: PgnSanPlus,
    ) -> ControlFlow<Self::Output> {
        let m = match san_plus.san.to_move(&self.pos) {
            Ok(m) => m,
            Err(_) => return ControlFlow::Break(()),
        };

        self.pos.play_unchecked(m);
        self.hash = zobrist_hash_of_position(&self.pos);

        ControlFlow::Continue(())
    }

    pgn_visitor_skip_variations!();

    fn end_game(&mut self, _movetext: Self::Movetext) -> Self::Output {}
}

fn movetext_final_zobrist_hash(movetext: &str) -> Option<u64> {
    if movetext.trim().is_empty() {
        return None;
    }

    let mut reader = Reader::new(io::Cursor::new(movetext.as_bytes()));
    let mut visitor = ZobristHashVisitor::default();
    visitor.init();

    match reader.read_game(&mut visitor) {
        Ok(Some(())) => Some(visitor.hash),
        Ok(None) => None,
        Err(_) => None,
    }
}

impl VScalar for ChessMovesHashScalar {
    type State = ();

    unsafe fn invoke(
        _state: &Self::State,
        input: &mut DataChunkHandle,
        output: &mut dyn WritableVector,
    ) -> Result<(), Box<dyn Error>> {
        invoke_unary_varchar_to_u64_nullable(input, output, movetext_final_zobrist_hash)
    }

    fn signatures() -> Vec<ScalarFunctionSignature> {
        vec![ScalarFunctionSignature::exact(
            vec![LogicalTypeHandle::from(LogicalTypeId::Varchar)],
            LogicalTypeHandle::from(LogicalTypeId::UBigint),
        )]
    }
}

// Spec: move-analysis - Subsumption Detection
pub struct ChessMovesSubsetScalar;

impl VScalar for ChessMovesSubsetScalar {
    type State = ();

    unsafe fn invoke(
        _state: &Self::State,
        input: &mut DataChunkHandle,
        output: &mut dyn WritableVector,
    ) -> Result<(), Box<dyn Error>> {
        invoke_binary_varchar_varchar_to_bool_nullable(input, output, check_moves_subset)
    }

    fn signatures() -> Vec<ScalarFunctionSignature> {
        vec![ScalarFunctionSignature::exact(
            vec![
                LogicalTypeHandle::from(LogicalTypeId::Varchar),
                LogicalTypeHandle::from(LogicalTypeId::Varchar),
            ],
            LogicalTypeHandle::from(LogicalTypeId::Boolean),
        )]
    }
}

fn check_moves_subset(short_movetext: &str, long_movetext: &str) -> bool {
    if let Some(fast_result) = check_moves_subset_fast(short_movetext, long_movetext) {
        return fast_result;
    }

    check_moves_subset_with_parser(short_movetext, long_movetext)
}

fn check_moves_subset_fast(short_movetext: &str, long_movetext: &str) -> Option<bool> {
    if !is_clean_mainline_movetext(short_movetext) || !is_clean_mainline_movetext(long_movetext) {
        return None;
    }

    let short_moves = extract_clean_mainline_sans(short_movetext)?;
    let long_moves = extract_clean_mainline_sans(long_movetext)?;

    Some(is_prefix_subset(&short_moves, &long_moves))
}

fn is_clean_mainline_movetext(movetext: &str) -> bool {
    let trimmed = movetext.trim();
    if trimmed.is_empty() {
        return true;
    }

    if trimmed.chars().any(is_uncertain_syntax_char) {
        return false;
    }

    let mut saw_result = false;
    let mut saw_san = false;

    for token in trimmed.split_whitespace() {
        if saw_result {
            return false;
        }

        if is_move_number_token(token) {
            continue;
        }

        if is_result_marker(token) {
            saw_result = true;
            continue;
        }

        if !looks_like_san_token(token) {
            return false;
        }

        saw_san = true;
    }

    saw_san
}

fn is_uncertain_syntax_char(c: char) -> bool {
    matches!(c, '{' | '}' | '(' | ')' | '$' | '!' | '?' | ';')
}

fn is_move_number_token(token: &str) -> bool {
    let Some(first_dot_index) = token.find('.') else {
        return false;
    };

    if first_dot_index == 0 {
        return false;
    }

    let (digits, dots) = token.split_at(first_dot_index);
    if !digits.chars().all(|c| c.is_ascii_digit()) {
        return false;
    }

    dots == "." || dots == "..."
}

fn is_result_marker(token: &str) -> bool {
    matches!(token, "1-0" | "0-1" | "1/2-1/2" | "*")
}

fn looks_like_san_token(token: &str) -> bool {
    if token.is_empty() || !token.is_ascii() || token.contains('.') {
        return false;
    }

    if matches!(
        token,
        "O-O" | "O-O+" | "O-O#" | "O-O-O" | "O-O-O+" | "O-O-O#"
    ) {
        return true;
    }

    let Some(first_byte) = token.as_bytes().first() else {
        return false;
    };

    if !matches!(*first_byte, b'K' | b'Q' | b'R' | b'B' | b'N' | b'a'..=b'h') {
        return false;
    }

    token
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || matches!(c, 'x' | '+' | '#' | '=' | '-'))
}

fn extract_clean_mainline_sans(movetext: &str) -> Option<MoveList> {
    if movetext.trim().is_empty() {
        return Some(MoveList::new());
    }

    let mut saw_result = false;
    let mut position = Chess::default();
    let mut sans = MoveList::new();

    for token in movetext.split_whitespace() {
        if saw_result {
            return None;
        }

        if is_move_number_token(token) {
            continue;
        }

        if is_result_marker(token) {
            saw_result = true;
            continue;
        }

        if !looks_like_san_token(token) {
            return None;
        }

        let san_plus: SanPlus = token.parse().ok()?;
        let m = san_plus.san.to_move(&position).ok()?;
        position.play_unchecked(m);

        sans.push(san_plus.to_string());
    }

    Some(sans)
}

fn is_prefix_subset(short_moves: &[String], long_moves: &[String]) -> bool {
    if short_moves.len() > long_moves.len() {
        return false;
    }

    short_moves
        .iter()
        .zip(long_moves.iter())
        .all(|(short, long)| short == long)
}

fn check_moves_subset_with_parser(short_movetext: &str, long_movetext: &str) -> bool {
    let short_parsed = parse_movetext_mainline(short_movetext);
    let long_parsed = parse_movetext_mainline(long_movetext);
    let short_non_empty = !short_movetext.trim().is_empty();
    let long_non_empty = !long_movetext.trim().is_empty();

    let short_parse_failed = short_parsed.parse_error
        || (short_non_empty && short_parsed.sans.is_empty() && short_parsed.outcome.is_none());
    let long_parse_failed = long_parsed.parse_error
        || (long_non_empty && long_parsed.sans.is_empty() && long_parsed.outcome.is_none());

    if short_parse_failed {
        return false;
    }
    if long_parse_failed {
        return false;
    }

    is_prefix_subset(&short_parsed.sans, &long_parsed.sans)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process_moves_basic() {
        let input = "1. e4 e5";
        let json = process_moves_with_limit(input, None).unwrap();
        // Check structure roughly
        assert!(json.starts_with('['));
        assert!(json.ends_with(']'));
        assert!(json.contains(r#""ply":1,"move":"e4""#));
        assert!(json.contains(r#""ply":2,"move":"e5""#));
        assert!(json.contains(r#""epd":"#));
    }

    #[test]
    fn test_process_moves_with_annotations() {
        let input = "1. e4 {comment} e5";
        let json = process_moves_with_limit(input, None).unwrap();
        assert!(json.contains(r#""move":"e5""#));
    }

    #[test]
    fn test_process_moves_empty() {
        let input = "";
        let json = process_moves_with_limit(input, None).unwrap();
        assert_eq!(json, "[]");
    }

    #[test]
    fn test_process_moves_max_ply_zero() {
        let input = "1. e4 e5";
        let json = process_moves_with_limit(input, Some(0)).unwrap();
        assert_eq!(json, "[]");
    }

    #[test]
    fn test_process_moves_with_result_marker() {
        let input = "1. e4 e5 1-0";
        let json = process_moves_with_limit(input, None).unwrap();
        assert!(json.contains(r#""ply":1,"move":"e4""#));
        assert!(json.contains(r#""ply":2,"move":"e5""#));
        // Should not contain result marker
        assert!(!json.contains("1-0"));
    }

    #[test]
    fn test_process_moves_with_invalid_move() {
        let input = "1. e4 e5 INVALID";
        let json = process_moves_with_limit(input, None).unwrap();
        // Should return valid prefix up to e5
        assert!(json.contains(r#""ply":1,"move":"e4""#));
        assert!(json.contains(r#""ply":2,"move":"e5""#));
        // Should not include INVALID move
        assert!(!json.contains("INVALID"));
    }

    #[test]
    fn test_process_moves_malformed_non_pgn_returns_empty_array() {
        let json = process_moves_with_limit("this is not movetext", None).unwrap();
        assert_eq!(json, "[]");
    }

    #[test]
    fn test_process_moves_unterminated_comment_keeps_valid_prefix() {
        let json = process_moves_with_limit("1. e4 { unterminated comment", None).unwrap();
        assert!(json.starts_with('['));
        assert!(json.ends_with(']'));
        assert!(json.contains(r#""ply":1,"move":"e4""#));
    }

    #[test]
    fn test_fen_to_epd_valid() {
        let fen = "rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 1";
        assert_eq!(
            fen_to_epd(fen).as_deref(),
            Some("rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3")
        );
    }

    #[test]
    fn test_fen_to_epd_invalid() {
        assert!(fen_to_epd("not a fen").is_none());
        assert!(fen_to_epd("").is_none());
    }

    #[test]
    fn test_ply_count_ignores_junk_and_stops() {
        assert_eq!(ply_count("1. e4! {c} e5?? 1-0"), 2);
        assert_eq!(ply_count("1. e4 e5 INVALID 2. Nf3"), 3);
        assert_eq!(ply_count("1. e4 INVALID 2. Nf3"), 2);
        assert_eq!(ply_count("1. e4 e5 2. Kxe8"), 3);
    }

    #[test]
    fn test_ply_count_malformed_parse_returns_zero() {
        assert_eq!(ply_count("1. e4 { unterminated comment"), 0);
    }

    #[test]
    fn test_ply_count_empty_or_whitespace_returns_zero() {
        assert_eq!(ply_count(""), 0);
        assert_eq!(ply_count("   \n\t"), 0);
    }

    #[test]
    fn test_chess_moves_hash_consistency_formatting() {
        // Test identical moves with different formatting produce same hash
        let hash1 = movetext_final_zobrist_hash("1. e4 e5").unwrap();
        let hash2 = movetext_final_zobrist_hash("1.e4 e5").unwrap();
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_chess_moves_hash_consistency_comments() {
        // Test identical moves with comments produce same hash
        let hash1 = movetext_final_zobrist_hash("1. e4 e5").unwrap();
        let hash2 = movetext_final_zobrist_hash("1. e4 {comment} e5").unwrap();
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_chess_moves_hash_consistency_variations() {
        // Test identical moves with variations produce same hash
        let hash1 = movetext_final_zobrist_hash("1. e4 e5").unwrap();
        let hash2 = movetext_final_zobrist_hash("1. e4 (1. d4) e5").unwrap();
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_chess_moves_hash_consistency_nags() {
        // Test identical moves with NAGs produce same hash
        let hash1 = movetext_final_zobrist_hash("1. e4 e5").unwrap();
        let hash2 = movetext_final_zobrist_hash("1. e4! e5?").unwrap();
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_chess_moves_hash_discrimination_different_moves() {
        // Test different moves produce different hashes
        let hash1 = movetext_final_zobrist_hash("1. e4 e5").unwrap();
        let hash2 = movetext_final_zobrist_hash("1. d4 d5").unwrap();
        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_chess_moves_hash_discrimination_different_length() {
        // Test different length sequences produce different hashes
        let hash1 = movetext_final_zobrist_hash("1. e4").unwrap();
        let hash2 = movetext_final_zobrist_hash("1. e4 e5").unwrap();
        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_chess_moves_hash_empty_string() {
        // Empty input returns NULL.
        assert!(movetext_final_zobrist_hash("").is_none());
    }

    #[test]
    fn test_chess_moves_hash_transposition_collision() {
        let hash1 = movetext_final_zobrist_hash("1. Nf3 d5 2. g3").unwrap();
        let hash2 = movetext_final_zobrist_hash("1. g3 d5 2. Nf3").unwrap();
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_chess_moves_subset_exact_subset() {
        // Test short is prefix of long
        assert!(check_moves_subset("1. e4", "1. e4 e5"));
    }

    #[test]
    fn test_chess_moves_subset_different_moves() {
        // Test different moves
        assert!(!check_moves_subset("1. d4", "1. e4 e5"));
    }

    #[test]
    fn test_chess_moves_subset_same_game() {
        // Test identical sequences
        assert!(check_moves_subset("1. e4 e5", "1. e4 e5"));
    }

    #[test]
    fn test_chess_moves_subset_short_longer_than_long() {
        // Test short is longer than long
        assert!(!check_moves_subset("1. e4 e5 2. Nf3", "1. e4"));
    }

    #[test]
    fn test_chess_moves_subset_with_annotations() {
        // Test subset with annotations ignored
        assert!(check_moves_subset("1. e4 {comment} e5", "1. e4 e5 2. Nf3"));
    }

    #[test]
    fn test_chess_moves_subset_with_variations() {
        // Test subset with variations ignored
        assert!(check_moves_subset("1. e4 (1. d4) e5", "1. e4 e5 2. Nf3"));
    }

    #[test]
    fn test_chess_moves_subset_with_nags() {
        // Test subset with NAGs ignored
        assert!(check_moves_subset("1. e4! e5?", "1. e4 e5 2. Nf3"));
    }

    #[test]
    fn test_chess_moves_subset_empty_cases() {
        // Test empty string cases
        assert!(check_moves_subset("", "1. e4"));
        assert!(!check_moves_subset("1. e4", ""));
        assert!(check_moves_subset("", ""));
    }

    #[test]
    fn test_chess_moves_subset_invalid_non_empty_short() {
        assert!(!check_moves_subset("not movetext", "1. e4"));
    }

    #[test]
    fn test_chess_moves_subset_invalid_non_empty_long() {
        assert!(!check_moves_subset("1. e4", "not movetext"));
    }

    #[test]
    fn test_chess_moves_subset_both_invalid_non_empty() {
        assert!(!check_moves_subset("not movetext", "still not movetext"));
    }

    #[test]
    fn test_chess_moves_subset_fast_path_clean_equivalence() {
        let cases = [
            ("1. e4", "1. e4 e5", true),
            ("1. e4 e5", "1. e4 e5", true),
            ("1. d4", "1. e4 e5", false),
            ("1. e4 e5 2. Nf3", "1. e4", false),
        ];

        for (short, long, expected) in cases {
            assert_eq!(check_moves_subset_fast(short, long), Some(expected));
            assert_eq!(check_moves_subset_with_parser(short, long), expected);
            assert_eq!(check_moves_subset(short, long), expected);
        }
    }

    #[test]
    fn test_chess_moves_subset_fast_path_ignores_trailing_results() {
        let cases = [
            ("1. e4 e5 1-0", "1. e4 e5", true),
            ("1. e4 e5", "1. e4 e5 0-1", true),
            ("1. e4 e5 1/2-1/2", "1. e4 e5 *", true),
            ("1. e4 e5 2. Nf3 *", "1. e4 e5", false),
        ];

        for (short, long, expected) in cases {
            assert_eq!(check_moves_subset_fast(short, long), Some(expected));
            assert_eq!(check_moves_subset_with_parser(short, long), expected);
            assert_eq!(check_moves_subset(short, long), expected);
        }
    }

    #[test]
    fn test_chess_moves_subset_falls_back_for_uncertain_input() {
        let cases = [
            ("1. e4 {comment} e5", "1. e4 e5 2. Nf3"),
            ("1. e4 (1. d4) e5", "1. e4 e5 2. Nf3"),
            ("1. e4! e5?", "1. e4 e5 2. Nf3"),
        ];

        for (short, long) in cases {
            assert_eq!(check_moves_subset_fast(short, long), None);
            assert_eq!(
                check_moves_subset(short, long),
                check_moves_subset_with_parser(short, long)
            );
        }
    }

    #[test]
    fn test_chess_moves_subset_falls_back_for_invalid_clean_tokens() {
        assert_eq!(check_moves_subset_fast("1. e4 e4", "1. e4 e4"), None);
        assert_eq!(
            check_moves_subset("1. e4 e4", "1. e4 e4"),
            check_moves_subset_with_parser("1. e4 e4", "1. e4 e4")
        );
    }

    #[test]
    fn test_is_clean_mainline_movetext_detector() {
        assert!(is_clean_mainline_movetext("1. e4 e5 2. Nf3 Nc6"));
        assert!(is_clean_mainline_movetext("e4 e5"));
        assert!(is_clean_mainline_movetext("   "));

        assert!(!is_clean_mainline_movetext("1."));
        assert!(!is_clean_mainline_movetext("1.e4 e5"));
        assert!(!is_clean_mainline_movetext("1. e4 {comment} e5"));
        assert!(!is_clean_mainline_movetext("1. e4 (1. d4) e5"));
        assert!(!is_clean_mainline_movetext("1. e4! e5?"));
        assert!(!is_clean_mainline_movetext("not movetext"));
    }
}
