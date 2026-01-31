## 1. Spec + Semantics Alignment

- [x] 1.1 Confirm parse-failure behavior: non-NULL unparsable input returns empty string; NULL input returns NULL
- [x] 1.2 Add/adjust SQLLogicTest coverage for parse-failure empty-string behavior

## 2. Visitor-Based Normalizer

- [x] 2.1 Implement `NormalizeVisitor` using `pgn-reader` that collects mainline SAN tokens and optional outcome
- [x] 2.2 Ensure variations are skipped via `begin_variation` returning `Skip(true)`
- [x] 2.3 Ensure comments and NAGs are dropped (ignore corresponding visitor events)
- [x] 2.4 Serialize collected tokens into canonical movetext with move numbers and single-space separation; append outcome when present

## 3. Wire Into `chess_moves_normalize`

- [x] 3.1 Replace `normalize_movetext` implementation to use `pgn-reader` parsing (remove bespoke scanner/token stripper)
- [x] 3.2 Implement parse-error handling to return empty string (for non-NULL input) without fallback
- [x] 3.3 Ensure scalar wrapper preserves NULLs (NULL input -> NULL output) and does not allocate/insert invalid CString values

## 4. Tests + Regression Suite

- [x] 4.1 Update Rust unit tests in `src/chess/filter.rs` to match canonical visitor output (move numbers, spacing, result markers)
- [x] 4.2 Add unit tests for nested comments, nested variations, and mixed NAG/comment/variation inputs
- [x] 4.3 Add unit test for parse failure returning empty string (non-NULL input)
- [x] 4.4 Add/adjust SQLLogicTest(s) for `chess_moves_normalize` including parse failure and NULL propagation

## 5. Validation

- [x] 5.1 Run `make dev` and fix any fmt/clippy/test failures
- [x] 5.2 Run `make test-release-rs` and fix any release-mode failures
