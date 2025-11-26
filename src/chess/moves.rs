use duckdb::{
    core::{DataChunkHandle, Inserter, LogicalTypeHandle, LogicalTypeId},
    vscalar::{ScalarFunctionSignature, VScalar},
    vtab::arrow::WritableVector,
    Result,
};
use libduckdb_sys::duckdb_string_t;
use shakmaty::{Chess, Position, san::SanPlus, EnPassantMode};
use std::collections::hash_map::DefaultHasher;
use std::error::Error;
use std::ffi::CString;
use std::fmt::Write;
use std::hash::{Hash, Hasher};

use crate::chess::filter::normalize_movetext;

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

        let input_slice = input_vec.as_slice::<duckdb_string_t>();

        for i in 0..len {
            let val = read_duckdb_string(input_slice[i]);
            match process_moves(&val) {
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
        vec![ScalarFunctionSignature::exact(
            vec![LogicalTypeHandle::from(LogicalTypeId::Varchar)],
            LogicalTypeHandle::from(LogicalTypeId::Varchar),
        )]
    }
}

fn process_moves(movetext: &str) -> Result<String, Box<dyn Error>> {
    let clean_text = normalize_movetext(movetext);
    let mut pos = Chess::default();
    let mut json = String::new();
    
    json.push('[');
    
    let mut first = true;
    let mut ply = 0;

    for token in clean_text.split_whitespace() {
        if token.ends_with('.') || token.contains("...") {
            continue;
        }

        if token == "1-0" || token == "0-1" || token == "1/2-1/2" || token == "*" {
            continue;
        }

        let san: SanPlus = match token.parse() {
            Ok(s) => s,
            Err(_) => {
                break;
            }
        };

        let m = san.san.to_move(&pos)?;
        pos.play_unchecked(m);
        ply += 1;

        if !first {
            json.push(',');
        }
        first = false;

        let fen = duckdb_fen(&pos);
        
        write!(json, r#"{{"ply":{},"move":"{}","fen":"{}"}}"#, ply, token, fen)?;
    }

    json.push(']');
    Ok(json)
}

fn duckdb_fen(pos: &Chess) -> String {
    use shakmaty::fen::Fen;
    
    let fen = Fen::from_position(pos, EnPassantMode::Legal);
    fen.to_string()
}

unsafe fn read_duckdb_string(s: duckdb_string_t) -> String {
    if s.value.inlined.length <= 12 {
        let len = s.value.inlined.length as usize;
        let slice = &s.value.inlined.inlined;
        let slice_u8 = std::slice::from_raw_parts(slice.as_ptr() as *const u8, len);
        String::from_utf8_lossy(slice_u8).into_owned()
    } else {
        let len = s.value.pointer.length as usize;
        let ptr = s.value.pointer.ptr;
        let slice = std::slice::from_raw_parts(ptr as *const u8, len);
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

        for i in 0..len {
            let val = read_duckdb_string(input_slice[i]);
            let normalized = normalize_movetext(&val);
            
            // Compute hash
            let mut hasher = DefaultHasher::new();
            normalized.hash(&mut hasher);
            output_slice[i] = hasher.finish() as i64;
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

        for i in 0..len {
            let short_text = read_duckdb_string(input_slice_0[i]);
            let long_text = read_duckdb_string(input_slice_1[i]);
            
            output_slice[i] = check_moves_subset(&short_text, &long_text);
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
    let short_normalized = normalize_movetext(short_movetext);
    let long_normalized = normalize_movetext(long_movetext);
    
    // Extract just the moves (without move numbers)
    let short_moves = extract_moves(&short_normalized);
    let long_moves = extract_moves(&long_normalized);
    
    // Check if short is a prefix of long
    if short_moves.len() > long_moves.len() {
        return false;
    }
    
    short_moves.iter()
        .zip(long_moves.iter())
        .all(|(s, l)| s == l)
}

fn extract_moves(movetext: &str) -> Vec<String> {
    movetext
        .split_whitespace()
        .filter(|token| {
            // Skip move numbers (e.g., "1.", "2.", "1...")
            !token.ends_with('.') && !token.contains("...")
                // Skip result markers
                && *token != "1-0" && *token != "0-1" && *token != "1/2-1/2" && *token != "*"
        })
        .map(|s| s.to_string())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process_moves_basic() {
        let input = "1. e4 e5";
        let json = process_moves(input).unwrap();
        // Check structure roughly
        assert!(json.starts_with('['));
        assert!(json.ends_with(']'));
        assert!(json.contains(r#""ply":1,"move":"e4""#));
        assert!(json.contains(r#""ply":2,"move":"e5""#));
    }

    #[test]
    fn test_process_moves_with_annotations() {
        let input = "1. e4 {comment} e5";
        let json = process_moves(input).unwrap();
        assert!(json.contains(r#""move":"e5""#));
    }
}
