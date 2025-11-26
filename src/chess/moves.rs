use duckdb::{
    core::{DataChunkHandle, Inserter, LogicalTypeHandle, LogicalTypeId},
    vscalar::{ScalarFunctionSignature, VScalar},
    vtab::arrow::WritableVector,
    Result,
};
use libduckdb_sys::duckdb_string_t;
use shakmaty::{Chess, Position, san::SanPlus, EnPassantMode};
use std::error::Error;
use std::ffi::CString;
use std::fmt::Write;

use crate::chess::filter::filter_movetext_annotations;

pub struct MovesJsonScalar;

impl VScalar for MovesJsonScalar {
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
    let clean_text = filter_movetext_annotations(movetext);
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
