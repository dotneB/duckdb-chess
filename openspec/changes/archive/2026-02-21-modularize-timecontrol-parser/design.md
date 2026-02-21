## Context

`src/chess/timecontrol.rs` currently holds several concerns in one place: strict PGN-shaped parsing, real-world shorthand inference, warning tagging, JSON rendering, and an extensive test suite. The module works, but its size and mixed responsibilities make it harder to review and evolve safely. The change must preserve the existing SQL contract (`chess_timecontrol_normalize`, `chess_timecontrol_json`, `chess_timecontrol_category`) and keep warning/inference semantics stable.

## Goals / Non-Goals

**Goals:**
- Split TimeControl logic into focused modules with explicit responsibility boundaries.
- Preserve SQL-visible behavior, including normalized outputs, warning taxonomy, inference flags, and NULL behavior.
- Keep public Rust/SQL entrypoints stable so extension wiring and callers do not change.
- Improve test navigability by grouping strict, inference, and JSON behavior tests near the responsible module while keeping compatibility coverage.

**Non-Goals:**
- No new parsing heuristics or warning codes.
- No changes to SQL function names, signatures, or return shapes.
- No performance rewrite beyond incidental effects of moving code.

## Decisions

- Convert `timecontrol` into a directory module with a thin facade (`src/chess/timecontrol/mod.rs`) and focused internals (`strict`, `inference`, `json`, plus shared model/warning types as needed).
  - Alternative considered: keep one file and only add section markers. Rejected because it does not reduce coupling enough for future maintenance.
- Keep one canonical parse/intermediate representation shared by strict parsing, inference normalization, and JSON rendering.
  - Alternative considered: separate representations per module with conversions. Rejected because it increases duplication and drift risk for warnings/flags.
- Centralize warning code definitions and inference classification in one module/type so strict and inferred paths emit the same taxonomy as today.
  - Alternative considered: hardcode warning strings inside each module. Rejected due to high risk of accidental taxonomy changes.
- Keep top-level public functions and SQL registration untouched; only internal call graph/module locations change.
  - Alternative considered: expose new public helper API. Rejected to keep backward compatibility and minimize migration scope.
- Reorganize tests into module-local suites plus a compact compatibility suite that checks representative fixtures for behavior parity.
  - Alternative considered: move all tests into one integration file. Rejected because local reasoning and failure diagnosis become slower.

## Risks / Trade-offs

- [Behavior drift during code moves] -> Mitigation: preserve existing assertions, add compatibility-focused cases for warning codes/inferred flags, and run full existing test suites.
- [Over-fragmented modules reduce readability] -> Mitigation: keep only meaningful boundaries (`strict`, `inference`, `json`, shared types) and avoid premature micro-modules.
- [Temporary churn in imports/paths can introduce clippy warnings] -> Mitigation: complete move in one pass and run `just check` before marking done.

## Migration Plan

- No user migration required; SQL contract remains unchanged.
- Land internal module split behind the existing `timecontrol` facade.
- Validate with `just check` and `just test` to ensure no warnings and no behavior regressions.
- Roll back by reverting the refactor commit if parity issues are discovered.

## Open Questions

- Do any tests currently rely on private helper placement rather than behavior? If so, update those tests to assert behavior-oriented contracts instead of file-local implementation details.
