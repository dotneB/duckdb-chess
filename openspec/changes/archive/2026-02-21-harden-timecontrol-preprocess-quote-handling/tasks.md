## 1. Quote-Handling Parser Hardening

- [x] 1.1 Locate current TimeControl preprocessing quote logic in `src/chess/timecontrol.rs` and isolate quote classification responsibilities (wrapper noise vs apostrophe unit markers).
- [x] 1.2 Implement/adjust context-aware quote preprocessing so mixed outer quote wrappers are stripped without mutating numeric/operators or apostrophe unit tokens.
- [x] 1.3 Ensure ambiguous quote residue paths return safe parse failure (`normalized = NULL`) instead of stitched or partially mangled numeric output.

## 2. Cross-Function Consistency

- [x] 2.1 Verify `chess_timecontrol_normalize(...)`, `chess_timecontrol_category(...)`, and `chess_timecontrol_json(...)` all consume the same hardened preprocessing output.
- [x] 2.2 Confirm ambiguous quote cases produce aligned failure semantics across all three scalar outputs.

## 3. Regression Tests

- [x] 3.1 Add/extend Rust unit tests in `src/chess/timecontrol.rs` for mixed wrapper quotes, repeated outer quote noise, apostrophe preservation, and ambiguous unbalanced quote failures.
- [x] 3.2 Add/extend SQLLogicTests in `test/sql/chess_timecontrol.test` to lock normalize/category/json behavior for the same quote-heavy cases.

## 4. Validation

- [x] 4.1 Run `just check` and resolve formatting/lint issues.
- [x] 4.2 Run `just test` and confirm all debug tests pass with the new quote-handling coverage.
