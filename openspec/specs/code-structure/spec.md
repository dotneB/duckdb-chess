# code-structure Specification

## Purpose
TBD - created by archiving change refactor-lib-modules. Update Purpose after archive.
## Requirements
### Requirement: Shared Domain Types Module
The project MUST define shared domain structures in a dedicated `types` module to prevent circular dependencies and promote reuse.

#### Scenario: GameRecord definition
Given the `GameRecord` struct
It must be defined in `src/types.rs`
And it must be public for use by other modules

### Requirement: PGN Parsing Logic Module
The project MUST encapsulate PGN parsing logic in a dedicated `visitor` module.

#### Scenario: Visitor implementation
Given the `GameVisitor` struct and `Visitor` trait implementation
It must be located in `src/visitor.rs`
And it must expose necessary state to the reader module

### Requirement: Reader Table Function Module
The project MUST encapsulate the `read_pgn` DuckDB table function in a dedicated `reader` module.

#### Scenario: Table function implementation
Given the `ReadPgnVTab` struct
It must be located in `src/reader.rs`
And it must use types from `types` and `visitor` modules

### Requirement: Filter Logic Module
The project MUST encapsulate annotation filtering logic in a dedicated `filter` module.

#### Scenario: Filter implementation
Given the `filter_movetext_annotations` function
It must be located in `src/filter.rs`

### Requirement: Clean Entry Point
The `src/lib.rs` file MUST serve primarily as the extension entry point.

#### Scenario: Extension registration
Given the `extension_entrypoint` function
It must remain in `src/lib.rs`
And `src/lib.rs` must declare the sub-modules

### Requirement: Unit Test Support
The project MUST support unit testing for core logic modules to ensure reliability and facilitate refactoring.

#### Scenario: Filter Logic Testing
- **WHEN** `cargo test` is run
- **THEN** `filter_movetext_annotations` logic is verified against a suite of test cases (nested braces, whitespace) without requiring a database connection

#### Scenario: Visitor Logic Testing
- **WHEN** `cargo test` is run
- **THEN** `GameVisitor` parsing logic is verified against mock PGN fragments without requiring file I/O

### Requirement: Test Data Organization
All PGN test data files MUST be located within the `test/pgn_files/` directory to maintain a clean structure.

#### Scenario: Data Location
- **WHEN** a new test PGN file is added
- **THEN** it must be placed in `test/pgn_files/`
- **AND** `test/` root must not contain loose `.pgn` files

