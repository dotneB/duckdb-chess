## Why

Unsafe DuckDB string decoding logic is duplicated across scalar modules, which increases maintenance risk and makes safety guarantees harder to audit. Centralizing this logic into one helper with explicit `SAFETY` documentation reduces drift and improves confidence without changing SQL behavior.

## What Changes

- Add a shared DuckDB string decoding helper used by scalar implementations.
- Remove duplicated `read_duckdb_string` implementations and route call sites through the shared helper.
- Add explicit `SAFETY` documentation for the centralized unsafe boundary and enforce null-row checks at call sites before unsafe reads.
- Preserve all current SQL-visible behavior for `chess_moves_*` and related scalar functions.
- Add or adjust tests to ensure no functional behavior changes while safety/maintainability improves.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `code-structure`: add a requirement for centralized DuckDB string decoding safety and documented unsafe usage patterns.

## Impact

- Affected code: `src/chess/filter.rs`, `src/chess/moves.rs`, and a new/updated shared helper module under `src/chess/`.
- Affected tests: scalar function unit tests and SQLLogicTests for null propagation and unchanged behavior.
- User-visible impact: none intended; change is internal refactoring and safety hardening.
