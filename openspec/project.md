# Project Context

## Purpose
This is a DuckDB extension written in Rust that provides chess-specific functionality, primarily focused on parsing and querying PGN (Portable Game Notation) files. The extension enables SQL-based analysis of chess games by exposing PGN data through table functions that match the Lichess dataset schema.

Key features:
- Parse PGN files and expose game data as SQL tables
- Support glob patterns for reading multiple PGN files
- Filter and query chess games by opening, player, rating, time control, etc.
- Extract and manipulate movetext data
- Compatible with Lichess database schema for easy integration

## Tech Stack
- **Language**: Rust (2024 edition)
- **Database**: DuckDB (v1.4.3)
- **Core Dependencies**:
  - `duckdb` (1.4.3) - DuckDB Rust bindings with vtab-loadable features
  - `duckdb-ext-macros` (0.1.0) - Modern macros for loadable extensions (Rust 2024 compatible)
  - `libduckdb-sys` (1.4.3) - Low-level DuckDB C API bindings
  - `pgn-reader` (0.28) - PGN parsing library
  - `shakmaty` (0.29) - Chess logic and move validation
  - `glob` (0.3) - File pattern matching
- **Build System**: Cargo + cargo-duckdb-ext-tools (optional Makefile wrappers)
- **Build Tools**: `cargo-duckdb-ext-tools` - Native Rust packaging (replaces Python scripts)
- **Testing**: SQLLogicTest format using DuckDB Python client + Rust unit tests
- **CI/CD**: GitHub Actions with direct cargo builds

## Project Conventions

### Code Style
- Follow standard Rust formatting conventions (use `cargo fmt`)
- Use snake_case for function and variable names
- **DuckDB Scalar Functions**: All scalar functions exposed to SQL must be prefixed with `chess_`.
  - Use `chess_move_` (singular) if the function operates on a single move (e.g. `chess_move_validate('e4')`).
  - Use `chess_moves_` (plural) if the function operates on a sequence of moves / movetext (e.g. `chess_moves_normalize`, `chess_moves_hash`, `chess_moves_json`).
- Use PascalCase for types and structs
- Prefer explicit error handling with `Result<T, E>` over panics
- Use descriptive variable names that reflect chess domain terminology
- Document public APIs and complex logic with doc comments
- Keep functions focused and relatively short (favor composition)
- Use `make dev` as a usefull wrapper that run lint and formatting checks, builds in debug, then the full test suite

### Architecture Patterns
- **Extension Entry Point**: Use `#[duckdb_extension]` macro from `duckdb-ext-macros` to define extension initialization
- **Table Functions**: Implement `VTab` trait for exposing data as SQL tables
- **Visitor Pattern**: Use `pgn-reader`'s `Visitor` trait for streaming PGN parsing
- **Data Flow**:
  1. Bind phase: Parse parameters, define output schema, expand globs
  2. Init phase: Initialize state (atomic flags, mutexes for shared data)
  3. Function phase: Parse PGN files, accumulate games, output in chunks (2048 rows)
- **Error Handling**: Return `Box<dyn Error>` for extension errors, log warnings for malformed games
- **Thread Safety**: Use atomic types and mutexes for shared state across table function calls
- **Chunked Processing**: Output data in chunks to avoid memory issues with large datasets

### Testing Strategy
- Tests are written in SQLLogicTest format (.test files in `test/sql/`)
- Test files must start with `require chess` to load the extension
- Test categories:
  - Schema validation (DESCRIBE queries)
  - Row count verification
  - Query correctness (filtering, ordering, projections)
  - Glob pattern support
- **Unit Testing**: Run `cargo test` to verify core logic in `src/filter.rs` and `src/visitor.rs`.
- **Integration Testing**:
  - Run `cargo duckdb-ext-build` or `make build` to build the extension.
  - Use `make test` to run Rust unit tests.
  - For SQLLogicTest tests, manually load extension in DuckDB and verify functionality.
- Use sample PGN files in `test/pgn_files/`.
- Use `make test-rs` for an easy way to build in debug, run the unit tests and run the integration tests
- Use `make test-release-rs` for an easy way to build in release, run the unit tests and run the integration tests

### Git Workflow
- Use descriptive commit messages that explain "why" not just "what"
- Branch naming: feature/, bugfix/, refactor/, docs/
- Test locally before pushing: `cargo test` && `cargo duckdb-ext-build -- --release`
- CI/CD pipeline runs on push via GitHub Actions
- Extension must build for multiple platforms (Linux amd64/arm64, Windows, macOS)

## Domain Context

### Chess Domain Knowledge
- **PGN Format**: Standard chess game notation with header tags and movetext
- **Lichess Schema**: Extension matches Lichess database export format for compatibility
- **Key Headers**: Event, Site, White, Black, Result, WhiteElo, BlackElo, ECO, Opening, Termination, TimeControl, UTCDate, UTCTime
- **Movetext**: Sequence of moves in Standard Algebraic Notation (SAN) with move numbers
- **Annotations**: Comments in curly braces `{ }` that can be filtered out
- **ECO Codes**: Encyclopedia of Chess Openings classification system
- **Result Format**: "1-0" (white wins), "0-1" (black wins), "1/2-1/2" (draw), "*" (ongoing)

### Table Functions
1. **read_pgn(path_pattern)**: Main function to parse PGN files
   - Supports single file paths or glob patterns
   - Returns 16 columns matching Lichess schema
   - Handles malformed games gracefully (logs warnings, continues parsing)

2. **chess_moves_normalize(movetext)**: Utility function
   - Removes annotations, variations, and numeric glyphs from movetext
   - Preserves move structure and numbering
   - Returns a canonical "main line" string

## Important Constraints
- **DuckDB Version**: Target v1.4.3 (set in Cargo.toml dependencies)
- **Rust Edition**: Requires Rust 2024 edition (Rust 1.88.0+)
- **Platform Support**: Must build for Linux (amd64/arm64), Windows, macOS
- **Unsigned Extensions**: Local testing requires `duckdb -unsigned` flag
- **LTO & Strip**: Release builds use link-time optimization and symbol stripping for size
- **Chunk Size**: Output limited to 2048 rows per chunk to manage memory
- **Error Tolerance**: Parser skips malformed games rather than failing entire batch

## External Dependencies
- **DuckDB C API**: Core extension interface via libduckdb-sys
- **pgn-reader**: Third-party PGN parsing library from chess ecosystem
- **shakmaty**: Chess move validation and game logic
- **cargo-duckdb-ext-tools**: Native Rust build and packaging tools (replaces Python scripts)
- **duckdb-ext-macros**: Community-maintained procedural macros for Rust 2024 Edition
- **GitHub Actions**: CI/CD platform for automated builds and testing
- **SQLLogicTest**: Testing framework for SQL correctness verification
