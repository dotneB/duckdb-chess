# allocation-efficiency Specification

## Purpose
To define non-functional allocation-efficiency requirements for PGN parsing and move-analysis internals so hot paths minimize avoidable allocations while preserving existing SQL-visible behavior.
## Requirements
### Requirement: Borrow-First DuckDB String Decoding
The system MUST decode DuckDB scalar string inputs using a borrow-first strategy that avoids heap allocation for valid UTF-8 inputs and allocates only when lossy replacement is required.

#### Scenario: Valid UTF-8 input avoids owned allocation path
- **WHEN** a scalar function reads a valid UTF-8 `duckdb_string_t` argument
- **THEN** the shared decoder returns a borrowed string view suitable for immediate processing

#### Scenario: Invalid UTF-8 input remains resilient
- **WHEN** a scalar function reads a `duckdb_string_t` argument containing invalid UTF-8 bytes
- **THEN** decoding succeeds using replacement semantics and allocates only for the lossy representation

### Requirement: Allocation Optimization Preserves Existing SQL Semantics
All allocation-reduction refactors MUST preserve existing SQL-visible behavior for affected table/scalar functions, including NULL handling and error/degradation semantics.

#### Scenario: Behavior parity for movetext helpers
- **WHEN** allocation-focused refactors are applied to move-analysis internals
- **THEN** results for equivalent inputs match pre-change semantics for `chess_moves_json`, `chess_moves_normalize`, `chess_ply_count`, and `chess_moves_hash`

#### Scenario: Behavior parity for PGN row extraction
- **WHEN** allocation-focused refactors are applied to PGN visitor and reader internals
- **THEN** `read_pgn` continues to emit the same schema and preserves malformed-game continuation and `parse_error` behavior

### Requirement: Visitor tag decoding avoids unnecessary allocation
The `read_pgn` PGN visitor MUST avoid decoding or allocating tag values that do not contribute to the emitted row, including unknown tags and duplicate known tags that are ignored by existing semantics.

#### Scenario: Unknown tags do not affect output
- **WHEN** a game contains header tags outside the known output tag set
- **THEN** those tags do not affect any emitted output columns
- **AND** parsing continues normally

#### Scenario: Duplicate known tag uses first value
- **WHEN** a game contains a known header tag more than once (e.g., two `Event` tags)
- **THEN** the emitted column uses the first parsed value
- **AND** later duplicates are ignored

### Requirement: Movetext finalization avoids unnecessary cloning
The `read_pgn` visitor MUST avoid creating a new owned movetext string during record finalization when trimming would not change the assembled movetext, while preserving the exact trimming semantics of the current implementation.

#### Scenario: Movetext with no surrounding whitespace is preserved
- **WHEN** the visitor assembles movetext with no leading or trailing whitespace
- **THEN** the emitted movetext value is identical to the assembled movetext

#### Scenario: Movetext surrounding whitespace is trimmed
- **WHEN** the visitor assembles movetext that includes leading or trailing whitespace
- **THEN** the emitted movetext value matches the result of trimming surrounding whitespace
