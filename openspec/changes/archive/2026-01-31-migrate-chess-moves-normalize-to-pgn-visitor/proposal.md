## Why

`chess_moves_normalize` currently relies on bespoke string parsing, which is easy to get wrong on real-world PGN movetext (comments, nested braces, variations, NAGs) and tends to drift from the parser behavior used elsewhere in the project. Moving normalization onto a `pgn-reader` visitor-based parse reduces correctness risk and maintenance cost while keeping the SQL API stable.

## What Changes

- Reimplement `chess_moves_normalize(movetext)` using a `pgn-reader` Visitor as the source of truth instead of custom token scanning.
- Preserve user-visible behavior: strip comments, recursive variations, and NAGs; emit a canonical mainline movetext string with standardized spacing.
- **BREAKING**: Remove legacy string-parser fallback; if `pgn-reader` cannot parse the input, `chess_moves_normalize` returns `NULL`/empty output (instead of best-effort normalization).
- Improve handling of edge cases (nested annotation braces, deeply nested variations, odd whitespace) to better satisfy existing specs.
- Add/adjust regression tests (Rust unit tests + SQLLogicTest) to lock in behavior.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `move-analysis`: define parse-failure behavior for `chess_moves_normalize` when movetext cannot be parsed.

## Impact

- Affected code: `src/chess/filter.rs` (implementation of `chess_moves_normalize`) and any shared movetext helpers.
- APIs: no signature changes; output intended to remain compatible.
- Dependencies: increases reliance on existing `pgn-reader` parsing patterns; no new external dependencies expected.
- Tests: new/updated fixtures for tricky PGN inputs (comments/variations/NAGs).
