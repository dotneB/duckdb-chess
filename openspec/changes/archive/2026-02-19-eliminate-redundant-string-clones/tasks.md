## 1. HeaderFields Refactor

- [x] 1.1 Change `HeaderFields` fields from `Option<String>` to `String`
- [x] 1.2 Add helper method `opt_take(&mut String) -> Option<String>` for zero-copy transfer
- [x] 1.3 Update `set_known_tag` to check `is_empty()` instead of `is_none()`

## 2. build_game_record Update

- [x] 2.1 Replace all `.clone()` calls with `mem::take()` using helper method
- [x] 2.2 Add `use std::mem;` import if not present

## 3. Verification

- [x] 3.1 Run `just dev` to verify lint, build, and tests pass
- [x] 3.2 Run `just full` to verify release build and tests pass