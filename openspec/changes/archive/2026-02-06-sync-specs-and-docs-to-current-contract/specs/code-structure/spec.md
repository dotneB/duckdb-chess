## MODIFIED Requirements

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
