# Change: Refactor Test Suite

## Why
The current test suite relies heavily on integration tests via `sqllogictest`. Core logic in `filter.rs` and `visitor.rs` lacks direct unit tests, making it harder to verify edge cases, debug logic errors, and refactor safely without running the full extension.

Additionally, test data is scattered between `test/sample.pgn` and `test/pgn_files/`, leading to inconsistent usage patterns.

## What Changes
- Add `#[cfg(test)]` modules to `src/filter.rs` and `src/visitor.rs` (or separate test files if preferred, but colocation is standard Rust).
- Implement comprehensive unit tests for `filter_movetext_annotations` covering nested annotations, whitespace normalization, and edge cases.
- Implement unit tests for `GameVisitor` by feeding it mock PGN data via `pgn_reader::Reader` and asserting on the resulting `GameRecord`.
- Consolidate test data by moving `test/sample.pgn` to `test/pgn_files/sample.pgn` and updating all references.
- Update `openspec/project.md` to explicitly state that `make debug` or `make release` must be run before `make test`.
- Update `code-structure` spec to explicitly require unit test support for core logic and organized test data.

## Impact
- **Affected specs**: `code-structure`
- **Affected code**: `src/filter.rs`, `src/visitor.rs`, `src/lib.rs` (if module visibility changes needed)
- **Affected tests**: `test/sql/*.test`, `test_chess.sql`, `test_direct.sql`
- **Documentation**: `openspec/project.md`
