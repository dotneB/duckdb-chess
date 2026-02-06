## Why

`chess_moves_subset` currently treats some invalid non-empty movetext values as empty move lists, which can produce surprising `TRUE` results and false positives in deduplication workflows. We need explicit, stable parse-failure semantics so users can trust subset checks on messy real-world data.

## What Changes

- Define explicit `chess_moves_subset` behavior for invalid non-empty movetext.
- Set parse-failure semantics to return `FALSE` when either non-NULL input is non-empty and cannot be parsed as movetext.
- Preserve intentional existing behavior for empty-string cases and DuckDB NULL propagation.
- Update implementation and tests (Rust + SQLLogicTest) for invalid short/long/both-invalid input combinations.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `move-analysis`: refine `chess_moves_subset` requirements to define parse-failure semantics for invalid non-empty movetext while preserving empty and NULL behavior.

## Impact

- Affected code: `src/chess/moves.rs` (`check_moves_subset` / scalar behavior).
- Affected tests: `src/chess/moves.rs` unit tests and `test/sql/chess_moves_subset.test`.
- User-visible effect: fewer false positives for malformed inputs; deterministic subset semantics.
