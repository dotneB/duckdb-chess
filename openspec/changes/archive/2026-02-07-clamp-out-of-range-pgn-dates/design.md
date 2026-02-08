## Context

`read_pgn` currently parses `UTCDate` candidates (`UTCDate`, `Date`, `EventDate`) by normalizing separators and then using `chrono` date parsing. For malformed values where year/month are valid but day exceeds the month limit (for example `2015.11.31` or `1997.02.29`), conversion fails and produces `parse_error` entries like `chrono: input is out of range`.

The requested behavior is to preserve year/month fidelity for these common PGN data quality issues by replacing an out-of-range day with the last valid day of that month.

## Goals / Non-Goals

**Goals:**
- Clamp known out-of-range day values to month end when year and month are valid.
- Keep existing date fallback precedence and partial-date handling (`?` month/day defaults).
- Reduce non-actionable conversion errors for month-end overflow inputs while preserving deterministic row output.
- Add explicit tests for 30-day months and February leap/non-leap boundaries.

**Non-Goals:**
- Do not infer missing year/month/day beyond current `?` handling semantics.
- Do not accept invalid year/month values (these remain conversion failures or NULL outcomes per current rules).
- Do not change time parsing (`UTCTime`) or unrelated field conversions.

## Decisions

1. Add a date normalization step before final `NaiveDate` construction.
   - Parse numeric year/month/day components after existing separator and `?` normalization.
   - Compute the last day of the target month for the parsed year/month.
   - If provided day is greater than month maximum, replace it with month maximum.
   - Then build the final date and continue existing epoch-range conversion.
   - Alternatives considered:
     - Retry parsing after matching `chrono` error strings: rejected as brittle and error-message dependent.
     - Skip invalid primary and rely on fallback headers only: rejected because valid year/month from primary would still be discarded.

2. Treat successful clamping as successful conversion, not as an error.
   - Rows that normalize successfully keep `parse_error` unchanged for date conversion.
   - Conversion errors remain for malformed structure, invalid year/month, or out-of-range epoch conversion.
   - Alternatives considered:
     - Record a warning in `parse_error` for every clamp: rejected to avoid noisy diagnostics for recoverable normalization.

3. Validate behavior with both unit and SQL-level coverage.
   - Extend Rust tests around date conversion and fallback chaining.
   - Add SQLLogicTest cases that assert normalized `UTCDate` values for malformed day inputs.
   - Alternatives considered:
     - Unit tests only: rejected because user-visible SQL behavior must be covered end-to-end.

## Risks / Trade-offs

- [Silent normalization can mask source-data mistakes] -> Mitigation: constrain normalization to day-overflow only and keep invalid month/year failures explicit.
- [Leap-year handling mistakes could produce wrong February results] -> Mitigation: derive month length via `chrono` date construction rather than hardcoded tables.
- [Behavior change affects downstream queries expecting NULL on overflow] -> Mitigation: capture the change in specs/tests so consumers have a stable contract.
