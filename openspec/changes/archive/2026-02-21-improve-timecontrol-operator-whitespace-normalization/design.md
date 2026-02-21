## Context

`TimeControl` normalization already accepts several whitespace-heavy shorthand forms, but operator-adjacent spacing is not handled consistently across all compound shapes. Inputs that are semantically equivalent to canonical controls (for example multi-stage values with spaces around `/`, `+`, or `:`) can fail parsing or normalize inconsistently.

The change must preserve existing invariants: canonical output remains compact, invalid inputs still return NULL (or row-level parse errors where applicable), and current behavior for already-canonical controls remains unchanged.

## Goals / Non-Goals

**Goals:**
- Accept optional whitespace around structural time-control operators (`+`, `/`, `:`) in otherwise valid controls.
- Emit canonical normalized output without operator-adjacent whitespace.
- Add regression coverage for both scalar normalization and SQL-visible behavior.

**Non-Goals:**
- Introduce new shorthand inference families beyond whitespace tolerance.
- Change category thresholds, warning codes, or parse modes.
- Alter output schema for `read_pgn` or timecontrol JSON payload shape.

## Decisions

- Pre-normalize operator-adjacent whitespace before strict parse.
  - Rationale: keeps parser logic simple and avoids duplicating grammar variants.
  - Alternative considered: broaden every parser branch to accept spaced tokens directly. Rejected because it increases grammar complexity and maintenance risk.
- Restrict whitespace normalization to structural operators only.
  - Rationale: avoids over-aggressive token stripping that could hide malformed numeric content.
  - Alternative considered: remove all internal whitespace globally. Rejected because it can accidentally merge unrelated tokens and reduce parse safety.
- Add focused tests for multi-operator spacing and preserve existing strict failures.
  - Rationale: proves the feature boundary and prevents regressions in permissiveness.
  - Alternative considered: rely only on existing broad shorthand tests. Rejected because these do not isolate operator-level normalization guarantees.

## Risks / Trade-offs

- [Risk] Whitespace collapsing could unintentionally accept previously invalid free-text forms. -> Mitigation: constrain cleanup to known operator boundaries and assert continued NULL outcomes for malformed suffix-bearing inputs.
- [Risk] Inference and canonical pathways diverge after preprocessing. -> Mitigation: route cleaned strings through existing parse pipeline and add parity tests against canonical equivalents.
- [Trade-off] Slightly more permissive parser behavior. -> Mitigation: keep behavior scoped to semantically equivalent operator spacing only.

## Migration Plan

- No migration required for API consumers.
- Rollout is test-driven: add failing unit/SQL tests, implement whitespace handling, verify with `just dev`.
- Rollback path: revert parser preprocessing and associated tests if unexpected acceptance behavior appears.

## Open Questions

- None.
