## 1. Extend single-stage normalization coverage

- [x] 1.1 Add targeted parsing for apostrophe-per-move controls (for example `N' + I''/mv from move 1`) in `src/chess/timecontrol.rs`.
- [x] 1.2 Add compact minute/second text parsing for forms like `90m+30s`, `Standard: 90mins + 30sec increment`, and `90+30 sec per move`.
- [x] 1.3 Add clock-style base parsing for `H:MM.SS + increment` forms (for example `1:30.00 + 30 seconds increment from move 1`).
- [x] 1.4 Add guarded, language-agnostic trailing qualifier handling (no keyword dictionary) with explicit warning codes in parsed JSON output.

## 2. Extend compact staged/FIDE normalization coverage

- [x] 2.1 Implement compact FIDE apostrophe shorthand parsing for `base/40 + rest + increment` variants (including `/G`, `/end`, `/move`, `/mv`, and `&`).
- [x] 2.2 Implement bonus/additional wording variants that imply FIDE two-stage controls (for example `bonus increment`, `additional +30mins after move 40`).
- [x] 2.3 Implement compact staged shorthand handling for `90 + 30 + 30s per move` and `90+30/30+30` with explicit inference warnings.

## 3. Test and validate behavior

- [x] 3.1 Add Rust unit tests in `src/chess/timecontrol.rs` for each new normalization scenario and ambiguity guardrail.
- [x] 3.2 Add SQLLogic coverage in `test/sql/chess_timecontrol.test` for new successful normalizations and warning-bearing JSON cases.
- [x] 3.3 Run `just dev` and fix any formatting, clippy, or test failures.
- [x] 3.4 Run `just full`
