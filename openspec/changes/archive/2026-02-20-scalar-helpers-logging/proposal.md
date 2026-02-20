## Why

DuckDB scalar functions in this extension currently duplicate a lot of low-level wrapper logic (input vector decoding, NULL checks, `duckdb_string_t` decoding, and output insertion) across multiple modules, and logging/diagnostic behavior is inconsistent (including `eprintln!` in fallbacks). This makes behavior-preserving refactors risky and complicates maintenance.

## What Changes

- Factor shared scalar invoke boilerplate into reusable helper(s) used by `src/chess/moves.rs`, `src/chess/filter.rs`, and `src/chess/timecontrol.rs`.
- Centralize warning/error reporting so scalars use a consistent `warn()`/`error()` policy.
- Remove `eprintln!` usage from scalar fallbacks while preserving existing SQL-visible behaviors (NULL / `[]` / empty-string outputs).
- Validate behavior with the existing test suite (`just test`).

## Capabilities

### New Capabilities
- None.

### Modified Capabilities
- `code-structure`: Define/strengthen requirements for DuckDB scalar wrapper structure (shared decode/encode helpers) and logging policy to keep behavior stable while reducing duplication.

## Impact

- Affected code: `src/chess/moves.rs`, `src/chess/filter.rs`, `src/chess/timecontrol.rs`, plus new shared helper module(s) and logging wrapper(s).
- User-facing SQL behavior: unchanged outputs and NULL-handling; only internal logging wiring is standardized.
- Validation: run `just test`.
