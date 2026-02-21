## Context

`timecontrol` normalization currently infers minute-based shorthand (`N`, `N+I`, punctuation variants, and compact textual forms) by converting minute values to seconds with unchecked `u32` arithmetic. In optimized builds, overflow can wrap silently and feed invalid values into normalized strings, category derivation, and JSON output. This violates deterministic parse behavior and can misclassify game speed for extreme inputs.

## Goals / Non-Goals

**Goals:**
- Make all inference-path minute/second arithmetic overflow-safe.
- Ensure overflow outcomes are deterministic: no wrapped values, `normalized = NULL`, and explicit warning tags.
- Keep non-overflow behavior stable for normalization, category, and JSON outputs.
- Add boundary and overflow regression coverage for inference code paths.

**Non-Goals:**
- No change to Lichess category thresholds or mapping rules.
- No expansion of supported shorthand grammars beyond existing inference rules.
- No changes to unrelated PGN parsing or `read_pgn` table-function behavior.

## Decisions

- Introduce dedicated checked conversion helpers for inference arithmetic (`minutes -> seconds`, staged additions, and any base+increment composition) using `checked_mul`/`checked_add`.
  - Alternative considered: widening to `u64` and truncating/clamping to `u32`. Rejected because it can hide overflow conditions and produce lossy outputs that appear valid.
- Treat overflow in inference as a safe parse degradation, not a hard error: inferred parse metadata is preserved, `normalized` becomes `NULL`, and warnings include a stable overflow tag.
  - Alternative considered: returning a hard SQL error on overflow. Rejected because current parsing behavior is tolerant and favors row-level recoverability.
- Keep canonical already-seconds inputs on existing strict parse path so large valid values are not penalized by inference overflow safeguards.
  - Alternative considered: applying overflow warning logic to all parse modes. Rejected to avoid behavior drift for canonical values that already parse deterministically.
- Ensure `chess_timecontrol_category` and `chess_timecontrol_json` consume the same overflow-safe parse result so category/JSON behavior remains consistent with normalization.
  - Alternative considered: independent overflow handling per function. Rejected because it risks divergence across scalar outputs.

## Risks / Trade-offs

- [Warning-tag naming drift could break downstream expectations] -> Mitigation: define one stable overflow warning code and lock it with tests.
- [Boundary tests may become brittle if tied to exact message text] -> Mitigation: assert on warning codes and normalized/category semantics, not free-form prose.
- [More checked branches in hot parsing paths] -> Mitigation: keep helpers inline and scoped to inference-only arithmetic.

## Migration Plan

- No API migration required; scalar function names and return types remain unchanged.
- Roll out as an internal parser hardening change with new unit and SQL regression tests.
- Validate with `just check` and `just test`; rollback is a simple revert if unexpected regressions appear.

## Open Questions

- None currently; overflow behavior and warning semantics are fully specified for implementation.
