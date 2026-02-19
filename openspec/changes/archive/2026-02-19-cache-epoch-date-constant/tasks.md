## 1. Implementation

- [x] 1.1 Add `EPOCH` static constant using `LazyLock<NaiveDate>` in `visitor.rs`
- [x] 1.2 Update `parse_date_field` to use `EPOCH` instead of constructing epoch on each call
- [x] 1.3 Remove unreachable error-handling branch for epoch construction failure

## 2. Verification

- [x] 2.1 Run `just dev` to verify lint, build, and tests pass
- [x] 2.2 Run `just full` to verify release build and tests pass