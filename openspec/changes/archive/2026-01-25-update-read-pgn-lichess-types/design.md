## Context
Lichess PGN headers store timestamps and ratings as strings. Today, `read_pgn()` keeps them as `VARCHAR`, but DuckDB users typically want typed columns to join against typed datasets and avoid repeated casts.

## Goals / Non-Goals
- Goals:
  - Return typed columns for `UTCDate`, `UTCTime`, `WhiteElo`, `BlackElo`.
  - Preserve existing column names, order, nullability, and fallback header behavior.
  - Preserve existing column names, order, nullability, and fallback header behavior.
  - Preserve `read_pgn()`'s non-fatal posture: conversion failures do not stop parsing.
  - Surface non-empty conversion failures via `parse_error` for traceability.
- Non-Goals:
  - Adding a second “raw string” representation of these fields.
  - Making conversion failures fatal for a game or file.

## Decisions
- Decision: Parse `UTCDate` into `DATE`.
  - Supported inputs: `YYYY.MM.DD` (Lichess PGN), and optionally `YYYY-MM-DD` if present.
  - Unknown/partial dates (e.g., `????.??.??`) parse to `NULL`.
  - If the chosen source header (prefer `UTCDate`, else `Date`) is present and non-empty but fails to parse, append a conversion error to `parse_error`.
- Decision: Parse `UTCTime` into `TIMETZ` and treat it as UTC.
  - Supported inputs: `HH:MM:SS` (Lichess PGN). The produced `TIMETZ` uses `+00:00`.
  - If a timezone offset is present in input (rare in PGN), preserve it.
  - Invalid times parse to `NULL`.
  - If the chosen source header (prefer `UTCTime`, else `Time`) is present and non-empty but fails to parse, append a conversion error to `parse_error`.
- Decision: Parse `WhiteElo`/`BlackElo` into `UINTEGER`.
  - Invalid, negative, or out-of-range values parse to `NULL`.
  - If the header is present and non-empty but fails to parse, append a conversion error to `parse_error`.
 - Decision: Multiple conversion issues may occur in one row.
  - If more than one conversion error occurs, `parse_error` contains a single string with messages joined by `; `.
  - Conversion errors MUST NOT replace an existing parsing error; they append to it.

## Risks / Trade-offs
- This is a breaking schema change for users who rely on `VARCHAR` behavior (e.g., `LIKE` on Elo/date/time). Mitigation: users can `CAST(... AS VARCHAR)` in queries.
- Some PGNs may contain unexpected formats; treating them as `NULL` avoids query failure, while recording conversion errors in `parse_error` adds traceability.

## Migration Plan
- Users can migrate existing queries by:
  - `CAST(WhiteElo AS VARCHAR)` / `CAST(BlackElo AS VARCHAR)` if string behavior is needed.
  - `strftime(UTCDate, '%Y.%m.%d')` and `strftime(UTCTime, '%H:%M:%S%z')` for formatting.

## Open Questions
- None (assumes `UTCTime` is UTC and should be represented with `+00:00`).
