## 1. Implementation
- [x] 1.1 Update `read_pgn` bind schema to use `DATE`, `TIMETZ`, and `UINTEGER` for the specified columns <!-- id: 1 -->
- [x] 1.2 Update `GameRecord` types and visitor parsing to produce typed values (including fallback headers) <!-- id: 2 -->
- [x] 1.3 Ensure conversion failures yield `NULL`, append a conversion message to `parse_error`, and do not stop parsing <!-- id: 3 -->

## 2. Tests
- [x] 2.1 Update SQLLogicTest schema assertions in `test/sql/read_pgn.test` to expect `DATE`, `TIMETZ`, `UINTEGER` <!-- id: 4 -->
- [x] 2.2 Add/extend a SQLLogicTest case validating successful conversion for a known PGN (date/time/elo) <!-- id: 5 -->
- [x] 2.3 Add/extend a SQLLogicTest case validating invalid-but-non-empty values map to `NULL` and set `parse_error` (parsing continues) <!-- id: 6 -->

## 3. Docs & Validation
- [x] 3.1 Update `README.md` `read_pgn` returned column types table <!-- id: 7 -->
- [x] 3.2 Run `cargo test` <!-- id: 8 -->
- [x] 3.3 Run SQLLogicTest suite (`duckdb-slt.exe ... test/sql/*.test`) <!-- id: 9 -->
