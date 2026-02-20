## ADDED Requirements

### Requirement: Reader column schema is defined once
The `read_pgn` reader implementation MUST define column order, names, and DuckDB types in a single schema descriptor shared by bind-time and execution-time code paths.

#### Scenario: Bind uses shared schema descriptor
- **WHEN** `bind()` registers `read_pgn` output columns
- **THEN** it derives column names and types from the shared descriptor in `src/chess/reader.rs`
- **AND** no separate duplicated column list is maintained in `bind()`

#### Scenario: Row writing uses shared schema descriptor
- **WHEN** `func()` emits a game row
- **THEN** it uses indexes and types derived from the same shared descriptor used by `bind()`
- **AND** column order and types remain identical to the existing SQL contract

### Requirement: Reader chunk writing is modularized without behavior changes
The `read_pgn` table function MUST use dedicated helpers for reader acquisition, game parsing, row emission, and chunk finalization while preserving existing glob, compression, and `parse_error` semantics.

#### Scenario: Chunk row limit uses named constant
- **WHEN** `func()` fills an output chunk
- **THEN** maximum rows per chunk is controlled by a named constant
- **AND** the constant value is `2048`

#### Scenario: Row output uses chunk writer abstraction
- **WHEN** a parsed game record is written to the output chunk
- **THEN** row writes go through a `ChunkWriter`/`write_row` abstraction
- **AND** nullability and typed column writes match existing behavior

#### Scenario: Helper decomposition preserves behavior
- **WHEN** `func()` delegates to `acquire_reader`, `read_next_game`, `write_row`, and `finalize_chunk`
- **THEN** explicit single-path file failures still fail hard
- **AND** glob multi-file unreadable entries are still skipped with warnings
- **AND** `parse_error` accumulation and malformed-game continuation semantics remain unchanged
