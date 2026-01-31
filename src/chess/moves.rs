use duckdb::{
    Result,
    core::{DataChunkHandle, Inserter, LogicalTypeHandle, LogicalTypeId},
    vscalar::{ScalarFunctionSignature, VScalar},
    vtab::arrow::WritableVector,
};
use libduckdb_sys::duckdb_string_t;
use shakmaty::{Chess, EnPassantMode, Position, fen::Fen, san::SanPlus};
use std::collections::hash_map::DefaultHasher;
use std::error::Error;
use std::ffi::CString;
use std::fmt::Write;
use std::hash::{Hash, Hasher};

use crate::chess::filter::parse_movetext_mainline;

pub struct ChessMovesJsonScalar;

impl VScalar for ChessMovesJsonScalar {
    type State = ();

    unsafe fn invoke(
        _state: &Self::State,
        input: &mut DataChunkHandle,
        output: &mut dyn WritableVector,
    ) -> Result<(), Box<dyn Error>> {
        let len = input.len();
        let input_vec = input.flat_vector(0);
        let output_vec = output.flat_vector();

        let max_ply_vec = if input.num_columns() > 1 {
            Some(input.flat_vector(1))
        } else {
            None
        };

        let input_slice = input_vec.as_slice::<duckdb_string_t>();

        for (i, s) in input_slice.iter().take(len).enumerate() {
            if input_vec.row_is_null(i as u64) {
                output_vec.insert(i, CString::new("[]")?);
                continue;
            }

            let val = unsafe { read_duckdb_string(*s) };

            let max_ply = match &max_ply_vec {
                None => None,
                Some(v) => {
                    if v.row_is_null(i as u64) {
                        None
                    } else {
                        Some(v.as_slice::<i64>()[i])
                    }
                }
            };

            match process_moves_with_limit(&val, max_ply) {
                Ok(json) => {
                    output_vec.insert(i, CString::new(json)?);
                }
                Err(e) => {
                    eprintln!("Error processing moves: {}", e);
                    output_vec.insert(i, CString::new("[]")?);
                }
            }
        }
        Ok(())
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
    if let Some(max_ply) = max_ply
        && max_ply <= 0
    {
        return Ok("[]".to_string());
    }

    let parsed = parse_movetext_mainline(movetext);
    let mut pos = Chess::default();
    let mut json = String::new();

    json.push('[');

    let mut first = true;
    let mut ply = 0;

    for token in parsed.sans {
        let san: SanPlus = match token.parse() {
            Ok(s) => s,
            Err(_) => break,
        };

        let m = match san.san.to_move(&pos) {
            Ok(m) => m,
            Err(_) => break,
        };

        pos.play_unchecked(m);
        ply += 1;

        if let Some(max_ply) = max_ply
            && ply > max_ply
        {
            break;
        }

        if !first {
            json.push(',');
        }
        first = false;

        let fen = duckdb_fen(&pos);
        let epd = fen_str_to_epd(&fen).unwrap_or_default();

        write!(
            json,
            r#"{{"ply":{},"move":"{}","fen":"{}","epd":"{}"}}"#,
            ply, token, fen, epd
        )?;
    }

    json.push(']');
    Ok(json)
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
        let len = input.len();
        let input_vec = input.flat_vector(0);
        let mut output_vec = output.flat_vector();

        let input_slice = input_vec.as_slice::<duckdb_string_t>();

        for (i, s) in input_slice.iter().take(len).enumerate() {
            if input_vec.row_is_null(i as u64) {
                output_vec.set_null(i);
                continue;
            }

            let val = unsafe { read_duckdb_string(*s) };
            match fen_to_epd(&val) {
                Some(epd) => output_vec.insert(i, CString::new(epd)?),
                None => output_vec.set_null(i),
            }
        }

        Ok(())
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
        let len = input.len();
        let input_vec = input.flat_vector(0);
        let mut output_vec = output.flat_vector();

        let input_slice = input_vec.as_slice::<duckdb_string_t>();
        let output_slice = output_vec.as_mut_slice::<i64>();

        for (i, s) in input_slice.iter().take(len).enumerate() {
            if input_vec.row_is_null(i as u64) {
                output_slice[i] = 0;
                continue;
            }

            let val = unsafe { read_duckdb_string(*s) };
            output_slice[i] = ply_count(&val);
        }

        Ok(())
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

    let parsed = parse_movetext_mainline(movetext);
    if parsed.parse_error {
        return 0;
    }

    parsed.sans.len() as i64
}

unsafe fn read_duckdb_string(s: duckdb_string_t) -> String {
    if unsafe { s.value.inlined.length } <= 12 {
        let len = unsafe { s.value.inlined.length } as usize;
        let slice = unsafe { &s.value.inlined.inlined };
        let slice_u8 = unsafe { std::slice::from_raw_parts(slice.as_ptr() as *const u8, len) };
        String::from_utf8_lossy(slice_u8).into_owned()
    } else {
        let len = unsafe { s.value.pointer.length } as usize;
        let ptr = unsafe { s.value.pointer.ptr };
        let slice = unsafe { std::slice::from_raw_parts(ptr as *const u8, len) };
        String::from_utf8_lossy(slice).into_owned()
    }
}

// Spec: move-analysis - Moves Hashing
pub struct ChessMovesHashScalar;

impl VScalar for ChessMovesHashScalar {
    type State = ();

    unsafe fn invoke(
        _state: &Self::State,
        input: &mut DataChunkHandle,
        output: &mut dyn WritableVector,
    ) -> Result<(), Box<dyn Error>> {
        let len = input.len();
        let input_vec = input.flat_vector(0);
        let mut output_vec = output.flat_vector();

        let input_slice = input_vec.as_slice::<duckdb_string_t>();
        let output_slice = output_vec.as_mut_slice::<i64>();

        for (out, s) in output_slice.iter_mut().take(len).zip(input_slice.iter()) {
            let val = unsafe { read_duckdb_string(*s) };
            let canonical = parse_movetext_mainline(&val).sans.join(" ");

            // Compute hash
            let mut hasher = DefaultHasher::new();
            canonical.hash(&mut hasher);
            *out = hasher.finish() as i64;
        }
        Ok(())
    }

    fn signatures() -> Vec<ScalarFunctionSignature> {
        vec![ScalarFunctionSignature::exact(
            vec![LogicalTypeHandle::from(LogicalTypeId::Varchar)],
            LogicalTypeHandle::from(LogicalTypeId::Bigint),
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
        let len = input.len();
        let input_vec_0 = input.flat_vector(0);
        let input_vec_1 = input.flat_vector(1);
        let mut output_vec = output.flat_vector();

        let input_slice_0 = input_vec_0.as_slice::<duckdb_string_t>();
        let input_slice_1 = input_vec_1.as_slice::<duckdb_string_t>();
        let output_slice = output_vec.as_mut_slice::<bool>();

        for ((out, s0), s1) in output_slice
            .iter_mut()
            .take(len)
            .zip(input_slice_0.iter())
            .zip(input_slice_1.iter())
        {
            let short_text = unsafe { read_duckdb_string(*s0) };
            let long_text = unsafe { read_duckdb_string(*s1) };

            *out = check_moves_subset(&short_text, &long_text);
        }
        Ok(())
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
    let short_moves = parse_movetext_mainline(short_movetext).sans;
    let long_moves = parse_movetext_mainline(long_movetext).sans;

    // Check if short is a prefix of long
    if short_moves.len() > long_moves.len() {
        return false;
    }

    short_moves
        .iter()
        .zip(long_moves.iter())
        .all(|(s, l)| s == l)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

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
    fn test_chess_moves_hash_consistency_formatting() {
        // Test identical moves with different formatting produce same hash
        let hash1 = compute_hash("1. e4 e5");
        let hash2 = compute_hash("1.e4 e5");
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_chess_moves_hash_consistency_comments() {
        // Test identical moves with comments produce same hash
        let hash1 = compute_hash("1. e4 e5");
        let hash2 = compute_hash("1. e4 {comment} e5");
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_chess_moves_hash_consistency_variations() {
        // Test identical moves with variations produce same hash
        let hash1 = compute_hash("1. e4 e5");
        let hash2 = compute_hash("1. e4 (1. d4) e5");
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_chess_moves_hash_consistency_nags() {
        // Test identical moves with NAGs produce same hash
        let hash1 = compute_hash("1. e4 e5");
        let hash2 = compute_hash("1. e4! e5?");
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_chess_moves_hash_discrimination_different_moves() {
        // Test different moves produce different hashes
        let hash1 = compute_hash("1. e4 e5");
        let hash2 = compute_hash("1. d4 d5");
        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_chess_moves_hash_discrimination_different_length() {
        // Test different length sequences produce different hashes
        let hash1 = compute_hash("1. e4");
        let hash2 = compute_hash("1. e4 e5");
        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_chess_moves_hash_empty_string() {
        // Test hash of empty string
        let hash = compute_hash("");
        // Known hash value for empty string
        assert_eq!(hash, 3476900567878811119);
    }

    // Helper function to compute hash like the scalar function does
    fn compute_hash(movetext: &str) -> i64 {
        let canonical = parse_movetext_mainline(movetext).sans.join(" ");
        let mut hasher = DefaultHasher::new();
        canonical.hash(&mut hasher);
        hasher.finish() as i64
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
}
