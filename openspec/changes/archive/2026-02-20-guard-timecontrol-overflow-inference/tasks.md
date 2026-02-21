## 1. Inference Arithmetic Hardening

- [x] 1.1 Identify every inference-path minute/second arithmetic site in `src/chess/timecontrol.rs` and route it through shared checked helpers.
- [x] 1.2 Implement checked conversion/composition helpers (`checked_mul`/`checked_add`) for minute-to-second and base-plus-increment calculations used by inference.
- [x] 1.3 Replace direct arithmetic in shorthand inference branches with helper calls so overflow cannot wrap in release builds.

## 2. Deterministic Overflow Semantics

- [x] 2.1 Define a stable overflow warning code (for example `inference_arithmetic_overflow`) and attach it when checked arithmetic fails.
- [x] 2.2 Ensure overflowed inference results degrade safely to `normalized = NULL` instead of emitting wrapped values.
- [x] 2.3 Keep non-overflow inference and canonical parse outputs unchanged, including existing warning behavior.

## 3. Cross-Function Consistency

- [x] 3.1 Ensure `chess_timecontrol_category(...)` uses overflow-safe parse results and returns NULL for overflowed inference values.
- [x] 3.2 Ensure `chess_timecontrol_json(...)` surfaces `normalized: null` plus the overflow warning code for overflowed inference values.
- [x] 3.3 Verify normalize/category/json stay aligned for both overflow and non-overflow inputs.

## 4. Tests and Validation

- [x] 4.1 Add/extend unit tests for boundary and overflow inference conversions (including exact-boundary success and just-over-boundary failure cases).
- [x] 4.2 Add/extend SQLLogicTests in `test/sql/chess_timecontrol.test` for overflow normalization/category/JSON outcomes and non-overflow stability checks.
- [x] 4.3 Run `just check` and fix any formatting/lint findings.
- [x] 4.4 Run `just test` and confirm full debug test pass.
