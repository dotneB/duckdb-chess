## ADDED Requirements

### Requirement: Isolated BindInfo FFI boundary
The project MUST isolate `BindInfo` to `duckdb_bind_info` interop behind a dedicated adapter with a narrow API surface.

#### Scenario: Single cast location
- **WHEN** reviewing code that bridges `duckdb::vtab::BindInfo` to DuckDB C bind APIs
- **THEN** the cast/interop is implemented in exactly one dedicated adapter location
- **AND** that location is `src/chess/duckdb/bind_info_ffi.rs`
- **AND** `src/chess/reader.rs` uses the adapter API rather than performing direct pointer-layout casts

#### Scenario: Explicit safety invariants
- **WHEN** reviewing the adapter's unsafe boundary
- **THEN** the implementation includes `SAFETY` documentation describing required invariants (valid DuckDB-owned bind pointer, callback-scoped lifetime, and ownership rules)
- **AND** the unsafe code is localized to that boundary

### Requirement: Named-parameter bind behavior parity
Refactoring to isolate the BindInfo FFI boundary MUST preserve existing `read_pgn` named-parameter behavior.

#### Scenario: Compression parameter semantics are unchanged
- **WHEN** `read_pgn` bind resolves the `compression` named parameter through the adapter boundary
- **THEN** omitted or SQL `NULL` values continue to select plain input mode
- **AND** `zstd` remains accepted (case-insensitive)
- **AND** unsupported values continue to return a descriptive bind-time error

### Requirement: duckdb-rs upgrade guidance for BindInfo boundary
The project MUST include maintainer guidance for validating BindInfo FFI boundary safety across `duckdb-rs` version bumps.

#### Scenario: Upgrade checklist is documented
- **WHEN** maintainers prepare a `duckdb-rs` dependency upgrade
- **THEN** repository guidance includes a checklist to verify adapter invariants, validate named-parameter behavior, and run the full test suite

#### Scenario: Upstream accessor adoption path
- **WHEN** `duckdb-rs` provides a stable accessor that replaces raw BindInfo layout casting
- **THEN** the adapter uses the upstream accessor in place of the manual cast
- **AND** guidance documents how to confirm and adopt this transition

### Requirement: DuckDB helper modules are grouped under a dedicated namespace
DuckDB-specific helper modules for scalar invocation, DuckDB string decoding, and bind-time FFI interop MUST be grouped under `src/chess/duckdb/`.

#### Scenario: Helper module locations are explicit
- **WHEN** reviewing DuckDB-specific helper code
- **THEN** scalar invoke helpers are implemented in `src/chess/duckdb/scalar.rs`
- **AND** `duckdb_string_t` decoding is implemented in `src/chess/duckdb/string.rs`
- **AND** BindInfo FFI boundary code is implemented in `src/chess/duckdb/bind_info_ffi.rs`

#### Scenario: Call sites use dedicated duckdb helper namespace imports
- **WHEN** chess modules consume DuckDB helper modules
- **THEN** they import through a dedicated DuckDB helper namespace module backed by `src/chess/duckdb/mod.rs`
- **AND** root-level helper modules for these concerns are not used
