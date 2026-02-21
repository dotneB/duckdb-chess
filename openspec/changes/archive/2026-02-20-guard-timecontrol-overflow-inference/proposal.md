## Why

`timecontrol` inference currently uses unchecked `u32` arithmetic when converting inferred minute-style values into seconds. For extreme inputs this can overflow in release builds, producing wrapped values that look valid but are wrong and nondeterministic for downstream category and JSON consumers.

## What Changes

- Introduce checked arithmetic helpers for all inference-path minute/second conversions (`checked_mul`, `checked_add`).
- Treat arithmetic overflow in inference as a safe parse failure for normalization output (`normalized = NULL`) instead of returning wrapped values.
- Emit deterministic warning tags on overflow so callers can distinguish parse uncertainty from invalid free-text input.
- Add boundary and overflow regression tests for normalization, category derivation, and JSON parse output.

## Capabilities

### New Capabilities
- None.

### Modified Capabilities
- `timecontrol-normalization`: Update inference requirements so overflow-prone minute/second arithmetic is checked and overflow outcomes are reported deterministically without wrapping.

## Impact

- Affects `src/chess/timecontrol.rs` inference and categorization code paths.
- Expands unit and SQLLogicTest coverage in `src/chess/timecontrol.rs` and `test/sql/chess_timecontrol.test`.
- Preserves existing outputs for non-overflow inputs while making overflow behavior explicit and stable.
