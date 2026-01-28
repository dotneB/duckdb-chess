## 1. Implementation
- [x] 1.1 Add `chrono` dependency to `Cargo.toml` (choose minimal feature set that supports `parse_from_str` usage).
- [x] 1.2 Update date parsing helper to:
  - Normalize `.` to `-`
  - Implement header candidate selection over `UTCDate` -> `Date` -> `EventDate`:
    - Prefer fully specified dates; otherwise prefer the most complete partial date (year+month over year-only)
    - Tie-break by header precedence
  - Support `????-??-??` as NULL
  - Default unknown month/day to `01`
  - Use `chrono` for validation
  - Append `chrono` parse errors to `parse_error` on failures
- [x] 1.3 Update time parsing helper to use `chrono` for `HH:MM:SS` validation while preserving offset handling and DuckDB `TIMETZ` packing.
- [x] 1.4 Add/extend Rust unit tests covering:
  - Date candidate selection prefers most complete value (e.g., `Date=1951.??.??`, `EventDate=1951.09.??` => 1951-09-01)
  - Header fallback tie-break by precedence (e.g., `UTCDate=2000.??.??`, `Date=1999.12.31` => 1999-12-31)
  - `????.??.??` => NULL date
  - `2000.??.??` => 2000-01-01
  - `2000.06.??` => 2000-06-01
  - Invalid date/time values => NULL + `parse_error` includes `chrono` error
  - Accepted time variants (`Z`, `+HH:MM`, `-HH:MM`)
- [x] 1.5 Add and run integration tests
- [x] 1.6 Run `cargo test` and ensure all tests pass.


## 2. Validation
- [x] 2.1 Run `make dev` and `make test-release-rs`
- [x] 2.2 Run `openspec validate update-date-time-parsing-chrono --strict --no-interactive`.
