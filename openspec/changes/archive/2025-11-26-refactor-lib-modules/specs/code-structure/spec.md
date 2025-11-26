# Code Structure & Modularization

This specification defines the architectural module boundaries for the project.

## ADDED Requirements

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
