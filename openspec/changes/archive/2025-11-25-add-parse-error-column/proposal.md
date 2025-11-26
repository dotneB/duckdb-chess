# Change: Add Parse Error Column for Game Diagnostics

## Why

Currently, when the PGN parser encounters a malformed or invalid game, it logs a warning to stderr and skips the game entirely. This approach has several limitations:

- **Lost Data**: Games with partial parsing errors are completely discarded, losing potentially valuable information
- **Poor Observability**: Users cannot easily identify which games failed to parse or why
- **Difficult Debugging**: No way to query and analyze problematic games within SQL
- **Data Quality Issues**: Users cannot distinguish between "no games found" and "games exist but failed to parse"

By adding a parse error column to the output schema, users can:
- Query and filter games that encountered parsing issues using SQL
- Investigate data quality problems systematically
- Make informed decisions about whether to fix the source PGN or handle errors differently
- Maintain partial game data even when parsing fails

## What Changes

- **NEW COLUMN**: Add `parse_error` column (VARCHAR, nullable) to the `read_pgn` output schema
  - Contains NULL for successfully parsed games
  - Contains error message string for games that encountered parsing errors
  - Allows users to query `WHERE parse_error IS NOT NULL` to find problematic games

- **MODIFIED BEHAVIOR**: Instead of completely skipping malformed games, output them with:
  - All successfully parsed fields (headers and/or movetext) depending on where parsing failed
  - The error message in the `parse_error` column indicating what failed
  - Errors can occur at multiple stages:
    - **Header parsing errors**: Output minimal game record with error (e.g., malformed Event header)
    - **Movetext parsing errors**: Output headers with empty/partial movetext and error
    - **Line reading errors**: Currently skipped but could be captured with file/line context
  - This preserves whatever data was successfully parsed before the error

- **SCHEMA CHANGE**: Updates from 16 to 17 columns
  - **BREAKING**: Extends the Lichess-compatible schema with an additional column
  - Note: This is technically breaking for code that expects exactly 16 columns, but adds value

## Impact

**Affected specs:**
- Modifies: `data-schema` (adds new column requirement)
- Modifies: `pgn-parsing` (changes malformed game handling behavior)

**Affected code:**
- `src/lib.rs`: 
  - Add `parse_error` field to `GameRecord` struct
  - Modify bind phase to add 17th column
  - Update error handling to capture game data with error messages
  - Update output phase to insert parse_error column values

**Benefits:**
- Improved data observability and debugging capabilities
- No data loss for partially valid games
- Better error reporting for data quality issues at all parsing stages
- Enables SQL-based analysis of parsing problems
- Can identify specific types of errors (header vs movetext vs file reading)

**Trade-offs:**
- **BREAKING**: Schema changes from 16 to 17 columns
- Slightly larger output (additional column overhead)
- More complex error handling logic

**Migration Considerations:**
- Users expecting exactly 16 columns will need to adjust queries
- Existing queries using `SELECT *` will get an additional column
- Column-based selection (`SELECT Event, Site, ...`) remains compatible if parse_error not included
