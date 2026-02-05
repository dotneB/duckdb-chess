## Why

PGN exports often include a `Source` header (origin/attribution, import pipeline, or dataset provenance), but `read_pgn` currently drops it. Exposing it as a column makes it easy to filter/trace games by origin without parsing raw PGN text.

## What Changes

- `read_pgn` parses the PGN header tag `Source` and returns it in a new nullable `Source` column.
- The `read_pgn` output schema is extended from 17 to 18 columns.
- The new column is appended to the end of the table function output to preserve existing column order as much as possible.

## Capabilities

### New Capabilities

<!-- None -->

### Modified Capabilities

- `data-schema`: extend the `read_pgn` schema to include a nullable `Source` column and update the documented column count.

## Impact

- User-visible: `read_pgn` schema/README update; SQLLogicTests updated to reflect the extra column.
- Code: PGN visitor/tag extraction, record types, and DuckDB table function column definitions.
