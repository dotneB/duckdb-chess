## Why

PGN `TimeControl` tags in real-world datasets frequently deviate from the seconds-based PGN specification (e.g. `3+2`, `15+10`, punctuation variants, and free-text descriptions). This prevents reliable aggregation and makes it difficult to compute or filter by actual time limits without bespoke cleanup.

## What Changes

- Add a lenient `TimeControl` parser/normalizer that converts common non-standard inputs into a canonical, spec-shaped, seconds-based representation when it can do so with high confidence.
- Expose new `chess_timecontrol_*` scalar function(s) to return:
  - a canonical normalized `TimeControl` string (or NULL on failure)
  - an optional structured representation (e.g. JSON) including periods, increments, and explicit warnings when inference was applied
- Preserve existing `read_pgn` behavior by continuing to output the raw `TimeControl` tag value as-is (no column additions or replacements).
- Add unit tests and SQLLogicTests covering representative non-standard inputs observed in `timecontrolfreq.csv`.

## Capabilities

### New Capabilities
- `timecontrol-normalization`: Parse and normalize PGN `TimeControl` tag values into a canonical seconds-based PGN format, with structured output and explicit inference warnings.

### Modified Capabilities

(none)

## Impact

- New public SQL API surface: one or more `chess_timecontrol_*` scalar functions.
- Rust implementation: new parsing/normalization module and extension registration updates.
- Tests: new Rust unit tests and `test/sql/*.test` coverage for NULL behavior and common non-standard forms.
- Documentation: update README and/or specs to describe the new functions and canonicalization semantics.
