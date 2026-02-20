## 1. Create ErrorAccumulator Module

- [x] 1.1 Create `src/chess/error.rs` module file
- [x] 1.2 Implement `ErrorAccumulator` struct with `Option<String>` storage
- [x] 1.3 Implement `push(&mut self, msg: &str)` method with separator handling
- [x] 1.4 Implement `take(&mut self) -> Option<String>` method
- [x] 1.5 Implement `is_empty(&self) -> bool` method
- [x] 1.6 Implement `Default` trait for `ErrorAccumulator`
- [x] 1.7 Add unit tests for `ErrorAccumulator`

## 2. Update Module Exports

- [x] 2.1 Add `mod error;` to `src/chess/mod.rs`
- [x] 2.2 Export `ErrorAccumulator` from `src/chess/mod.rs`

## 3. Refactor visitor.rs

- [x] 3.1 Add `use crate::chess::error::ErrorAccumulator;` import
- [x] 3.2 Replace `parse_error: Option<String>` field with `parse_error: ErrorAccumulator`
- [x] 3.3 Replace `push_error` calls with `self.parse_error.push(msg)`
- [x] 3.4 Update `build_game_record` to use `self.parse_error.take()`

## 4. Refactor reader.rs

- [x] 4.1 Add `use crate::chess::error::ErrorAccumulator;` import
- [x] 4.2 Replace `append_parse_error` function with `ErrorAccumulator` usage
- [x] 4.3 Update all `append_parse_error` calls to use accumulator

## 5. Verification

- [x] 5.1 Run `just dev` to verify lint, build, and tests pass
- [x] 5.2 Run `just full` to verify release build and tests pass
