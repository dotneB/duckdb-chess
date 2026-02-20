## Context

`ReadPgnVTab` in `src/chess/reader.rs` currently mixes schema declaration, reader acquisition, game parsing, chunk buffering, and DuckDB vector writes inside a large `func()` flow. Column metadata is effectively duplicated between `bind()` and row-writing logic, and chunk sizing still relies on literals (for example `2048`).

The requested change is an internal refactor only: keep SQL-visible behavior and parsing semantics identical while making the reader easier to evolve safely.

## Goals / Non-Goals

**Goals:**
- Define `read_pgn` output columns in one shared source used by both `bind()` and row writing.
- Introduce a `ChunkWriter`/`write_row()` abstraction to centralize vector writes and nullability handling.
- Decompose `func()` into focused helpers (`acquire_reader`, `read_next_game`, `write_row`, `finalize_chunk`).
- Replace magic numbers with named constants (including rows-per-chunk).
- Preserve all current behavior: column order/types, glob handling, compression behavior, malformed-game continuation, and `parse_error` semantics.

**Non-Goals:**
- No user-facing API changes to `read_pgn` signature or output schema.
- No parser/visitor behavior changes for movetext, comments, or variation handling.
- No change to glob-expansion rules, file-open failure policy, or compression support.
- No new performance architecture (threading model, chunking strategy, or buffering model).

## Decisions

1. **Shared schema descriptor for `read_pgn` columns**
   - Add a single schema definition (column name + DuckDB type metadata) in `reader.rs` and use it in both `bind()` and row-writing code paths.
   - Use schema-derived constants for column count and indexes to remove duplicated numeric indexing.
   - **Alternative considered:** Keep `bind()` and writer metadata separate but document ordering better. Rejected because drift remains possible during future edits.

2. **`ChunkWriter` abstraction for row output**
   - Introduce a small writer type that owns chunk-local write concerns (current row index, column vectors, typed write helpers, null-mask handling).
   - Implement a `write_row()` routine that maps one parsed game record into all output columns through this abstraction.
   - **Alternative considered:** Add multiple free helper functions operating on raw vectors. Rejected because call sites stay verbose and error-prone.

3. **Decompose `func()` into staged helpers**
   - `acquire_reader`: obtain/reuse current parser state and handle file transitions.
   - `read_next_game`: parse one game, producing either a game row, EOF/file transition signal, or error path outcome.
   - `write_row`: emit parsed record into the output chunk.
   - `finalize_chunk`: set output cardinality and finalize per-call state.
   - Keep control flow equivalent to existing behavior for malformed games and file-level failure modes.
   - **Alternative considered:** Introduce a new iterator/stream type across modules. Rejected as too invasive for a behavior-preserving refactor.

4. **Named constants for magic values**
   - Replace inline numeric literals with descriptive constants (for example `ROWS_PER_CHUNK: usize = 2048`).
   - Keep values unchanged to preserve runtime behavior.
   - **Alternative considered:** make chunk size configurable. Rejected because this change intentionally avoids user-visible behavior changes.

## Risks / Trade-offs

- **[Risk] Helper split alters error/continuation control flow** -> **Mitigation:** preserve existing branch semantics and cover with current SQLLogicTests for parse errors and file handling.
- **[Risk] Column mapping drift during refactor** -> **Mitigation:** enforce schema-driven ordering and verify with schema-focused tests (`DESCRIBE`/`SELECT` expectations already present).
- **[Risk] Subtle NULL-handling differences in vector writes** -> **Mitigation:** centralize null writes in `ChunkWriter` methods and keep typed conversion behavior unchanged.
- **[Trade-off] Added abstraction layer** -> **Mitigation:** keep helper API narrow and local to `reader.rs` to retain readability without architecture churn.

## Migration Plan

- Internal refactor only; no data migration or rollout sequencing is required.
- Validate with `just test` before merge.
- If regressions appear, rollback is a standard code revert.

## Open Questions

- None at this time.
