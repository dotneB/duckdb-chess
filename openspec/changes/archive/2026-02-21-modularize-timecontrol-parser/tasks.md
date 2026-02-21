## 1. TimeControl Module Decomposition

- [x] 1.1 Convert `src/chess/timecontrol.rs` into a directory module with a thin `mod.rs` facade.
- [x] 1.2 Create focused internal modules for strict parsing, inference, and JSON rendering (plus shared types/warnings as needed).
- [x] 1.3 Preserve existing public Rust/SQL entrypoints so `chess_timecontrol_normalize`, `chess_timecontrol_json`, and `chess_timecontrol_category` behavior contracts stay unchanged.

## 2. Behavior-Preserving Logic Migration

- [x] 2.1 Move strict parser logic into the strict module without changing accepted/rejected input behavior.
- [x] 2.2 Move inference and warning assignment logic into the inference module while preserving warning taxonomy and inference semantics.
- [x] 2.3 Move JSON rendering/serialization logic into the JSON module while preserving output shape, field names, and NULL semantics.
- [x] 2.4 Rewire the facade to use the new modules and remove duplicated or dead code from the legacy monolith.

## 3. Test Reorganization and Parity Coverage

- [x] 3.1 Split existing `timecontrol` tests into module-local suites aligned with strict, inference, and JSON responsibilities.
- [x] 3.2 Add/adjust compatibility tests for representative strict and inferred inputs to assert unchanged normalized outputs and warnings.
- [x] 3.3 Keep or extend SQL-facing regression coverage for the three public TimeControl scalar functions.

## 4. Validation

- [x] 4.1 Run `just check` and fix any formatting/clippy issues introduced by the refactor.
- [x] 4.2 Run `just test` and resolve regressions until existing fixtures and SQL outputs are behavior-equivalent.
