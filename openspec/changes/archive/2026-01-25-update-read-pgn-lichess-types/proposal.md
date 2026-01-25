# Change: Update read_pgn Column Types For Lichess Compatibility

## Why
`read_pgn()` currently returns `UTCDate`, `UTCTime`, `WhiteElo`, and `BlackElo` as `VARCHAR`, which makes it awkward to union/join with Lichess dataset tables that use native `DATE`, `TIMETZ`, and unsigned integer rating columns.

## What Changes
- **BREAKING**: Change `read_pgn()` output types:
  - `UTCDate`: `VARCHAR` -> `DATE`
  - `UTCTime`: `VARCHAR` -> `TIMETZ`
  - `WhiteElo`: `VARCHAR` -> `UINTEGER`
  - `BlackElo`: `VARCHAR` -> `UINTEGER`
- Keep all other columns unchanged.
- Keep fallback behavior (`UTCDate` from `Date`, `UTCTime` from `Time`).
- Invalid or unknown values for these columns become `NULL`. If the value was not empty but failed to parse, add the error to parse_error but continue parsing.

## Impact
- Affected specs: `data-schema`
- Affected code:
  - `src/chess/reader.rs` (table function bind schema + vector writes)
  - `src/chess/visitor.rs` / `src/chess/types.rs` (typed parsing/storage)
  - SQLLogicTests under `test/sql/` that assert schema types
  - `README.md` API reference table for `read_pgn`
