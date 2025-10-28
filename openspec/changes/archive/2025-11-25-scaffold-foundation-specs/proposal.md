# Change: Scaffold Foundation Specifications and Improve Code Quality

## Why

The project currently has no formal specifications documenting the existing functionality. This makes it difficult to:
- Track what features are implemented and their expected behavior
- Ensure consistency when making changes
- Onboard new contributors who need to understand the system
- Identify gaps in test coverage and code quality

By scaffolding specifications that reflect the current implementation, we establish a solid foundation for future development.

## What Changes

- **NEW**: Create comprehensive specifications for three core capabilities:
  - `pgn-parsing`: PGN file reading with glob pattern support and Lichess schema compatibility
  - `annotation-filtering`: Movetext annotation removal functionality
  - `data-schema`: Lichess-compatible output schema definition
  
- **IMPROVEMENT**: Address identified code quality issues:
  - Implement proper NULL handling for optional fields (currently using empty strings)
  - Add comprehensive error handling tests
  - Expand test coverage for edge cases (malformed PGN, empty files, invalid paths)
  
- **ENHANCEMENT**: Document testing patterns and validation requirements

## Impact

**Affected specs:**
- Creates new: `pgn-parsing`, `annotation-filtering`, `data-schema`

**Affected code:**
- `src/lib.rs` - Improve NULL handling (line 354)
- `test/sql/` - Add new test cases for error scenarios and edge cases

**Benefits:**
- Establishes clear requirements for existing functionality
- Documents expected behavior for all table functions
- Provides foundation for future feature development
- Improves code robustness through better NULL handling
- Ensures comprehensive test coverage

**Non-breaking changes:**
- Fixing NULL handling is backward-compatible (improves correctness)
- Additional tests don't affect existing functionality
