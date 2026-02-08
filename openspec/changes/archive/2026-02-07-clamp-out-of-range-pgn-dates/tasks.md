## 1. Date Conversion Normalization

- [x] 1.1 Add a helper in `src/chess/visitor.rs` to compute the last valid day for a parsed year/month and clamp overflow day values before `NaiveDate` construction.
- [x] 1.2 Update `parse_date_field` to apply day-overflow clamping while preserving existing handling for unknown components (`?`), invalid year/month inputs, and fallback precedence.
- [x] 1.3 Ensure successful day clamping does not append conversion errors to `parse_error`, while true conversion failures still do.

## 2. Automated Test Coverage

- [x] 2.1 Extend Rust tests in `src/chess/reader.rs` and/or `src/chess/visitor.rs` to cover out-of-range day inputs in 30-day months and February leap/non-leap boundaries.
- [x] 2.2 Add SQLLogicTest coverage in `test/sql/*.test` (or a new dedicated `.test` file) asserting normalized `UTCDate` results for malformed day values from `UTCDate` and fallback date headers.

## 3. Verification

- [x] 3.1 Run `just test` to validate unit tests and SQLLogicTests in debug mode.
- [x] 3.2 Run `just check` to ensure formatting and clippy pass with `-D warnings`.
