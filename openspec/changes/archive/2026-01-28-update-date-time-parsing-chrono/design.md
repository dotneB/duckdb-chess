## Context
The `read_pgn` table function parses `UTCDate`/`Date` into DuckDB `DATE` and `UTCTime`/`Time` into DuckDB `TIMETZ`. Today this uses custom string splitting and validation.

Lichess (and some PGN sources) can emit unknown/partial dates using `?` placeholders (e.g., `????.??.??`, `2000.??.??`, `2000.06.??`). The current implementation treats any `?` in the date as unknown and returns NULL, losing partial information.

## Goals / Non-Goals
- Goals:
  - Use `chrono` to validate and parse date/time fields.
  - Support partial-date defaults as requested (unknown month/day default to `01`).
  - Preserve existing field fallback behavior (`UTCDate` from `Date`, `UTCTime` from `Time`).
  - Improve `parse_error` diagnostics by including `chrono`'s parse/validation error.
- Non-Goals:
  - Change the output schema or DuckDB types.
  - Expand accepted time formats beyond `HH:MM:SS` plus optional `Z` / `+HH:MM` / `-HH:MM`.

## Decisions
- Dependency: add `chrono` (crate `chrono`) and use its `NaiveDate` / `NaiveTime` parsing and validation.
- Date normalization: replace `.` with `-` before parsing to unify the handling of `YYYY.MM.DD` and `YYYY-MM-DD`.
- Date candidate set: consider at most one date value from each header in the chain `UTCDate`, `Date`, `EventDate`.
- Candidate selection: choose the "best" available date by completeness:
  - Prefer a fully specified date (`YYYY-MM-DD`) over any partial date.
  - If no fully specified date exists, prefer `YYYY-MM-??` over `YYYY-??-??`.
  - If multiple candidates have the same completeness, break ties by header precedence (`UTCDate` first, then `Date`, then `EventDate`).
- Partial defaults: after selecting a candidate, replace unknown components (`??`) with `01` before `chrono` validation/parsing.
- Partial dates:
  - If year is unknown (`????`), the date is treated as unknown and yields SQL `NULL` without a conversion error.
  - If year is known but month/day are unknown (`??`), default unknown parts to `01` and validate with `chrono`.
  - If a partially-specified date cannot be interpreted unambiguously (e.g., non-numeric year, or invalid month/day ranges after defaulting), yield SQL `NULL` and record a conversion error.
- Error messaging:
  - For non-empty values that fail parsing/validation, the conversion error string includes the field label, the original value, and the `chrono` error message.

## Risks / Trade-offs
- Adds a new dependency that may increase build time and binary size.
- `chrono` error messages can be verbose; messages will be included as-is to improve diagnosability.

## Migration Plan
- Implement parsing changes behind the existing parsing helpers.
- Update/add unit tests for date/time conversions, including partial date defaults and error strings.

## Open Questions
- None (formats and defaults are specified in the change request).
