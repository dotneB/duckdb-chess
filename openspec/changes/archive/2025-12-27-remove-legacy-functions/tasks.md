# Implementation Tasks

## 1. Remove Legacy Function Registrations
- [x] 1.1 Remove `filter_movetext_annotations` registration from `src/chess/mod.rs:20`
- [x] 1.2 Remove `moves_json` registration from `src/chess/mod.rs:21`
- [x] 1.3 Remove legacy function comment from `src/chess/mod.rs:19`

## 2. Update Test Files
- [x] 2.1 Remove backward compatibility test section from `test/sql/filter_movetext_annotations.test:104-106`
- [x] 2.2 Remove backward compatibility test section from `test/sql/moves_json.test:50-52`
- [x] 2.3 Update `filter_movetext_annotations` calls to `chess_moves_normalize` in `test/sql/filter_movetext_column.test:14`
- [x] 2.4 Update test file description in `test/sql/filter_movetext_column.test:2-3` to reference `chess_moves_normalize`

## 3. Validation
- [x] 3.1 Run `cargo build --release` to ensure code compiles
- [x] 3.2 Run `make test_release` to verify all tests pass with new function names
- [x] 3.3 Manually verify legacy function names are no longer available in DuckDB

## 4. Documentation
- [x] 4.1 Update `src/chess/filter.rs:10-11` comment to remove "Legacy function" notice from `filter_movetext_annotations` implementation
- [x] 4.2 Verify no references to legacy names remain in code comments
