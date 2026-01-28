use duckdb::{
    core::{DataChunkHandle, Inserter, LogicalTypeHandle, LogicalTypeId},
    vscalar::{ScalarFunctionSignature, VScalar},
    vtab::arrow::WritableVector,
};
use libduckdb_sys::duckdb_string_t;
use std::error::Error;
use std::ffi::CString;

/// Normalize chess movetext by removing comments {}, variations (), and NAGs ($n, !, ?, etc.)
/// Returns a canonical string representation with standardized spacing.
///
/// Note: Move numbers (e.g., `1.`, `12.`, `1...`) are preserved, but only when they precede
/// a subsequent non-move-number token.
/// Spec: move-analysis - Moves Normalization
pub fn normalize_movetext(movetext: &str) -> String {
    // Preprocess: add space after move numbers like "1.e4" -> "1. e4"
    let preprocessed = preprocess_move_numbers(movetext);

    let mut result = String::new();
    let mut in_comment = false;
    let mut in_variation = false;
    let mut comment_depth = 0;
    let mut variation_depth = 0;
    let mut prev_was_space = false;
    let mut buffer = String::new();
    let mut pending_move_number: Option<String> = None;

    fn push_token(out: &mut String, token: &str, prev_was_space: &mut bool) {
        if token.is_empty() {
            return;
        }
        if !out.is_empty() && !*prev_was_space {
            out.push(' ');
        }
        out.push_str(token);
        *prev_was_space = false;
    }

    for ch in preprocessed.chars() {
        match ch {
            '{' => {
                in_comment = true;
                comment_depth += 1;
            }
            '}' => {
                comment_depth -= 1;
                if comment_depth == 0 {
                    in_comment = false;
                    prev_was_space = true;
                }
            }
            '(' => {
                in_variation = true;
                variation_depth += 1;
            }
            ')' => {
                variation_depth -= 1;
                if variation_depth == 0 {
                    in_variation = false;
                    prev_was_space = true;
                }
            }
            ' ' if !in_comment && !in_variation => {
                if !buffer.is_empty() {
                    // Check if buffer contains NAG symbols to strip
                    let cleaned = strip_nags(&buffer);
                    if !cleaned.is_empty() {
                        if is_move_number(&cleaned) {
                            // Only emit move numbers if they're followed by a move/result token.
                            let _ = pending_move_number.take();
                            pending_move_number = Some(cleaned);
                        } else {
                            if let Some(mn) = pending_move_number.take() {
                                push_token(&mut result, &mn, &mut prev_was_space);
                            }
                            push_token(&mut result, &cleaned, &mut prev_was_space);
                        }
                    }
                    buffer.clear();
                }
                prev_was_space = true;
            }
            _ if !in_comment && !in_variation => {
                buffer.push(ch);
                prev_was_space = false;
            }
            _ => {}
        }
    }

    // Process remaining buffer
    if !buffer.is_empty() {
        let cleaned = strip_nags(&buffer);
        if !cleaned.is_empty() {
            if is_move_number(&cleaned) {
                // Trailing move number with no following token; drop it.
            } else {
                if let Some(mn) = pending_move_number.take() {
                    push_token(&mut result, &mn, &mut prev_was_space);
                }
                push_token(&mut result, &cleaned, &mut prev_was_space);
            }
        }
    }

    result.trim().to_string()
}

/// Check if a token is a move number (e.g., "1.", "12.", "1...")
pub(crate) fn is_move_number(token: &str) -> bool {
    let token = token.trim();
    if token.is_empty() {
        return false;
    }

    let bytes = token.as_bytes();
    let mut i = 0;
    while i < bytes.len() && bytes[i].is_ascii_digit() {
        i += 1;
    }
    if i == 0 {
        return false;
    }

    let mut j = i;
    while j < bytes.len() && bytes[j] == b'.' {
        j += 1;
    }

    let dot_count = j - i;
    (dot_count == 1 || dot_count == 3) && j == bytes.len()
}

/// Preprocess movetext to ensure space after move numbers
/// Converts "1.e4" to "1. e4"
pub fn preprocess_move_numbers(movetext: &str) -> String {
    let mut result = String::new();
    let mut chars = movetext.chars().peekable();

    while let Some(&ch) = chars.peek() {
        if ch.is_ascii_digit() {
            // Capture the full move number (digits)
            while let Some(&d) = chars.peek()
                && d.is_ascii_digit()
            {
                result.push(d);
                chars.next();
            }

            // Capture following dots ('.' or '...')
            if let Some(&'.') = chars.peek() {
                let mut dot_count = 0usize;
                while let Some(&'.') = chars.peek() {
                    result.push('.');
                    chars.next();
                    dot_count += 1;
                }

                // Add a space after a full move number token when immediately followed by a move.
                // - "1.e4"   -> "1. e4"
                // - "1...e5" -> "1... e5"
                if (dot_count == 1 || dot_count == 3)
                    && chars.peek().is_some_and(|c| !c.is_whitespace())
                {
                    result.push(' ');
                }
            }
        } else {
            result.push(ch);
            chars.next();
        }
    }

    result
}

/// Strip NAG symbols from a token
/// NAGs include: !, ?, !!, ??, !?, ?!, $1, $2, etc.
fn strip_nags(token: &str) -> String {
    let mut result = String::new();
    let mut chars = token.chars().peekable();

    while let Some(ch) = chars.next() {
        match ch {
            '$' => {
                // Skip $ and following digits
                while chars.peek().is_some_and(|c| c.is_ascii_digit()) {
                    chars.next();
                }
            }
            '!' | '?' => {
                // Skip NAG symbols (!, ??, !!, ?!, !?)
                // Continue skipping consecutive ! and ?
                while chars.peek().is_some_and(|c| *c == '!' || *c == '?') {
                    chars.next();
                }
            }
            _ => result.push(ch),
        }
    }

    result
}

pub struct ChessMovesNormalizeScalar;

impl VScalar for ChessMovesNormalizeScalar {
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

        for (i, s) in input_slice.iter().take(len).enumerate() {
            let val = unsafe { read_duckdb_string(*s) };
            let normalized = normalize_movetext(&val);
            output_vec.insert(i, CString::new(normalized)?);
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_complex() {
        let input = "1. e4! {Best by test} (1. d4 d5) e5?? $1 2. Nf3";
        assert_eq!(normalize_movetext(input), "1. e4 e5 2. Nf3");
    }

    #[test]
    fn test_normalize_nag_symbols() {
        assert_eq!(normalize_movetext("1. e4!"), "1. e4");
        assert_eq!(normalize_movetext("1. e4?"), "1. e4");
        assert_eq!(normalize_movetext("1. e4!!"), "1. e4");
        assert_eq!(normalize_movetext("1. e4??"), "1. e4");
        assert_eq!(normalize_movetext("1. e4!?"), "1. e4");
        assert_eq!(normalize_movetext("1. e4?!"), "1. e4");
        assert_eq!(normalize_movetext("1. e4$1"), "1. e4");
        assert_eq!(normalize_movetext("1. e4$10"), "1. e4");
    }

    #[test]
    fn test_normalize_empty() {
        assert_eq!(normalize_movetext(""), "");
    }

    #[test]
    fn test_strip_nags() {
        assert_eq!(strip_nags("e4!"), "e4");
        assert_eq!(strip_nags("e4?"), "e4");
        assert_eq!(strip_nags("e4!!"), "e4");
        assert_eq!(strip_nags("e4$1"), "e4");
        assert_eq!(strip_nags("Nf3+"), "Nf3+");
        assert_eq!(strip_nags("O-O"), "O-O");
    }

    #[test]
    fn test_normalize_with_different_spacing() {
        assert_eq!(
            normalize_movetext("1. e4 e5"),
            normalize_movetext("1.e4 e5")
        );
        assert_eq!(
            normalize_movetext("1. e4 e5"),
            normalize_movetext("1.  e4  e5")
        );
    }

    // Tests from SQL test files

    #[test]
    fn test_normalize_annotation_at_start() {
        let input = "{ opening comment } 1. e4 e5";
        assert_eq!(normalize_movetext(input), "1. e4 e5");
    }

    #[test]
    fn test_normalize_annotation_at_end() {
        let input = "1. e4 e5 { game ends }";
        assert_eq!(normalize_movetext(input), "1. e4 e5");
    }

    #[test]
    fn test_normalize_move_structure_preservation() {
        let input = "1. e4 { pawn } e5 { pawn } 2. Nf3+ { check } Nc6";
        assert_eq!(normalize_movetext(input), "1. e4 e5 2. Nf3+ Nc6");
    }

    #[test]
    fn test_normalize_result_markers_preserved() {
        let input = "1. e4 e5 2. Qh5 Nc6 3. Qxf7# { checkmate } 1-0";
        assert_eq!(
            normalize_movetext(input),
            "1. e4 e5 2. Qh5 Nc6 3. Qxf7# 1-0"
        );
    }

    #[test]
    fn test_normalize_lichess_style_annotations() {
        let input = "1. d4 { [%eval 0.25] [%clk 1:30:43] } Nf6 { [%eval 0.22] [%clk 1:30:42] }";
        let result = normalize_movetext(input);
        assert!(!result.contains('{'));
        assert_eq!(result, "1. d4 Nf6");
    }

    #[test]
    fn test_normalize_variations_removed() {
        let input = "1. e4 (1. d4) e5";
        assert_eq!(normalize_movetext(input), "1. e4 e5");
    }

    #[test]
    fn test_normalize_nag_symbols_removed() {
        let input = "1. e4! e5?";
        assert_eq!(normalize_movetext(input), "1. e4 e5");
    }

    #[test]
    fn test_normalize_numeric_nags_removed() {
        let input = "1. e4$1 e5$2";
        assert_eq!(normalize_movetext(input), "1. e4 e5");
    }

    #[test]
    fn test_normalize_complex_all_features() {
        let input = "1. e4! {Best by test} (1. d4 d5) e5?? $1 2. Nf3";
        assert_eq!(normalize_movetext(input), "1. e4 e5 2. Nf3");
    }

    #[test]
    fn test_normalize_complex_annotation_with_multiple_levels() {
        let input = "1. d4 { [%eval 0.25] } d5 { [%clk 1:30:00] } 2. c4 { best move } e6";
        assert_eq!(normalize_movetext(input), "1. d4 d5 2. c4 e6");
    }

    #[test]
    fn test_normalize_lichess_style_no_annotations() {
        let input = "1. d4 { [%eval 0.25] [%clk 1:30:43] } Nf6 { [%eval 0.22] [%clk 1:30:42] }";
        assert!(normalize_movetext(input).contains("d4"));
        assert!(normalize_movetext(input).contains("Nf6"));
        assert!(!normalize_movetext(input).contains('{'));
    }

    // Tests for preprocessing move numbers

    #[test]
    fn test_preprocess_move_numbers_basic() {
        assert_eq!(preprocess_move_numbers("1.e4"), "1. e4");
        assert_eq!(preprocess_move_numbers("10.e5"), "10. e5");
    }

    #[test]
    fn test_preprocess_move_numbers_with_existing_space() {
        assert_eq!(preprocess_move_numbers("1. e4"), "1. e4");
        assert_eq!(preprocess_move_numbers("1.  e4"), "1.  e4");
    }

    #[test]
    fn test_preprocess_move_numbers_ellipses() {
        assert_eq!(preprocess_move_numbers("1...e4"), "1... e4");
        assert_eq!(preprocess_move_numbers("10...e5"), "10... e5");
    }

    #[test]
    fn test_preprocess_move_numbers_mixed() {
        assert_eq!(preprocess_move_numbers("1.e4 e5 2.Nf3"), "1. e4 e5 2. Nf3");
    }

    // Additional edge case tests

    #[test]
    fn test_normalize_deeply_nested_variations() {
        let input = "1. e4 ((1. d4 (1. c4)) e5";
        let result = normalize_movetext(input);
        // The current implementation stops parsing when encountering malformed nested variations
        // This is actually reasonable behavior for robustness
        assert_eq!(result, "1. e4");
    }

    #[test]
    fn test_normalize_mixed_annotations_and_variations() {
        let input = "1. e4 {comment} (1. d4 {alternative}) e5";
        assert_eq!(normalize_movetext(input), "1. e4 e5");
    }

    #[test]
    fn test_normalize_empty_with_whitespace() {
        assert_eq!(normalize_movetext("   "), "");
        assert_eq!(normalize_movetext("\t\n"), "");
        assert_eq!(normalize_movetext(""), "");
    }

    #[test]
    fn test_normalize_only_annotations() {
        assert_eq!(normalize_movetext("{comment}"), "");
        assert_eq!(normalize_movetext("{comment} {another}"), "");
        assert_eq!(normalize_movetext("(variation)"), "");
    }

    #[test]
    fn test_normalize_only_move_numbers() {
        assert_eq!(normalize_movetext("1."), "");
        assert_eq!(normalize_movetext("1. 2. 3."), "");
        assert_eq!(normalize_movetext("1... 2..."), "");
    }

    #[test]
    fn test_normalize_castling_with_symbols() {
        assert_eq!(normalize_movetext("1. e4 O-O"), "1. e4 O-O");
        assert_eq!(normalize_movetext("1. e4 O-O+"), "1. e4 O-O+");
        assert_eq!(normalize_movetext("1. e4 O-O-O"), "1. e4 O-O-O");
        assert_eq!(normalize_movetext("1. e4 O-O-O+"), "1. e4 O-O-O+");
    }

    #[test]
    fn test_normalize_complex_move_notation() {
        assert_eq!(
            normalize_movetext("1. e4 e5 2. Nf3+ Nc6 3. Bb5 a6 4. Ba4 Nf6"),
            "1. e4 e5 2. Nf3+ Nc6 3. Bb5 a6 4. Ba4 Nf6"
        );
    }

    #[test]
    fn test_normalize_with_all_nag_variants() {
        let input = "1. e4!! e5?? Nf3!? Nc6?! $1 $2";
        assert_eq!(normalize_movetext(input), "1. e4 e5 Nf3 Nc6");
    }
}
