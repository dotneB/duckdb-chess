## ADDED Requirements

### Requirement: ChunkWriter VARCHAR insertion avoids CString allocation
The `read_pgn` row-emission path MUST write VARCHAR values using DuckDB length-based insertion (`Inserter<&str>` / pointer+length assignment) and MUST NOT require per-value `CString` allocation in `ChunkWriter`.

#### Scenario: Hot-path string columns use length-based insertion
- **WHEN** `ChunkWriter` writes `movetext`, `parse_error`, or optional VARCHAR output columns
- **THEN** it inserts string values through DuckDB's length-based inserter path
- **AND** it avoids per-row `CString::new(...)` construction for those writes

#### Scenario: SQL-visible behavior is preserved
- **WHEN** allocation-focused string-insertion refactors are applied to `read_pgn` row emission
- **THEN** emitted values and NULL semantics remain consistent with pre-change behavior
- **AND** malformed-game continuation and `parse_error` semantics remain unchanged

#### Scenario: Interior NUL handling parity
- **WHEN** an emitted string value contains interior NUL bytes
- **THEN** sanitization is applied before insertion
- **AND** sanitization diagnostics continue to follow existing `parse_error` rules
