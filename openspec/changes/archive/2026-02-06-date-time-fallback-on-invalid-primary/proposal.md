## Why

Date/time candidate selection currently prioritizes precedence and completeness before conversion validity, so an invalid primary header can prevent recovery from valid fallback headers. This causes avoidable `NULL` values in `UTCDate`/`UTCTime` and loses useful typed data.

## What Changes

- Update date conversion logic so invalid selected primary candidates continue fallback evaluation (`UTCDate` -> `Date` -> `EventDate`).
- Update time conversion logic so invalid `UTCTime` can fall back to `Time`.
- Preserve existing precedence/completeness policy among parseable candidates.
- Keep conversion diagnostics in `parse_error` while still recovering valid typed values when a later fallback candidate succeeds.
- Add tests for mixed-validity inputs, including precedence and partial-date rules.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `data-schema`: refine date/time fallback behavior so invalid primary headers do not block valid secondary candidates while keeping conversion error reporting.

## Impact

- Affected code: `src/chess/visitor.rs` date/time candidate parsing and selection helpers.
- Affected tests: unit tests in `src/chess/reader.rs` and SQLLogicTests under `test/sql/read_pgn_*.test`.
- User-visible effect: more rows recover non-NULL `UTCDate`/`UTCTime` from fallback headers with diagnostics retained in `parse_error`.
