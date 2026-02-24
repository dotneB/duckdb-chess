## MODIFIED Requirements

### Requirement: Reader chunk writing is modularized without behavior changes
The `read_pgn` table function MUST use dedicated helpers for reader acquisition, game parsing, row emission, and chunk finalization while preserving existing glob, compression, and `parse_error` semantics.

#### Scenario: Chunk row limit uses runtime vector capacity
- **WHEN** `func()` fills an output chunk
- **THEN** maximum rows per chunk is controlled by runtime capacity read from the output chunk vector(s)
- **AND** chunk-full checks use that runtime value instead of a fixed `2048` contract

#### Scenario: Row output uses chunk writer abstraction
- **WHEN** a parsed game record is written to the output chunk
- **THEN** row writes go through a `ChunkWriter`/`write_row` abstraction
- **AND** nullability and typed column writes match existing behavior

#### Scenario: Helper decomposition preserves behavior
- **WHEN** `func()` delegates to `acquire_reader`, `read_next_game`, `write_row`, and `finalize_chunk`
- **THEN** explicit single-path file failures still fail hard
- **AND** glob multi-file unreadable entries are still skipped with warnings
- **AND** `parse_error` accumulation and malformed-game continuation semantics remain unchanged
