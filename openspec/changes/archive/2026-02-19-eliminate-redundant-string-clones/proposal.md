## Why

The `build_game_record` method clones 15+ `Option<String>` fields when constructing `GameRecord`, causing unnecessary heap allocations for every parsed game. With large PGN datasets containing millions of games, this wastes memory and CPU.

## What Changes

- Replace `Option<String>` fields in `HeaderFields` with plain `String` (empty string = None)
- Use `mem::take()` to transfer string ownership without cloning
- Update `clear()` to reset via `Default` trait assignment
- Update field access patterns to treat empty strings as None

## Capabilities

### New Capabilities

None - this is a performance optimization with no new functionality.

### Modified Capabilities

None - no spec-level requirements change. The behavior remains identical; only the implementation efficiency improves.

## Impact

- **Affected Code**: `src/chess/visitor.rs` - `HeaderFields` struct and `build_game_record` method
- **Dependencies**: Uses `std::mem::take` (standard library)
- **Performance**: ~15 allocations eliminated per game parsed; significant for large PGN files
- **API**: No changes - internal implementation detail only