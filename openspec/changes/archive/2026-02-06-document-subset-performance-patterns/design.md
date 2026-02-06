## Context

Users currently have the core subset function (`chess_moves_subset`) but limited guidance on selecting the right query strategy for large datasets. In practice, workloads split into two common modes: raw/noisy movetext (requires parser-backed semantics) and already normalized canonical movetext (can use faster string-prefix filters). Documentation should make this decision explicit and prevent semantic mistakes such as substituting `contains` for prefix checks.

## Goals / Non-Goals

**Goals:**
- Add practical README guidance for choosing subset query patterns by data quality.
- Provide copy-paste SQL examples for both raw/noisy and normalized/materialized pipelines.
- Document semantic caveats around result markers and normalization assumptions.
- Explicitly warn that `contains` is not equivalent to subset-prefix semantics.

**Non-Goals:**
- Changing `chess_moves_subset` implementation or SQL signature.
- Introducing new scalar functions for performance guidance.
- Benchmarking across hardware/OS combinations in this change.

## Decisions

1) Decision table in README for pattern selection
- Decision: add a compact "when to use what" section mapping input quality to recommended SQL (`chess_moves_subset` vs `starts_with` on canonical columns).
- Rationale: users need quick guidance without scanning long prose.
- Alternative considered: only narrative text. Rejected as less scannable during query authoring.

2) Show canonicalization-first workflow for large datasets
- Decision: include example that materializes `chess_moves_normalize(movetext)` once and uses `starts_with` for repeated prefix filtering.
- Rationale: this reflects practical analytics usage and reduces repeated parsing in query hot paths.
- Alternative considered: keep all examples on `chess_moves_subset`. Rejected because it misses a common optimization pattern.

3) Preserve semantic correctness warnings
- Decision: include explicit warning and example proving `contains` is not subset semantics.
- Rationale: avoids subtle false positives from substring matching.
- Alternative considered: omit negative example. Rejected because mistakes are frequent and costly in dedup/opening analysis.

4) Include result-marker caveat in docs
- Decision: document that subset semantics are prefix-based over move sequence and illustrate result-marker caveats in examples.
- Rationale: reduces confusion when users compare raw text directly.

## Risks / Trade-offs

- [Documentation drifts from implementation over time] -> Mitigation: tie guidance to move-analysis spec scenarios and keep examples simple and testable.
- [Users over-apply starts_with to non-canonical data] -> Mitigation: call out normalization prerequisite and provide raw-data fallback using `chess_moves_subset`.

## Migration Plan

1. Add move-analysis spec delta for subset performance/semantics guidance.
2. Update README with decision table, canonicalization workflow, and caveat examples.
3. Run `make check` to ensure docs/examples remain consistent with linting workflow.

## Open Questions

- Should we add a tiny SQLLogicTest snippet in a docs-oriented test file to validate README examples over time, or keep this change docs/spec-only?
