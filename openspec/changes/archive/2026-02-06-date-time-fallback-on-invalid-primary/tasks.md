## 1. Add Mixed-Validity Test Coverage

- [x] 1.1 Add unit tests in `src/chess/reader.rs` for invalid `UTCDate` with valid `Date`/`EventDate` fallback recovery
- [x] 1.2 Add unit tests in `src/chess/reader.rs` for invalid `UTCTime` with valid `Time` fallback recovery
- [x] 1.3 Add tests that preserve precedence/completeness behavior among parseable date candidates (including partial-date rules)
- [x] 1.4 Add SQLLogicTests (new or existing `test/sql/read_pgn_*.test`) for mixed-validity fallback and `parse_error` retention

## 2. Implement Parse-Aware Fallback Selection

- [x] 2.1 Update date candidate selection in `src/chess/visitor.rs` to choose the best parseable candidate across `UTCDate` -> `Date` -> `EventDate`
- [x] 2.2 Update time candidate selection in `src/chess/visitor.rs` to fallback from invalid `UTCTime` to parseable `Time`
- [x] 2.3 Ensure conversion failures for rejected invalid candidates are still appended to `parse_error` even when fallback succeeds
- [x] 2.4 Keep typed output as `NULL` only when no candidate for that field converts successfully

## 3. Validate End-to-End Behavior

- [x] 3.1 Run `make check` and fix any formatting/clippy issues
- [x] 3.2 Run `make test-rs` and ensure all unit + SQLLogicTests pass
