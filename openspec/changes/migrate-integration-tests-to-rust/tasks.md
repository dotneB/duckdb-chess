# Implementation Tasks

## 1. Infrastructure Setup
- [ ] 1.1 Create `tests/` directory for integration tests
- [ ] 1.2 Create `tests/common/mod.rs` and `tests/common/helpers.rs` for shared utilities
- [ ] 1.3 Implement `load_extension()` helper with platform-specific path detection
- [ ] 1.4 Implement `test_pgn_path()` helper to reference PGN files in `test/pgn_files/`
- [ ] 1.5 Implement assertion helpers (`assert_query_result`, `assert_row_count`, etc.)
- [ ] 1.6 Write first smoke test to validate infrastructure works
- [ ] 1.7 Document test execution requirements (must build extension first)
- [ ] 1.8 Verify PGN test files in `test/pgn_files/` are accessible from tests

## 2. Migrate Scalar Function Tests
- [ ] 2.1 Create `tests/test_chess_moves.rs` module
- [ ] 2.2 Migrate `chess_moves_normalize.test` scenarios (~20 test cases)
- [ ] 2.3 Migrate `chess_moves_hash.test` scenarios (~15 test cases)
- [ ] 2.4 Migrate `chess_moves_json.test` scenarios (~10 test cases)
- [ ] 2.5 Migrate `chess_moves_subset.test` scenarios (~18 test cases)
- [ ] 2.6 Migrate `chess_moves_normalize_column.test` scenarios (~3 test cases)
- [ ] 2.7 Verify all scalar function tests pass

## 3. Migrate Table Function Basic Tests
- [ ] 3.1 Create `tests/test_read_pgn.rs` module
- [ ] 3.2 Migrate `read_pgn.test` schema validation test
- [ ] 3.3 Migrate `read_pgn.test` row count tests
- [ ] 3.4 Migrate `read_pgn.test` filtering and projection tests
- [ ] 3.5 Migrate `read_pgn_glob.test` glob pattern tests
- [ ] 3.6 Verify basic table function tests pass

## 4. Migrate Null and Edge Case Tests
- [ ] 4.1 Migrate `read_pgn_nulls.test` null handling scenarios (~15 test cases)
- [ ] 4.2 Test empty PGN files
- [ ] 4.3 Test files with missing headers
- [ ] 4.4 Verify null handling tests pass

## 5. Migrate Error Handling Tests
- [ ] 5.1 Create `tests/test_error_handling.rs` module
- [ ] 5.2 Migrate `read_pgn_errors.test` general error scenarios
- [ ] 5.3 Migrate `read_pgn_parse_errors.test` malformed PGN tests
- [ ] 5.4 Migrate `read_pgn_bad_utf8_real.test` encoding error tests
- [ ] 5.5 Implement `assert_query_error` helper for error testing
- [ ] 5.6 Verify all error handling tests pass

## 6. Test Coverage Validation
- [ ] 6.1 Create checklist mapping each SQLLogicTest scenario to Rust test
- [ ] 6.2 Run both test suites in parallel, compare results
- [ ] 6.3 Verify test count: SQLLogicTest scenarios vs Rust tests
- [ ] 6.4 Check for missing edge cases or scenarios
- [ ] 6.5 Add any missing tests identified

## 7. CI/CD Updates
- [ ] 7.1 Update `.github/workflows/MainDistributionPipeline.yml` test job
- [ ] 7.2 Remove Python installation step from CI
- [ ] 7.3 Ensure `cargo build` runs before `cargo test` in CI
- [ ] 7.4 Verify CI passes on all platforms (Linux, macOS, Windows)
- [ ] 7.5 Test CI with both test suites during transition

## 8. Documentation Updates
- [ ] 8.1 Update `README.md` Testing section
- [ ] 8.2 Remove Python test runner mentions
- [ ] 8.3 Document `cargo test` usage
- [ ] 8.4 Add note about building extension before testing
- [ ] 8.5 Update `openspec/project.md` Testing Strategy section
- [ ] 8.6 Document new test organization structure

## 9. Cleanup and Migration Finalization
- [ ] 9.1 Move `.test` files to `test/sql/archived/` or remove them
- [ ] 9.2 Remove Python test dependencies if any remain
- [ ] 9.3 Update Makefile test targets to use cargo test
- [ ] 9.4 Remove old test runner scripts/configs
- [ ] 9.5 Final validation: all tests pass with Rust-only workflow

## Dependencies
- Tasks 2.x-5.x depend on 1.x (need infrastructure before writing tests)
- Task 6.x depends on 2.x-5.x (need all tests migrated before validation)
- Task 7.x can be done in parallel with 2.x-5.x (CI updates independent)
- Tasks 8.x and 9.x depend on 6.x (documentation reflects final state)

## Parallelizable Work
- 2.x, 3.x, 4.x, 5.x can be done in parallel (independent test modules)
- 7.x can start early (prepare CI while migrating tests)
- 8.x can draft documentation while tests are being written

## Estimated Effort
- Phase 1 (Infrastructure): 2-4 hours
- Phase 2 (Scalar tests): 4-6 hours
- Phase 3-5 (Table/error tests): 6-8 hours
- Phase 6 (Validation): 2-3 hours
- Phase 7-9 (CI/CD, docs, cleanup): 2-3 hours
- **Total**: 16-24 hours (2-3 days)

## Test Count Estimation
- Current SQLLogicTest: ~11 files, estimated 80-100 test scenarios
- Target Rust tests: ~80-100 test functions
- Expected ratio: 1:1 or slightly more (can split complex scenarios)
