## ADDED Requirements

### Requirement: Centralized DuckDB Scalar Invoke Boilerplate
The project MUST implement shared helpers for DuckDB scalar invocation boilerplate (flat vector access, row iteration, NULL checks, DuckDB string decoding, and output insertion) so scalar modules do not duplicate unsafe wrapper code.

#### Scenario: Scalar wrapper uses shared helper
- **WHEN** implementing or refactoring a DuckDB scalar `invoke()` over `duckdb_string_t` inputs
- **THEN** the scalar uses the shared helper(s) for row iteration, null checks, and decoding
- **AND** per-scalar modules contain only minimal glue and domain logic.

#### Scenario: Unsafe boundary localized
- **WHEN** reviewing scalar wrapper code for unsafe pointer/string operations
- **THEN** unsafe DuckDB vector decoding is localized to the shared helper module(s)
- **AND** each unsafe boundary includes `SAFETY` documentation.

### Requirement: Centralized Extension Warning/Error Reporting
The project MUST route extension warnings and recoverable errors through a centralized logging/reporting module rather than direct stderr printing.

#### Scenario: No direct stderr printing in extension code
- **WHEN** extension code needs to emit a warning or error message (including scalar fallbacks)
- **THEN** it uses the centralized reporting helper (`warn()` / `error()` or equivalent)
- **AND** direct calls to `eprintln!` / `println!` are limited to the centralized reporting module (or removed entirely).
