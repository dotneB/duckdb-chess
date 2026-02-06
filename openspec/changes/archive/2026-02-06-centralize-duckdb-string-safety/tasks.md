## 1. Shared Helper Refactor

- [x] 1.1 Add a shared DuckDB string decoding helper module under `src/chess/` with explicit `SAFETY` documentation
- [x] 1.2 Replace duplicated `read_duckdb_string` implementations in `src/chess/filter.rs` and `src/chess/moves.rs` with the shared helper
- [x] 1.3 Remove old duplicated helper functions and update imports/usages

## 2. Null-Safety and Behavior Parity

- [x] 2.1 Verify scalar invoke paths still perform NULL-row checks before any unsafe string decoding
- [x] 2.2 Add or adjust unit tests to cover NULL handling and ensure unchanged behavior for affected scalar functions
- [x] 2.3 Ensure no SQL-visible behavior changes in existing SQLLogicTests for `chess_moves_*`

## 3. Validation

- [x] 3.1 Run `make check` and fix any formatting or clippy issues
- [x] 3.2 Run `make test-rs` and ensure unit + SQLLogicTests pass
