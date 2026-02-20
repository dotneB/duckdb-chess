## 1. Shared Scalar Helper Extraction

- [x] 1.1 Add a shared scalar helper module (e.g., `src/chess/scalar.rs`) and wire it into `src/chess/mod.rs`.
- [x] 1.2 Implement helpers for common scalar shapes (unary/binary `VARCHAR` inputs; optional `BIGINT` arg) covering flat-vector access, row iteration, NULL checks, and `duckdb_string_t` decoding.
- [x] 1.3 Implement helpers for output writing (VARCHAR `CString` insertion and primitive slice assignment) with explicit per-scalar NULL/default policies.

## 2. Centralized Logging Policy

- [x] 2.1 Add a centralized reporting module (e.g., `src/chess/log.rs`) exposing `warn()` / `error()` with consistent formatting.
- [x] 2.2 Replace scalar fallback `eprintln!` usage in `src/chess/moves.rs` with the centralized reporter (or remove per-row logging while preserving `[]` fallback behavior).
- [x] 2.3 Route reader warnings in `src/chess/reader.rs` through the centralized reporter (preserve existing warn-and-continue behavior for glob inputs).
- [x] 2.4 Verify there are no remaining direct `eprintln!` callsites outside the centralized reporter module.

## 3. Refactor Scalars To Use Helpers

- [x] 3.1 Refactor `ChessMovesNormalizeScalar` in `src/chess/filter.rs` to use the shared helpers (preserve NULL -> NULL behavior).
- [x] 3.2 Refactor timecontrol scalars in `src/chess/timecontrol.rs` to use the shared helpers (preserve JSON parse-error fallback behavior).
- [x] 3.3 Refactor move-analysis scalars in `src/chess/moves.rs` to use the shared helpers (preserve each scalar's NULL/default semantics and output types).

## 4. Validation

- [x] 4.1 Run `just check` (fmt + clippy) and fix any warnings.
- [x] 4.2 Run `just test` (unit + SQLLogicTest) and confirm no behavior regressions.
- [x] 4.3 Run `just dev`
- [x] 4.4 Run `just full`
