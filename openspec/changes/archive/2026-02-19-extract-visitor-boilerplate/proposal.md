## Why

Six `pgn-reader::Visitor` implementations across `moves.rs`, `filter.rs`, and `visitor.rs` share identical boilerplate methods (`nag`, `comment`, `partial_comment`, `begin_variation`). This ~50 lines of repetitive code harms maintainability and makes adding new visitors tedious.

## What Changes

- Create a `impl_pgn_visitor_boilerplate!` macro that generates the common no-op Visitor methods
- Apply macro to all 6 visitor implementations to eliminate repetitive code
- Centralize "skip variations, ignore NAGs/comments" behavior in one place

## Capabilities

### New Capabilities

None - this is a code organization improvement with no new functionality.

### Modified Capabilities

None - no spec-level requirements change. All visitor behavior remains identical.

## Impact

- **Affected Code**: `src/chess/moves.rs`, `src/chess/filter.rs`, `src/chess/visitor.rs`
- **Dependencies**: None (uses declarative macros, a built-in Rust feature)
- **Maintainability**: Adding new visitors becomes ~8 lines instead of ~20 lines
- **API**: No changes - internal implementation detail only