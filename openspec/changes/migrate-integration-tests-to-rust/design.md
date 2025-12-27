# Design: Migrate Integration Tests to Rust

## Context

The project recently migrated to a modern Rust-only build system, eliminating Python and Make dependencies. However, integration tests still use Python's SQLLogicTest framework via `duckdb-test-runner`.

**Current test structure:**
- **Unit tests**: 20 tests in `src/chess/*.rs` (pure Rust, test internal logic)
- **Integration tests**: 11 `.test` files in `test/sql/` (SQLLogicTest format, require Python)

**SQLLogicTest files:**
1. `chess_moves_hash.test` (71 lines)
2. `chess_moves_json.test` (48 lines)
3. `chess_moves_normalize.test` (102 lines)
4. `chess_moves_normalize_column.test` (17 lines)
5. `chess_moves_subset.test` (90 lines)
6. `read_pgn.test` (44 lines)
7. `read_pgn_bad_utf8_real.test` (13 lines)
8. `read_pgn_errors.test` (39 lines)
9. `read_pgn_glob.test` (33 lines)
10. `read_pgn_nulls.test` (76 lines)
11. `read_pgn_parse_errors.test` (31 lines)

**Stakeholders:**
- Developers (want consistent Rust-only workflow)
- CI/CD pipeline (wants simpler, faster tests)
- Extension users (unaffected, but benefit from more reliable tests)

**Constraints:**
- Must maintain or improve test coverage
- Must test actual DuckDB integration (not mocked)
- Must work cross-platform (Windows, Linux, macOS)
- Should not significantly increase test runtime

## Goals / Non-Goals

**Goals:**
- Eliminate Python dependency for testing
- Port all SQLLogicTest scenarios to Rust
- Maintain or improve test coverage
- Enable standard Rust testing workflow (`cargo test`)
- Simplify CI/CD pipeline

**Non-Goals:**
- Rewriting the SQLLogicTest framework in Rust (use `duckdb` crate directly)
- Testing DuckDB itself (only test extension functionality)
- Creating a general-purpose DuckDB test framework
- Migrating to a different database for testing

## Decisions

### Decision 1: Use Cargo Integration Tests (`tests/` directory)

**Rationale:**
- Standard Rust convention for integration tests
- Compiled as separate crate, tests external API
- Perfect for testing DuckDB extension loading and usage
- Automatic discovery by `cargo test`

**Alternatives considered:**
1. **Inline tests in src/**: Rejected, these are for unit tests
2. **Separate test crate**: Rejected, over-engineering for this use case
3. **Keep SQLLogicTest**: Rejected, contradicts Rust-only goal

**Structure:**
```
tests/
├── test_chess_moves.rs           # Scalar function tests
├── test_read_pgn.rs              # Table function tests
├── test_error_handling.rs        # Error case tests
├── common/                       # Shared test utilities
│   ├── mod.rs
│   └── helpers.rs                # Extension loading, assertions
└── pgn_files/                    # PGN test data (moved from test/pgn_files/)
    ├── sample.pgn
    ├── game1.pgn
    ├── game2.pgn
    ├── malformed.pgn
    ├── nulls.pgn
    ├── empty.pgn
    ├── bad_utf8_real.pgn
    └── parse_errors.pgn

test/                             # REMOVED after migration
└── sql/                          # REMOVED - SQLLogicTest files deleted
```

### Decision 2: Direct `duckdb` Crate Usage

**Rationale:**
- Already a project dependency (no new dependencies)
- Provides full DuckDB functionality
- Supports extension loading via `LOAD` command
- Native Rust API with type safety

**Pattern:**
```rust
use duckdb::{Connection, Result};

#[test]
fn test_chess_moves_normalize() -> Result<()> {
    let conn = Connection::open_in_memory()?;
    conn.execute_batch("LOAD './target/debug/duckdb_chess.duckdb_extension';")?;
    
    let result: String = conn.query_row(
        "SELECT chess_moves_normalize('1. e4 {comment} e5')",
        [],
        |row| row.get(0)
    )?;
    
    assert_eq!(result, "e4 e5");
    Ok(())
}
```

**Alternatives considered:**
1. **Create SQLLogicTest parser**: Rejected, over-engineering
2. **Use sqllogictest-rs crate**: Considered, but adds dependency and doesn't eliminate .test files
3. **Mock DuckDB**: Rejected, need real integration testing

### Decision 3: Test Organization by Functionality

**Rationale:**
- Mirrors the SQLLogicTest file organization
- Easy to track migration progress
- Clear responsibility boundaries

**Modules:**
- `test_read_pgn.rs` - read_pgn table function tests
- `test_chess_moves.rs` - All chess_moves_* scalar function tests
- `test_error_handling.rs` - Error cases and edge cases
- `common/` - Shared helpers

**Alternatives considered:**
1. **Single monolithic file**: Rejected, hard to maintain
2. **Group by test type (unit/integration)**: Rejected, all are integration
3. **Mirror .test file structure exactly**: Rejected, can consolidate similar tests

### Decision 4: Extension Loading Strategy

**Rationale:**
- Extension must be built before tests run
- Each test gets fresh connection to avoid state pollution
- Helper function encapsulates loading logic

**Implementation:**
```rust
// tests/common/helpers.rs
pub fn load_extension() -> Result<Connection> {
    let conn = Connection::open_in_memory()?;
    
    // Determine extension path based on platform and build type
    let ext_path = if cfg!(debug_assertions) {
        extension_path_debug()
    } else {
        extension_path_release()
    };
    
    conn.execute_batch(&format!("LOAD '{}';", ext_path))?;
    Ok(conn)
}

fn extension_path_debug() -> String {
    #[cfg(target_os = "windows")]
    return "./target/debug/duckdb_chess.duckdb_extension".to_string();
    
    #[cfg(target_os = "macos")]
    return "./target/debug/duckdb_chess.duckdb_extension".to_string();
    
    #[cfg(target_os = "linux")]
    return "./target/debug/duckdb_chess.duckdb_extension".to_string();
}

// Helper to get PGN test file paths
pub fn test_pgn_path(filename: &str) -> String {
    format!("tests/pgn_files/{}", filename)
}
```

**Alternatives considered:**
1. **Environment variable**: Rejected, not idiomatic
2. **Build script auto-detection**: Rejected, adds complexity
3. **Always use release build**: Rejected, slower for development

### Decision 5: Assertion Helpers for DuckDB Results

**Rationale:**
- SQLLogicTest has specific output formats
- Need ergonomic assertions for common patterns
- Reduce boilerplate in tests

**Helpers:**
```rust
// Assert single value
pub fn assert_query_result<T>(conn: &Connection, query: &str, expected: T) 
where T: PartialEq + Debug

// Assert row count
pub fn assert_row_count(conn: &Connection, query: &str, expected: usize)

// Assert column schema
pub fn assert_columns(conn: &Connection, query: &str, expected: &[(&str, &str)])

// Assert error contains message
pub fn assert_query_error(conn: &Connection, query: &str, error_contains: &str)
```

**Alternatives considered:**
1. **Use assert_eq! everywhere**: Rejected, too verbose
2. **Custom assertion macros**: Considered for future, not needed initially
3. **Property-based testing**: Deferred to future work

### Decision 6: Migration Approach

**Rationale:**
- Incremental migration reduces risk
- Parallel test suites during transition provide safety net
- Can validate coverage before removing old tests

**Phases:**
1. **Phase 1**: Set up infrastructure (common helpers, module structure)
2. **Phase 2**: Migrate scalar function tests (simpler, ~5 test files)
3. **Phase 3**: Migrate table function tests (more complex, ~6 test files)
4. **Phase 4**: Validate coverage, run both test suites
5. **Phase 5**: Remove SQLLogicTest files, update CI/CD

**Alternatives considered:**
1. **Big-bang migration**: Rejected, too risky
2. **Keep both forever**: Rejected, maintenance burden
3. **Migrate in random order**: Rejected, harder to track progress

## Technical Architecture

### Current Architecture (SQLLogicTest)
```
Developer
  ↓
pytest / duckdb-test-runner (Python)
  ↓
Parse .test files (SQLLogicTest format)
  ↓
Load extension in DuckDB Python client
  ↓
Execute queries, compare results
  ↓
Report pass/fail
```

### New Architecture (Rust Integration Tests)
```
Developer
  ↓
cargo test (native Rust)
  ↓
Compile tests/ directory
  ↓
Load extension via duckdb crate
  ↓
Execute test functions with assertions
  ↓
Report pass/fail (standard Rust test output)
```

### Test File Mapping

**Scalar Functions** (simpler):
- `chess_moves_normalize.test` → `tests/test_chess_moves.rs::test_normalize_*`
- `chess_moves_hash.test` → `tests/test_chess_moves.rs::test_hash_*`
- `chess_moves_json.test` → `tests/test_chess_moves.rs::test_json_*`
- `chess_moves_subset.test` → `tests/test_chess_moves.rs::test_subset_*`
- `chess_moves_normalize_column.test` → `tests/test_chess_moves.rs::test_normalize_column`

**Table Functions** (more complex):
- `read_pgn.test` → `tests/test_read_pgn.rs::test_basic_*`
- `read_pgn_glob.test` → `tests/test_read_pgn.rs::test_glob_*`
- `read_pgn_nulls.test` → `tests/test_read_pgn.rs::test_null_handling_*`
- `read_pgn_errors.test` → `tests/test_error_handling.rs::test_pgn_errors_*`
- `read_pgn_parse_errors.test` → `tests/test_error_handling.rs::test_parse_errors_*`
- `read_pgn_bad_utf8_real.test` → `tests/test_error_handling.rs::test_bad_utf8_*`

### Example Test Conversion

**Before** (SQLLogicTest):
```sql
# test/sql/chess_moves_normalize.test
require duckdb_chess

query I
SELECT chess_moves_normalize('1. e4 { comment } e5');
----
e4 e5

query I
SELECT chess_moves_normalize('1. e4 e5 2. Nf3');
----
e4 e5 Nf3
```

**After** (Rust):
```rust
// tests/test_chess_moves.rs
use crate::common::helpers::*;

#[test]
fn test_normalize_removes_comments() -> Result<()> {
    let conn = load_extension()?;
    assert_query_result(
        &conn,
        "SELECT chess_moves_normalize('1. e4 { comment } e5')",
        "e4 e5"
    )?;
    Ok(())
}

#[test]
fn test_normalize_basic_moves() -> Result<()> {
    let conn = load_extension()?;
    assert_query_result(
        &conn,
        "SELECT chess_moves_normalize('1. e4 e5 2. Nf3')",
        "e4 e5 Nf3"
    )?;
    Ok(())
}

// Example of table function test using PGN files
#[test]
fn test_read_pgn_sample_file() -> Result<()> {
    let conn = load_extension()?;
    let pgn_path = test_pgn_path("sample.pgn");
    
    assert_row_count(
        &conn,
        &format!("SELECT * FROM read_pgn('{}')", pgn_path),
        10  // sample.pgn has 10 games
    )?;
    Ok(())
}
```

## Risks / Trade-offs

### Risk 1: Test Coverage Gaps
**Impact:** Medium  
**Mitigation:**
- Create checklist mapping each SQLLogicTest scenario to Rust test
- Run both test suites in parallel during transition
- Use code coverage tools to validate
- Manual review of converted tests

### Risk 2: Extension Path Handling
**Impact:** Low  
**Mitigation:**
- Use cfg! macros for platform-specific paths
- Document requirement to build before testing
- Consider build.rs script to auto-build extension
- Provide clear error messages if extension not found

### Risk 3: Test Flakiness
**Impact:** Low  
**Mitigation:**
- Each test gets fresh Connection (no shared state)
- Avoid timing-dependent assertions
- Use deterministic test data
- Run tests sequentially if needed (cargo test -- --test-threads=1)

### Risk 4: Increased Test Maintenance
**Impact:** Low  
**Mitigation:**
- Well-organized helper functions reduce boilerplate
- Shared fixtures for common test data
- Clear naming conventions
- Good documentation

### Risk 5: Loss of SQLLogicTest Benefits
**Impact:** Low  
**Context:** SQLLogicTest provides standardized format, cross-database compatibility
**Mitigation:**
- We're not testing DuckDB itself, only our extension
- Rust tests are more idiomatic for this project
- Can keep .test files as documentation if desired
- Rust tests provide better IDE integration and debugging

## Migration Plan

### Phase 1: Infrastructure Setup (Day 1)
1. Create `tests/` directory structure
2. Implement `tests/common/helpers.rs` with extension loading
3. Create assertion helpers
4. Write first simple test to validate setup

### Phase 2: Migrate Scalar Function Tests (Day 1-2)
1. Convert `chess_moves_normalize.test`
2. Convert `chess_moves_hash.test`
3. Convert `chess_moves_json.test`
4. Convert `chess_moves_subset.test`
5. Convert `chess_moves_normalize_column.test`

### Phase 3: Migrate Table Function Tests (Day 2-3)
1. Convert `read_pgn.test` (basic functionality)
2. Convert `read_pgn_glob.test` (glob patterns)
3. Convert `read_pgn_nulls.test` (null handling)

### Phase 4: Migrate Error Handling Tests (Day 3)
1. Convert `read_pgn_errors.test`
2. Convert `read_pgn_parse_errors.test`
3. Convert `read_pgn_bad_utf8_real.test`

### Phase 5: Validation and Cleanup (Day 4)
1. Run both test suites, compare coverage
2. Update CI/CD to use only Rust tests
3. Update documentation
4. Optionally archive .test files or remove them

### Validation Checklist
- [ ] All SQLLogicTest scenarios have equivalent Rust tests
- [ ] All Rust integration tests pass
- [ ] Code coverage ≥ previous coverage
- [ ] Tests pass on Windows, Linux, macOS
- [ ] CI/CD runs without Python dependency
- [ ] Documentation updated

## Open Questions

1. **Should we keep .test files as documentation?**
   - Proposal: Move to `test/sql/archived/` for reference
   - Decision: TBD based on team preference

2. **Should we add code coverage reporting?**
   - Proposal: Use tarpaulin or llvm-cov in CI
   - Decision: Nice-to-have, not required for migration

3. **How to handle test data files (PGN)?**
   - Proposal: Keep in existing `test/pgn_files/` directory
   - Decision: Keep PGN files in current location (`test/pgn_files/`), reference with relative paths like `"test/pgn_files/sample.pgn"` from Rust tests
   - Rationale: 
     - PGN files are already in a good location
     - No symlinks needed (better Windows compatibility)
     - Tests run from workspace root, so paths work consistently
     - Keeps test data centralized and easy to find

4. **Should tests auto-build the extension?**
   - Proposal: Document that `cargo build` must run first
   - Alternative: Use build.rs in tests/ to trigger build
   - Decision: Document requirement, avoid build.rs complexity initially
