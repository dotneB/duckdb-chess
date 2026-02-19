## Why

Every parsed game date field recalculates `NaiveDate::from_ymd_opt(1970, 1, 1)` to compute days-since-epoch. This constant epoch date is computed millions of times when parsing large PGN datasets, wasting CPU cycles on a value that never changes.

## What Changes

- Replace repeated epoch date construction with a cached static constant
- Use `LazyLock<NaiveDate>` for thread-safe, one-time initialization
- Remove unnecessary `match` error handling (epoch construction never fails)

## Capabilities

### New Capabilities

None - this is a performance optimization with no new functionality.

### Modified Capabilities

None - no spec-level requirements change. The behavior remains identical; only the implementation efficiency improves.

## Impact

- **Affected Code**: `src/chess/visitor.rs` - `parse_date_field` method
- **Dependencies**: Uses `std::sync::LazyLock` (stable in Rust 1.80+, project MSRV is 1.89)
- **Performance**: Eliminates function call and branch on every date parse; significant for large PGN files (millions of games)
- **API**: No changes - internal implementation detail only