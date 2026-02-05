## 1. Specs & Fixtures

- [x] 1.1 Add/adjust PGN fixture including a `Source` header tag
- [x] 1.2 Update SQLLogicTests that assert `read_pgn` schema/column count to expect 18 columns and include `Source`

## 2. Parsing & Schema Wiring

- [x] 2.1 Extend the internal game record/type to store `source: Option<String>`
- [x] 2.2 Update the PGN visitor/tag extraction to capture the `Source` header into the record
- [x] 2.3 Append the `Source` column to the DuckDB table function schema (column name `Source`, type VARCHAR, nullable)
- [x] 2.4 Populate the `Source` column when outputting rows (set validity mask for NULL when tag missing)

## 3. Documentation & Verification

- [x] 3.1 Update `README.md` to document the added `Source` column and the new column count
- [x] 3.2 Run `make dev` and fix any fmt/clippy/test failures
