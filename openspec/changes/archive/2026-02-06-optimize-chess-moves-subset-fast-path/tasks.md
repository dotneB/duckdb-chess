## 1. Test Coverage for Fast-Path Equivalence

- [x] 1.1 Add Rust unit tests in `src/chess/moves.rs` for clean canonical inputs asserting fast-path-equivalent subset outcomes
- [x] 1.2 Add Rust unit tests for result-marker-insensitive prefix behavior (`1-0`, `0-1`, `1/2-1/2`, `*`)
- [x] 1.3 Add Rust unit tests for uncertain/annotated inputs to ensure parser fallback preserves existing behavior
- [x] 1.4 Add SQLLogicTest cases in `test/sql/chess_moves_subset.test` for clean fast-path candidates and dirty fallback cases
- [x] 1.5 Keep/assert existing NULL propagation behavior in SQL tests

## 2. Implement Conservative Fast Path

- [x] 2.1 Add a conservative clean-input detector in `src/chess/moves.rs` that rejects comments, variations, NAGs, and ambiguous syntax
- [x] 2.2 Add lightweight token extraction/comparison helpers that ignore move numbers and trailing result markers
- [x] 2.3 Update `check_moves_subset` to attempt fast path only when both inputs are clean; otherwise fall back to `parse_movetext_mainline`
- [x] 2.4 Ensure behavior remains semantically equivalent to parser-based comparison for valid and uncertain inputs

## 3. Verify and Stabilize

- [x] 3.1 Run `make check` and fix any formatting or clippy issues
- [x] 3.2 Run `make test-rs` and ensure all Rust + SQLLogicTests pass
