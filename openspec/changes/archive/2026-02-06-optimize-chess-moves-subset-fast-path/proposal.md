## Why

`chess_moves_subset` currently reparses and normalizes both arguments for every row, even when movetext is already canonical mainline text. This creates unnecessary CPU cost on large datasets and blocks simple early-out behavior while users still need exact subset semantics.

## What Changes

- Add a conservative clean-input fast path for `chess_moves_subset` that avoids full PGN parsing when both inputs are obviously canonical mainline movetext.
- Implement lightweight token-prefix comparison in the fast path, ignoring move numbers and trailing result markers to preserve current subset semantics.
- Keep fallback behavior: if either input is uncertain (comments, variations, NAGs, malformed/ambiguous tokens), use the existing parser-based path.
- Preserve current SQL NULL behavior.
- Add Rust and SQLLogicTest coverage proving fast-path and fallback equivalence.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `move-analysis`: refine `chess_moves_subset` requirements to allow a conservative clean-input fast path with guaranteed equivalence to parser-based results.

## Impact

- Affected code: `src/chess/moves.rs` (`check_moves_subset` and helper parsing/token logic).
- Affected tests: `src/chess/moves.rs` unit tests and `test/sql/chess_moves_subset.test`.
- User-visible behavior: no intentional semantic change; expected improvement in performance for clean movetext inputs.
