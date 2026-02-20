## Why

`src/chess/reader.rs` currently defines the `read_pgn` output schema and row-writing flow across multiple places, which makes behavior-preserving refactors risky and increases maintenance cost. We need a focused internal refactor now so future changes can evolve safely without regressing column shape, parse diagnostics, or streaming behavior.

## What Changes

- Refactor `read_pgn` internals so column metadata is defined once and shared by `bind()` and `func()`.
- Introduce a `ChunkWriter`/`write_row()` abstraction to centralize per-row vector writes and null handling.
- Split `func()` into smaller helpers (`acquire_reader`, `read_next_game`, `write_row`, `finalize_chunk`) while preserving current behavior.
- Replace magic numbers (including rows-per-chunk) with named constants in `reader.rs`.
- Validate no behavioral regressions with existing tests (`just test`).

## Capabilities

### New Capabilities
- None.

### Modified Capabilities
- `code-structure`: Tighten reader-module structure requirements so `read_pgn` schema and chunk-writing logic remain maintainable and behavior-preserving during refactors.

## Impact

- Affected code: `src/chess/reader.rs` and any nearby reader-focused tests that need small updates.
- User-facing SQL contract: unchanged (column order/types, glob behavior, compression handling, malformed-game continuation, and `parse_error` semantics stay identical).
- Validation: run `just test` after implementation.
