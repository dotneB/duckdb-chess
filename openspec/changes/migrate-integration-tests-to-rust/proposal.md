# Change: Migrate Integration Tests from Python SQLLogicTest to Rust

## Why

The current integration tests use Python's SQLLogicTest framework, which creates an inconsistency with the project's recent migration to a pure Rust workflow:

- **Dependency mismatch**: Tests require Python setup (duckdb-test-runner) while the build system is pure Rust
- **Maintenance burden**: Maintaining 11 SQLLogicTest files (564 lines) requires Python environment setup
- **Developer experience**: New developers must install Python + DuckDB Python client for tests, defeating the "Rust-only" promise
- **CI/CD complexity**: GitHub Actions needs Python installation step for testing
- **Test gap**: Current Rust tests (20 unit tests) only cover internal logic, not end-to-end DuckDB integration

The project now has 11 `.test` files covering:
- `read_pgn` table function (6 test files, ~236 lines)
- `chess_moves_*` scalar functions (5 test files, ~328 lines)

These test DuckDB integration but require Python infrastructure that contradicts the recent modernization.

## What Changes

- **Create Rust integration test module**: New `tests/` directory for DuckDB extension testing
- **Port SQLLogicTest cases**: Convert 11 `.test` files to Rust integration tests
- **Use duckdb crate directly**: Leverage `duckdb` Rust crate for testing (already a dependency)
- **Remove Python test dependency**: Eliminate need for Python-based test runner
- **Move PGN test files**: Relocate `test/pgn_files/` to `tests/pgn_files/` for better organization
- **Remove SQLLogicTest files**: Delete `.test` files after migration is validated
- **Update CI/CD**: Simplify GitHub Actions by removing Python setup for tests

## Impact

**Affected specs:**
- `testing` (new capability spec)

**Affected code:**
- `tests/` (new directory) - Rust integration tests and PGN test data
- `test/pgn_files/` → `tests/pgn_files/` - Moved for better organization
- `test/sql/` (removed) - SQLLogicTest files deleted after validation
- `test/` (potentially removed) - Directory may be empty after migration
- `.github/workflows/MainDistributionPipeline.yml` - Remove Python test steps
- `Makefile` - Update test targets
- `README.md` - Update testing documentation
- `openspec/project.md` - Update testing strategy

**Benefits:**
- **Pure Rust workflow**: No Python required for any development task
- **Faster CI/CD**: No Python installation overhead
- **Better IDE integration**: Rust tests work with cargo test, rust-analyzer, etc.
- **Type safety**: Compile-time checks for test queries
- **Easier debugging**: Standard Rust debugging tools work
- **Consistency**: All tests use same tooling (cargo test)

**Risks:**
- **Test coverage verification**: Need to ensure all SQLLogicTest scenarios are covered
- **Extension loading complexity**: Must handle extension loading/unloading in tests
- **Path handling**: Cross-platform path handling for test PGN files
- **Output assertion format**: SQLLogicTest has specific output format; need equivalent in Rust

**Migration strategy:**
- Start with simplest tests (scalar functions)
- Progress to complex tests (table functions, error handling)
- Validate all tests pass before removing `.test` files
- Move PGN files to `tests/pgn_files/` for cleaner organization
- Remove `test/` directory entirely if no longer needed
