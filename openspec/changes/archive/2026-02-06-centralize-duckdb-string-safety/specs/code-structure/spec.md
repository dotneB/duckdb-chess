## ADDED Requirements

### Requirement: Centralized DuckDB String Decoding Safety
The project MUST implement DuckDB scalar string decoding through a shared helper module rather than duplicated per-module unsafe implementations.

The shared helper MUST document its unsafe contract with explicit `SAFETY` guidance.

#### Scenario: Shared helper usage
- **WHEN** scalar functions decode `duckdb_string_t` input arguments
- **THEN** they call a shared decoding helper from a single module
- **AND** duplicated `read_duckdb_string` implementations are not present in multiple scalar modules

#### Scenario: Safety contract documentation
- **WHEN** reviewing the shared decoder implementation
- **THEN** the unsafe boundary includes a `SAFETY` explanation describing required preconditions for valid reads

#### Scenario: Null checks before decoding
- **WHEN** a scalar function row contains NULL input for a string argument
- **THEN** the scalar invoke path checks row nullability before calling the shared decoding helper
- **AND** behavior remains consistent with prior NULL handling semantics
