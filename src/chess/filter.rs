use super::duckdb_string::decode_duckdb_string;
use duckdb::{
    core::{DataChunkHandle, Inserter, LogicalTypeHandle, LogicalTypeId},
    vscalar::{ScalarFunctionSignature, VScalar},
    vtab::arrow::WritableVector,
};
use libduckdb_sys::duckdb_string_t;
use smallvec::SmallVec;
use std::error::Error;
use std::ffi::CString;
use std::fmt::Write;
use std::io;
use std::ops::ControlFlow;

use pgn_reader::{Nag, Outcome, RawComment, Reader, SanPlus, Skip, Visitor};

use crate::pgn_visitor_skip_variations;

type MoveList = SmallVec<[String; 128]>;

/// Normalize chess movetext by removing comments {}, variations (), and NAGs ($n, !, ?, etc.)
/// Returns a canonical string representation with standardized spacing.
/// Spec: move-analysis - Moves Normalization
pub fn normalize_movetext(movetext: &str) -> String {
    if movetext.trim().is_empty() {
        return String::new();
    }

    let mut reader = Reader::new(io::Cursor::new(movetext.as_bytes()));
    let mut visitor = NormalizeSerializeVisitor::default();

    match reader.read_game(&mut visitor) {
        Ok(Some(())) => visitor.output,
        Ok(None) | Err(_) => String::new(),
    }
}

pub(crate) struct ParsedMovetext {
    pub sans: MoveList,
    pub outcome: Option<String>,
    pub parse_error: bool,
}

pub(crate) fn parse_movetext_mainline(movetext: &str) -> ParsedMovetext {
    if movetext.trim().is_empty() {
        return ParsedMovetext {
            sans: MoveList::new(),
            outcome: None,
            parse_error: false,
        };
    }

    let mut reader = Reader::new(io::Cursor::new(movetext.as_bytes()));
    let mut visitor = NormalizeVisitor::default();

    match reader.read_game(&mut visitor) {
        Ok(Some(())) => ParsedMovetext {
            sans: visitor.sans,
            outcome: visitor.outcome,
            parse_error: false,
        },
        Ok(None) => ParsedMovetext {
            sans: visitor.sans,
            outcome: visitor.outcome,
            parse_error: true,
        },
        Err(_) => ParsedMovetext {
            sans: visitor.sans,
            outcome: visitor.outcome,
            parse_error: true,
        },
    }
}

#[derive(Default)]
struct NormalizeSerializeVisitor {
    output: String,
    move_count: usize,
    outcome: Option<String>,
}

impl Visitor for NormalizeSerializeVisitor {
    type Tags = ();
    type Movetext = ();
    type Output = ();

    fn begin_tags(&mut self) -> ControlFlow<Self::Output, Self::Tags> {
        self.output.clear();
        self.move_count = 0;
        self.outcome = None;
        ControlFlow::Continue(())
    }

    fn begin_movetext(&mut self, _tags: Self::Tags) -> ControlFlow<Self::Output, Self::Movetext> {
        ControlFlow::Continue(())
    }

    fn san(
        &mut self,
        _movetext: &mut Self::Movetext,
        san_plus: SanPlus,
    ) -> ControlFlow<Self::Output> {
        if self.move_count.is_multiple_of(2) {
            if !self.output.is_empty() {
                self.output.push(' ');
            }
            let move_no = (self.move_count / 2) + 1;
            let _ = write!(self.output, "{}.", move_no);
            self.output.push(' ');
        } else {
            self.output.push(' ');
        }

        let _ = write!(self.output, "{}", san_plus);
        self.move_count += 1;
        ControlFlow::Continue(())
    }

    pgn_visitor_skip_variations!();

    fn outcome(
        &mut self,
        _movetext: &mut Self::Movetext,
        outcome: Outcome,
    ) -> ControlFlow<Self::Output> {
        self.outcome = Some(outcome.to_string());
        ControlFlow::Continue(())
    }

    fn end_game(&mut self, _movetext: Self::Movetext) -> Self::Output {
        if let Some(outcome) = self.outcome.take() {
            if !self.output.is_empty() {
                self.output.push(' ');
            }
            self.output.push_str(&outcome);
        }
    }
}

#[derive(Default)]
struct NormalizeVisitor {
    sans: MoveList,
    outcome: Option<String>,
}

impl Visitor for NormalizeVisitor {
    type Tags = ();
    type Movetext = ();
    type Output = ();

    fn begin_tags(&mut self) -> ControlFlow<Self::Output, Self::Tags> {
        self.sans.clear();
        self.outcome = None;
        ControlFlow::Continue(())
    }

    fn begin_movetext(&mut self, _tags: Self::Tags) -> ControlFlow<Self::Output, Self::Movetext> {
        ControlFlow::Continue(())
    }

    fn san(
        &mut self,
        _movetext: &mut Self::Movetext,
        san_plus: SanPlus,
    ) -> ControlFlow<Self::Output> {
        self.sans.push(san_plus.to_string());
        ControlFlow::Continue(())
    }

    pgn_visitor_skip_variations!();

    fn outcome(
        &mut self,
        _movetext: &mut Self::Movetext,
        outcome: Outcome,
    ) -> ControlFlow<Self::Output> {
        self.outcome = Some(outcome.to_string());
        ControlFlow::Continue(())
    }

    fn end_game(&mut self, _movetext: Self::Movetext) -> Self::Output {}
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
        let mut output_vec = output.flat_vector();

        let input_slice = input_vec.as_slice::<duckdb_string_t>();

        for (i, s) in input_slice.iter().take(len).enumerate() {
            if input_vec.row_is_null(i as u64) {
                output_vec.set_null(i);
                continue;
            }

            let val = unsafe { decode_duckdb_string(s) };
            let normalized = normalize_movetext(val.as_ref());
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

    // Additional edge case tests

    #[test]
    fn test_normalize_deeply_nested_variations() {
        let input = "1. e4 ((1. d4 (1. c4)) e5";
        let result = normalize_movetext(input);
        // We only keep mainline; malformed/unbalanced variations may cause the
        // remainder of the string to be skipped by the PGN parser.
        assert_eq!(result, "1. e4");
    }

    #[test]
    fn test_normalize_parse_failure_returns_empty_string() {
        assert_eq!(normalize_movetext("this is not movetext"), "");
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
        assert_eq!(normalize_movetext(input), "1. e4 e5 2. Nf3 Nc6");
    }
}
