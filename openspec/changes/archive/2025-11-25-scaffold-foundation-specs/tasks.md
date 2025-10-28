# Implementation Tasks

## 1. Specification Creation
- [x] 1.1 Create `pgn-parsing` capability spec with requirements and scenarios
- [x] 1.2 Create `annotation-filtering` capability spec with requirements and scenarios
- [x] 1.3 Create `data-schema` capability spec with Lichess schema requirements
- [x] 1.4 Validate all specs with `openspec validate scaffold-foundation-specs --strict`

## 2. Code Quality Improvements
- [x] 2.1 Implement proper NULL handling in `src/lib.rs:354` (replace empty strings with actual NULL values)
- [x] 2.2 Add vector validity mask setting for NULL values in output columns
- [x] 2.3 Verify NULL handling works correctly with DuckDB
- [x] 2.4 Run existing tests to ensure no regressions

## 3. Test Coverage Expansion
- [x] 3.1 Create `test/sql/read_pgn_nulls.test` for NULL value scenarios
- [x] 3.2 Create `test/sql/read_pgn_errors.test` for error handling (invalid paths, malformed PGN)
- [x] 3.3 Create `test/sql/filter_movetext_annotations.test` for annotation filtering edge cases
- [x] 3.4 Add tests for empty PGN files and files with no valid games
- [x] 3.5 Add tests for glob patterns that match no files

## 4. Documentation
- [x] 4.1 Ensure all new test files include descriptive headers
- [x] 4.2 Update code comments to reference spec requirements where applicable
- [x] 4.3 Document NULL handling behavior in relevant code sections

## 5. Validation
- [x] 5.1 Run `make test_debug` and ensure all tests pass
- [x] 5.2 Run `make test_release` and ensure all tests pass
- [x] 5.3 Manually test with various PGN files including edge cases
- [x] 5.4 Verify specs are complete and accurate with `openspec validate --strict`
