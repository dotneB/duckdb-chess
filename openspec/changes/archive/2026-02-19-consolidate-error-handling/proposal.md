## Why

Error accumulation is duplicated across `visitor.rs` (`push_error` method at lines 104-114) and `reader.rs` (`append_parse_error` function at lines 137-147). Both implement the same pattern: append a message to an `Option<String>`, using `"; "` as a separator. This duplication harms maintainability and risks inconsistent behavior.

## What Changes

- Create a new `ErrorAccumulator` utility type in `src/chess/error.rs`
- Implement `push(&mut self, msg: &str)` method with separator handling
- Implement `take(&mut self) -> Option<String>` for consuming the accumulated errors
- Refactor `GameVisitor::push_error` to use `ErrorAccumulator`
- Refactor `append_parse_error` in `reader.rs` to use `ErrorAccumulator`

## Capabilities

### New Capabilities

None - this is a code organization improvement with no new functionality.

### Modified Capabilities

None - no spec-level requirements change. Error handling behavior remains identical.

## Impact

- **Affected Code**: `src/chess/visitor.rs`, `src/chess/reader.rs`, new file `src/chess/error.rs`
- **Dependencies**: None (standard library only)
- **Maintainability**: Single source of truth for error accumulation logic
- **API**: No changes - internal implementation detail only