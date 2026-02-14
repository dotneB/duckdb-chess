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
