## Context

`read_pgn` maps `UTCDate`/`UTCTime` and fallback headers (`Date`, `EventDate`, `Time`) into typed DuckDB columns. Current candidate selection emphasizes precedence/completeness before conversion validity, so an invalid high-priority value can prevent use of a valid lower-priority value. Users then lose recoverable typed data even though fallback headers are present.

## Goals / Non-Goals

**Goals:**
- Allow date/time fallback to continue after invalid primary candidates.
- Preserve existing precedence/completeness intent among parseable candidates.
- Preserve `parse_error` reporting for failed candidate conversions.
- Add mixed-validity tests (invalid primary + valid fallback; partial-date and precedence cases).

**Non-Goals:**
- Changing column types or schema shape.
- Changing fallback header order (`UTCDate` -> `Date` -> `EventDate`, `UTCTime` -> `Time`).
- Silencing conversion diagnostics.

## Decisions

1) Parse-aware candidate selection
- Decision: evaluate candidates in fallback order, parse each candidate independently, and choose the best parseable candidate according to existing completeness/precedence policy.
- Rationale: this preserves current selection behavior for valid inputs while avoiding hard failure from one invalid candidate.
- Alternative considered: strict first-parseable candidate by order only. Rejected because it can regress existing completeness preference behavior.

2) Keep diagnostics even on successful fallback
- Decision: when a higher-priority candidate fails conversion but a lower-priority candidate succeeds, populate typed column from the successful fallback and still append conversion error details for failed candidates to `parse_error`.
- Rationale: users get maximal typed recovery plus transparency on bad source values.
- Alternative considered: clear diagnostics if fallback succeeds. Rejected because it hides data quality issues.

3) Null output only when no candidate parses
- Decision: emit `NULL` for `UTCDate`/`UTCTime` only when all non-empty candidates for that typed field fail or are unknown/empty.
- Rationale: aligns with graceful degradation and makes fallback meaningful.

## Risks / Trade-offs

- [More verbose parse_error strings for rows with fallback recovery] -> Mitigation: keep consistent field-tagged format and message separator.
- [Behavior differences on edge-case mixed candidate sets] -> Mitigation: expand unit and SQL tests for precedence, completeness, partials, and invalid-primary cases.
- [Potential regressions in existing date candidate scoring] -> Mitigation: keep/extend current scoring tests and run full `make dev`.

## Migration Plan

1. Add/adjust tests that currently fail on invalid-primary + valid-fallback combinations.
2. Update date/time selection logic in `src/chess/visitor.rs` to pick best parseable candidate rather than failing early on invalid primaries.
3. Ensure conversion errors for rejected invalid candidates are still appended to `parse_error`.
4. Run `make dev` and verify all existing and new tests pass.

## Open Questions

- Should parse errors include which fallback candidate ultimately won (for extra observability), or remain focused on failures only?
