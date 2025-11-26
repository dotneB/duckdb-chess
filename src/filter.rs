use duckdb::{
    core::{DataChunkHandle, Inserter, LogicalTypeHandle, LogicalTypeId},
    vscalar::{ScalarFunctionSignature, VScalar},
    vtab::arrow::WritableVector,
};
use libduckdb_sys::duckdb_string_t;
use std::ffi::CString;
use std::error::Error;

/// Spec: annotation-filtering - Movetext Annotation Removal
/// Removes curly brace annotations from chess movetext while preserving move structure
/// Spec: annotation-filtering - Nested Annotation Handling (tracks brace depth)
/// Spec: annotation-filtering - Whitespace Normalization (collapses spaces and trims)
pub fn filter_movetext_annotations(movetext: &str) -> String {
    let mut result = String::new();
    let mut in_annotation = false;
    let mut brace_depth = 0;
    let mut prev_was_space = false;

    for ch in movetext.chars() {
        match ch {
            '{' => {
                in_annotation = true;
                brace_depth += 1;
            }
            '}' => {
                brace_depth -= 1;
                if brace_depth == 0 {
                    in_annotation = false;
                    // Mark that we should skip next space if any
                    prev_was_space = true;
                }
            }
            ' ' if !in_annotation => {
                if !prev_was_space && !result.is_empty() {
                    result.push(' ');
                    prev_was_space = true;
                }
            }
            _ if !in_annotation => {
                prev_was_space = false;
                result.push(ch);
            }
            _ => {}
        }
    }

    result.trim().to_string()
}

pub struct FilterMovetextScalar;

impl VScalar for FilterMovetextScalar {
    type State = ();

    unsafe fn invoke(
        _state: &Self::State,
        input: &mut DataChunkHandle,
        output: &mut dyn WritableVector,
    ) -> Result<(), Box<dyn Error>> {
        let len = input.len();
        let input_vec = input.flat_vector(0);
        let output_vec = output.flat_vector();

        // Get raw slice of duckdb_string_t
        let input_slice = input_vec.as_slice::<duckdb_string_t>();

        for i in 0..len {
            let val = read_duckdb_string(input_slice[i]);
            let filtered = filter_movetext_annotations(&val);
            output_vec.insert(i, CString::new(filtered)?);
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
    fn test_filter_simple_annotation() {
        let input = "1. e4 { comment } e5";
        assert_eq!(filter_movetext_annotations(input), "1. e4 e5");
    }

    #[test]
    fn test_filter_multiple_annotations() {
        let input = "1. e4 { first } e5 { second } 2. Nf3 { third }";
        assert_eq!(filter_movetext_annotations(input), "1. e4 e5 2. Nf3");
    }

    #[test]
    fn test_filter_no_annotations() {
        let input = "1. e4 e5 2. Nf3 Nc6";
        assert_eq!(filter_movetext_annotations(input), "1. e4 e5 2. Nf3 Nc6");
    }

    #[test]
    fn test_filter_nested_annotations() {
        let input = "1. e4 { outer { inner } text } e5";
        assert_eq!(filter_movetext_annotations(input), "1. e4 e5");
    }

    #[test]
    fn test_filter_whitespace_normalization() {
        let input = "1. e4   { comment }   e5";
        assert_eq!(filter_movetext_annotations(input), "1. e4 e5");
    }

    #[test]
    fn test_filter_empty_string() {
        assert_eq!(filter_movetext_annotations(""), "");
    }

    #[test]
    fn test_filter_leading_trailing_whitespace() {
        let input = "  1. e4 e5  ";
        assert_eq!(filter_movetext_annotations(input), "1. e4 e5");
    }
}
