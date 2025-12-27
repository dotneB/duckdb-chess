# Code Structure Capability Deltas

## MODIFIED Requirements

### Requirement: Extension Entry Point Macro
The extension SHALL define its entry point using the `#[duckdb_extension]` procedural macro from the `duckdb-ext-macros` crate, which provides modern Rust 2024 Edition compatibility.

#### Scenario: Entry point defined with modern macro
- **WHEN** the extension entry point is defined in `src/chess/mod.rs`
- **THEN** it SHALL use `#[duckdb_extension(name = "duckdb_chess", api_version = "v1.0.0")]`
- **AND** it SHALL import from `duckdb_ext_macros` instead of `duckdb_loadable_macros`

#### Scenario: Function signature preserved
- **WHEN** the entry point function is defined
- **THEN** it SHALL accept a `Connection` parameter
- **AND** it SHALL return `Result<(), Box<dyn Error>>`
- **AND** the function body SHALL register all table and scalar functions identically

#### Scenario: Extension name consistent
- **WHEN** the extension is built
- **THEN** the C API entry point function SHALL be named `duckdb_chess_init_c_api`
- **AND** the extension name SHALL be `duckdb_chess` consistently throughout
