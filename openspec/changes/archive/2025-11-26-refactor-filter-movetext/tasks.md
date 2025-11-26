# Tasks

- [x] 1. Update `src/lib.rs` to register `filter_movetext_annotations` as a scalar function. <!-- id: 1 -->
- [x] 2. Update `test/sql/filter_movetext_annotations.test` to verify scalar function usage (`SELECT filter_movetext_annotations('...')`). <!-- id: 2 -->
- [x] 3. Verify that `filter_movetext_annotations` can be used on table columns (e.g., create a temp table and select from it). <!-- id: 3 -->
- [x] 4. Remove the `FilterMovetextVTab` struct and implementation from `src/filter.rs`. <!-- id: 4 -->
