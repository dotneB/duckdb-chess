## 1. Baseline and Safety Nets

- [x] 1.1 Add/extend unit tests that lock current behavior for `chess_moves_json`, `chess_moves_normalize`, `chess_ply_count`, and `chess_moves_hash` on NULL, empty, annotated, and malformed inputs
- [x] 1.2 Add/extend PGN visitor tests that lock current header extraction, movetext formatting, comment formatting, and malformed-game continuation behavior
- [x] 1.3 Record a local baseline for allocation-sensitive workloads (representative fixtures + command notes) using project-native timing/profiling approaches (non-Criterion)

## 2. Borrow-First String Decoding

- [x] 2.1 Refactor `src/chess/duckdb_string.rs` to return a borrow-first representation (`Cow<str>`) with explicit `SAFETY` contract maintained
- [x] 2.2 Update scalar call sites in `src/chess/filter.rs` and `src/chess/moves.rs` to consume borrowed strings where possible and allocate only at ownership boundaries
- [x] 2.3 Ensure invalid UTF-8 decoding behavior remains resilient and covered by tests

## 3. Move-Analysis Hot Path Refactors

- [x] 3.1 Rework `chess_ply_count` to use streaming counting logic without intermediate `Vec<String>` allocation
- [x] 3.2 Rework `chess_moves_normalize` to build canonical output during visitation without intermediate move-vector materialization
- [x] 3.3 Rework `chess_moves_json` to use visitor-driven move application and early-stop behavior for `max_ply` while preserving existing output contract

## 4. PGN Visitor Allocation Refactors

- [x] 4.1 Refactor `GameVisitor::tag()` to populate dedicated known-header fields directly instead of relying on repeated scans/clones over a header vector
- [x] 4.2 Replace transient `format!`-heavy movetext assembly with append-oriented building (`write!`/`push_str`) and light preallocation
- [x] 4.3 Keep malformed-game and `parse_error` semantics unchanged while validating parity with existing fixtures

## 5. Reader Integration and Hardening

- [x] 5.1 Add an internal helper for repeated `Option<&str>` DuckDB vector insertion patterns in `src/chess/reader.rs` to reduce duplication and reduce mistake risk
- [x] 5.2 Decide and implement interior-NUL handling policy for string writes (sanitize + `parse_error` annotation or explicit deferred follow-up)

## 6. Verification and Finalization

- [x] 6.1 Run `just dev` and fix any regressions
- [x] 6.2 Run `just full` (or `just test-release` when time-constrained) and confirm debug/release parity
- [x] 6.3 Update README/OpenSpec notes if any user-visible guidance changed, then mark artifact checklist complete
