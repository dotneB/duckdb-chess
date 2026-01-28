# Change: Update Date/Time Parsing to Use chrono

## Why
Current PGN header date/time parsing is hand-rolled and does not support Lichess-style partial dates (e.g., `2000.??.??`). Using `chrono` simplifies validation, enables partial-date defaults, and improves diagnostics by capturing parser error details.

## What Changes
- Add `chrono` as a Rust dependency for parsing `UTCDate`/`Date` and `UTCTime`/`Time` headers.
- Normalize date separators by replacing `.` with `-` before parsing.
- Extend date fallback order to include `EventDate` (`UTCDate` -> `Date` -> `EventDate`).
- Select the "best" date from the fallback chain by completeness: prefer a fully specified date over a partial date; if multiple candidates are partial, prefer the one with more specified components (e.g., `YYYY-MM-??` over `YYYY-??-??`).
- Apply partial-date defaults only after selecting the best candidate.
- Treat unknown/partial dates as follows:
  - `????.??.??` (or equivalent with `-`) becomes SQL `NULL`.
  - `YYYY.??.??` defaults to `YYYY-01-01`.
  - `YYYY.MM.??` defaults to `YYYY-MM-01`.
- Keep time formats aligned with current support (`HH:MM:SS`, optional `Z` or `+HH:MM` / `-HH:MM`), but validate the time-of-day via `chrono`.
- When `chrono` fails to parse/validate a non-empty value, include the `chrono` parse error details in `parse_error` along with the field name and original value.

## Impact
- Affected specs: `data-schema`
- Affected code: `Cargo.toml`, `src/chess/visitor.rs`, Rust unit tests under `src/chess/reader.rs` (and/or new unit tests colocated with parsing helpers)
