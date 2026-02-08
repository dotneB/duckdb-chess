## Why

Some PGN datasets (for example, LumbrasGigaBase) include date tags where the year and month are correct but the day is out of range for that month (such as `2015.11.31`). Today these values fail conversion (`chrono: input is out of range`) and lose otherwise useful temporal information.

## What Changes

- Normalize out-of-range day values by clamping to the last valid day of the parsed month when year and month are valid.
- Apply this normalization to date candidates used for `UTCDate` resolution (`UTCDate`, fallback `Date`, fallback `EventDate`) before typed conversion.
- Preserve existing behavior for missing/unknown components (`?`) and truly invalid year/month values.
- Extend tests to cover month-end clamping cases (30-day months and February leap/non-leap boundaries).

## Capabilities

### New Capabilities
- None.

### Modified Capabilities
- `data-schema`: update `UTCDate` conversion requirements to preserve year/month by clamping invalid day values to month end instead of failing conversion.

## Impact

- Affected area: date conversion logic used by `read_pgn` typed `UTCDate` output.
- User-visible behavior: fewer NULL `UTCDate` values for malformed day components; normalized dates become queryable.
- Validation: add/update SQLLogicTests and unit tests for representative malformed PGN date inputs.
