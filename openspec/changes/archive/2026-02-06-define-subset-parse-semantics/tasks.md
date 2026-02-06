## 1. Tests First

- [x] 1.1 Add SQLLogicTest cases in `test/sql/chess_moves_subset.test` for invalid non-empty inputs (`short`, `long`, and `both`) expecting `FALSE`
- [x] 1.2 Add/adjust unit tests in `src/chess/moves.rs` for invalid non-empty cases while preserving existing empty-string behavior
- [x] 1.3 Keep/assert NULL propagation behavior in SQL tests (`NULL` input returns `NULL`)

## 2. Subset Semantics Implementation

- [x] 2.1 Update `check_moves_subset` in `src/chess/moves.rs` to return `FALSE` when either non-empty input has `parse_error = true`
- [x] 2.2 Preserve prefix comparison behavior for valid parsed inputs and empty-string inputs

## 3. Verification

- [x] 3.1 Run `make check` and fix any fmt/clippy issues
- [x] 3.2 Run `make test-rs` and ensure unit + SQLLogicTests pass
