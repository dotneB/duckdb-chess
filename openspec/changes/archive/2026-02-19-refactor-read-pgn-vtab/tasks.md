## 1. Shared Schema Source of Truth

- [x] 1.1 Capture current `read_pgn` column order and DuckDB types from `bind()`/`func()` and define a single shared schema descriptor in `src/chess/reader.rs`.
- [x] 1.2 Update `bind()` to register output columns from the shared schema descriptor without changing names, order, or types.

## 2. Reader Refactor and Chunk Writer

- [x] 2.1 Replace magic numbers in `reader.rs` with named constants, including rows-per-chunk (`2048`).
- [x] 2.2 Implement a `ChunkWriter` plus `write_row()` path that centralizes per-column vector writes and NULL handling while preserving `parse_error` behavior.
- [x] 2.3 Split `ReadPgnVTab::func()` into `acquire_reader`, `read_next_game`, `write_row`, and `finalize_chunk` helpers while preserving glob, compression, malformed-game continuation, and single-file failure semantics.

## 3. Validation

- [x] 3.1 Add or adjust tests only where needed to lock in schema parity and behavior-preserving refactor expectations.
- [x] 3.2 Run `just test` and fix any regressions until the suite passes.
