## 1. Spec & API Finalization

- [x] 1.1 Confirm final SQL function names and return types match `openspec/changes/timecontrol-parsing/specs/timecontrol-normalization/spec.md`
- [x] 1.2 Decide warning code set (stable strings) and JSON keys for `chess_timecontrol_json`

## 2. Core Parser Module

- [x] 2.1 Add pure Rust parser module (e.g. `src/chess/timecontrol.rs`) with data types (`ParsedTimeControl`, `Period`, `Mode`, error)
- [x] 2.2 Implement strict spec-shaped token parser (`?`, `-`, `*secs`, `secs[+inc]`, `moves/secs[+inc]`, `:` multi-stage)
- [x] 2.3 Implement pre-normalization transforms (trim, strip quotes, operator whitespace, `|`/`_` mapping, trailing apostrophe stripping) with warnings
- [x] 2.4 Implement inference rules for minute shorthand per spec (`N+I` with `N<60`, `N in {75,90} with I=30`, bare `N<60`, apostrophe units)
- [x] 2.5 Implement limited free-text templates for dominant patterns (FIDE classical description; "X minutes + Y seconds per move")
- [x] 2.6 Implement canonical string emitter for parsed periods and modes

## 3. DuckDB Scalar Functions

- [x] 3.1 Add `VScalar` for `chess_timecontrol_normalize` (VARCHAR -> VARCHAR), returning NULL on parse failure
- [x] 3.2 Add `VScalar` for `chess_timecontrol_json` (VARCHAR -> VARCHAR), returning JSON with `raw`, `normalized`, `mode`, `periods`, `warnings`, `inferred`
- [x] 3.3 Register the new scalars in `src/chess/mod.rs` and ensure naming follows `chess_*` convention

## 4. Tests

- [x] 4.1 Add Rust unit tests covering strict parsing and canonicalization
- [x] 4.2 Add Rust unit tests covering lenient inference cases from `timecontrolfreq.csv` (e.g. `3+2`, `15+10`, `15 + 10`, `75 | 30`, `10'+5''`, quoted values)
- [x] 4.3 Add Rust unit tests for failure cases (e.g. `klassisch`, ambiguous/unsupported free text)
- [x] 4.4 Add SQLLogicTests in `test/sql/` for NULL handling, representative normalizations, and JSON shape expectations

## 5. Docs & Validation

- [x] 5.1 Update `README.md` with the new `chess_timecontrol_*` functions and examples
- [x] 5.2 Run `just dev` and fix any failures (fmt, clippy, unit tests, SQLLogicTests)
