# code-structure Specification

## Purpose
To define the repository module layout and extension entrypoint wiring used by the current `src/chess/*` architecture.
## Requirements
### Requirement: Shared Domain Types Module
The project MUST define shared domain structures in a dedicated `types` module to prevent circular dependencies and promote reuse.

#### Scenario: GameRecord definition
- **WHEN** shared game row structures are defined
- **THEN** `GameRecord` is defined in `src/chess/types.rs`
- **AND** it is public for use by other chess modules

### Requirement: PGN Parsing Logic Module
The project MUST encapsulate PGN parsing logic in a dedicated `visitor` module.

#### Scenario: Visitor implementation
- **WHEN** implementing `GameVisitor` and `pgn_reader::Visitor`
- **THEN** the implementation is located in `src/chess/visitor.rs`
- **AND** it exposes the state needed by the reader module

### Requirement: Reader Table Function Module
The project MUST encapsulate the `read_pgn` DuckDB table function in a dedicated `reader` module.

#### Scenario: Table function implementation
- **WHEN** implementing `ReadPgnVTab`
- **THEN** the implementation is located in `src/chess/reader.rs`
- **AND** it uses shared types/visitor modules from `src/chess/`

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

### Requirement: Filter Logic Module
The project MUST encapsulate annotation/movetext filtering logic in a dedicated `filter` module.

#### Scenario: Filter implementation
- **WHEN** implementing movetext annotation filtering helpers
- **THEN** the implementation is located in `src/chess/filter.rs`

### Requirement: Clean Entry Point
The crate root and extension registration entrypoint MUST be separated into a thin root module and a chess extension module.

#### Scenario: Module root wiring
- **WHEN** reviewing crate root wiring
- **THEN** `src/lib.rs` primarily declares the `chess` module

#### Scenario: Extension registration location
- **WHEN** reviewing extension function registration
- **THEN** `extension_entrypoint` is implemented in `src/chess/mod.rs`
- **AND** `read_pgn` and `chess_*` scalar/macros are registered there

### Requirement: Extension Entry Point Macro
The project MUST use the modern `#[duckdb_extension]` macro for extension entrypoint registration.

#### Scenario: Modern macro usage
- **WHEN** reviewing `extension_entrypoint`
- **THEN** it uses `#[duckdb_extension(name = "chess", api_version = "v1.0.0")]`
- **AND** legacy entrypoint macros are not used

### Requirement: Unit Test Support
The project MUST support unit testing for core logic modules to ensure reliability and facilitate refactoring.

#### Scenario: Filter logic testing
- **WHEN** `cargo test` is run
- **THEN** movetext normalization and filtering behavior is verified without requiring database setup

#### Scenario: Visitor logic testing
- **WHEN** `cargo test` is run
- **THEN** `GameVisitor` parsing behavior is verified against PGN fragments without requiring full file ingestion workflows

### Requirement: Test Data Organization
All PGN test data files MUST be located within `test/pgn_files/` to maintain a clean structure.

#### Scenario: Data location
- **WHEN** a new PGN fixture is added
- **THEN** it is placed in `test/pgn_files/`
- **AND** `test/` root does not contain loose `.pgn` fixtures

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

### Requirement: TimeControl parser internals are decomposed by responsibility
The project MUST decompose TimeControl parsing internals into dedicated modules for strict parsing, inference/normalization, and JSON rendering while preserving backward-compatible SQL behavior.

#### Scenario: Dedicated module boundaries exist
- **WHEN** maintainers review TimeControl implementation layout
- **THEN** strict parsing, inference logic, and JSON rendering are implemented in separate `src/chess/timecontrol/*` modules
- **AND** a thin `timecontrol` facade exposes the existing public entrypoints

#### Scenario: SQL API remains stable after refactor
- **WHEN** users call `chess_timecontrol_normalize`, `chess_timecontrol_json`, and `chess_timecontrol_category`
- **THEN** function names and return contracts remain unchanged
- **AND** NULL handling semantics remain unchanged

#### Scenario: Warning taxonomy and inference semantics are preserved
- **WHEN** existing inferred shorthand and strict inputs are parsed
- **THEN** warning codes and inference decisions match pre-refactor behavior
- **AND** no new warning categories are introduced by the modularization itself

#### Scenario: Existing fixture behavior remains equivalent
- **WHEN** existing TimeControl fixtures and SQL tests are executed after modularization
- **THEN** normalized outputs and JSON parse fields remain equivalent to prior behavior

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
