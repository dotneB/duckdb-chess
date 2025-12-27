# Testing Capability

## ADDED Requirements

### Requirement: Rust Integration Test Infrastructure
The project SHALL provide Rust-based integration tests for DuckDB extension functionality using the `duckdb` crate.

#### Scenario: Integration tests directory exists
- **WHEN** the project is checked out
- **THEN** a `tests/` directory SHALL exist at the repository root
- **AND** it SHALL contain integration test modules

#### Scenario: Common test helpers available
- **WHEN** writing integration tests
- **THEN** a `tests/common/` module SHALL provide shared utilities
- **AND** helpers SHALL include extension loading and assertion functions

#### Scenario: Tests use duckdb crate
- **WHEN** integration tests run
- **THEN** they SHALL use the `duckdb` Rust crate for database operations
- **AND** they SHALL NOT require Python or external test frameworks

### Requirement: Extension Loading Helper
Integration tests SHALL provide a helper function to load the DuckDB extension automatically.

#### Scenario: Load extension helper exists
- **WHEN** a test needs to use the extension
- **THEN** it SHALL call `load_extension()` from common helpers
- **AND** the helper SHALL return a configured Connection

#### Scenario: Platform-specific paths handled
- **WHEN** loading the extension on different platforms
- **THEN** the helper SHALL determine the correct extension path for Windows, macOS, and Linux
- **AND** it SHALL use debug or release build based on configuration

#### Scenario: Fresh connection per test
- **WHEN** each test function runs
- **THEN** it SHALL get a fresh in-memory DuckDB connection
- **AND** the extension SHALL be loaded for that connection
- **AND** tests SHALL NOT share state

### Requirement: Assertion Helpers
Integration tests SHALL provide assertion helpers for common DuckDB test patterns.

#### Scenario: Assert query result helper
- **WHEN** testing a query that returns a single value
- **THEN** `assert_query_result()` helper SHALL be available
- **AND** it SHALL compare the query result to an expected value

#### Scenario: Assert row count helper
- **WHEN** testing query result size
- **THEN** `assert_row_count()` helper SHALL be available
- **AND** it SHALL verify the number of rows returned

#### Scenario: Assert column schema helper
- **WHEN** testing table structure
- **THEN** `assert_columns()` helper SHALL be available
- **AND** it SHALL verify column names and types

#### Scenario: Assert query error helper
- **WHEN** testing error conditions
- **THEN** `assert_query_error()` helper SHALL be available
- **AND** it SHALL verify that a query fails with expected error message

### Requirement: Scalar Function Test Coverage
Integration tests SHALL cover all scalar functions provided by the extension.

#### Scenario: chess_moves_normalize tested
- **WHEN** integration tests run
- **THEN** tests SHALL verify `chess_moves_normalize()` removes annotations
- **AND** tests SHALL verify it handles nested annotations
- **AND** tests SHALL verify it preserves move structure

#### Scenario: chess_moves_hash tested
- **WHEN** integration tests run
- **THEN** tests SHALL verify `chess_moves_hash()` produces consistent hashes
- **AND** tests SHALL verify identical movetext produces same hash
- **AND** tests SHALL verify annotation-free movetext handling

#### Scenario: chess_moves_json tested
- **WHEN** integration tests run
- **THEN** tests SHALL verify `chess_moves_json()` converts movetext to JSON
- **AND** tests SHALL verify JSON structure correctness

#### Scenario: chess_moves_subset tested
- **WHEN** integration tests run
- **THEN** tests SHALL verify `chess_moves_subset()` extracts specified number of moves
- **AND** tests SHALL verify boundary conditions (0 moves, more than available)

### Requirement: Table Function Test Coverage
Integration tests SHALL cover the `read_pgn` table function functionality.

#### Scenario: Schema validation tested
- **WHEN** integration tests run
- **THEN** tests SHALL verify `read_pgn()` returns correct column schema
- **AND** tests SHALL verify all 17 columns exist with correct types

#### Scenario: Basic PGN parsing tested
- **WHEN** integration tests run
- **THEN** tests SHALL verify `read_pgn()` reads PGN files correctly
- **AND** tests SHALL verify game count accuracy
- **AND** tests SHALL verify header field extraction

#### Scenario: Glob pattern support tested
- **WHEN** integration tests run
- **THEN** tests SHALL verify `read_pgn()` handles glob patterns
- **AND** tests SHALL verify multiple file processing

#### Scenario: Filtering and projection tested
- **WHEN** integration tests run
- **THEN** tests SHALL verify SQL filtering on PGN data
- **AND** tests SHALL verify column projection works correctly

### Requirement: Error Handling Test Coverage
Integration tests SHALL verify error handling for malformed inputs and edge cases.

#### Scenario: Malformed PGN handling tested
- **WHEN** integration tests run
- **THEN** tests SHALL verify extension handles malformed PGN gracefully
- **AND** tests SHALL verify error messages in `parse_error` column

#### Scenario: Bad UTF-8 handling tested
- **WHEN** integration tests run
- **THEN** tests SHALL verify extension handles invalid UTF-8 in PGN files
- **AND** tests SHALL NOT crash or panic

#### Scenario: Missing file handling tested
- **WHEN** integration tests run
- **THEN** tests SHALL verify appropriate error when PGN file doesn't exist
- **AND** error message SHALL be helpful

### Requirement: Null Value Handling
Integration tests SHALL verify correct handling of null/missing values in PGN data.

#### Scenario: Missing headers produce nulls
- **WHEN** PGN game lacks optional headers
- **THEN** tests SHALL verify NULL values returned for missing fields
- **AND** tests SHALL verify required fields still populated

#### Scenario: Empty movetext handled
- **WHEN** PGN game has empty or missing movetext
- **THEN** tests SHALL verify graceful handling
- **AND** tests SHALL verify appropriate null or empty string returned

### Requirement: Test Execution Independence
Integration tests SHALL run independently without Python or external dependencies.

#### Scenario: Cargo test runs all tests
- **WHEN** developer runs `cargo test`
- **THEN** all unit tests AND integration tests SHALL execute
- **AND** no Python installation SHALL be required

#### Scenario: Extension build prerequisite documented
- **WHEN** developer reads test documentation
- **THEN** it SHALL specify extension must be built before testing
- **AND** it SHALL provide command to build extension

#### Scenario: CI runs tests without Python
- **WHEN** CI/CD pipeline runs
- **THEN** integration tests SHALL execute using only Rust toolchain
- **AND** Python SHALL NOT be installed for testing

### Requirement: Test Organization
Integration tests SHALL be organized by functionality for maintainability.

#### Scenario: Module per function category
- **WHEN** exploring test code
- **THEN** scalar function tests SHALL be in `tests/test_chess_moves.rs`
- **AND** table function tests SHALL be in `tests/test_read_pgn.rs`
- **AND** error tests SHALL be in `tests/test_error_handling.rs`

#### Scenario: Shared utilities in common module
- **WHEN** tests need helper functions
- **THEN** shared code SHALL be in `tests/common/`
- **AND** it SHALL be reusable across test modules

### Requirement: Test Coverage Parity
Integration tests SHALL maintain or exceed the coverage provided by previous SQLLogicTest suite.

#### Scenario: All SQLLogicTest scenarios covered
- **WHEN** migration is complete
- **THEN** every scenario from `.test` files SHALL have equivalent Rust test
- **AND** test count SHALL be equal or greater

#### Scenario: Edge cases preserved
- **WHEN** validating migration
- **THEN** all edge cases from SQLLogicTest SHALL be included
- **AND** no test coverage SHALL be lost

## MODIFIED Requirements

N/A - This is a new testing capability

## REMOVED Requirements

N/A - Existing unit tests are retained
