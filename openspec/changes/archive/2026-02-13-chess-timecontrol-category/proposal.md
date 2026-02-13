## Why

TimeControl normalization exists, but users still need extra SQL logic to derive game speed buckets. Adding first-class categorization based on Lichess definitions makes downstream analysis simpler and consistent with widely used chess tooling.

## What Changes

- Add a new scalar function `chess_timecontrol_category(time_control)` that classifies a game as `ultra-bullet`, `bullet`, `blitz`, `rapid`, or `classical` using Lichess thresholds.
- Reuse existing TimeControl normalization/parsing logic so raw values like `3+2` and normalized values like `180+2` produce the same category.
- Define deterministic behavior for unsupported or non-comparable TimeControl modes (for example `?`, `-`, malformed text, or non-normal stage formats) with safe NULL output.
- Preserve existing small-base shorthand interpretation (`N+I` as minutes in ambiguous cases), and document how to express explicit seconds where needed.
- Add unit tests and SQLLogicTests covering thresholds, increment-driven boundary cases, and invalid inputs.

## Capabilities

### New Capabilities

- None.

### Modified Capabilities

- `timecontrol-normalization`: extend normalized TimeControl utilities with category derivation based on estimated game duration.

## Impact

- Affected code: `src/chess/timecontrol.rs`, `src/chess/mod.rs`, and SQL function registration.
- Affected tests: Rust unit tests for classification and `test/sql/*.test` for SQL-visible behavior.
- Public API: new scalar SQL function `chess_timecontrol_category`.
