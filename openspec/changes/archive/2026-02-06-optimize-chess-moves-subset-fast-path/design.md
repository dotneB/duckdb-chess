## Context

`chess_moves_subset(short_movetext, long_movetext)` currently calls `parse_movetext_mainline` for both inputs on every row, then compares the parsed SAN vectors as prefix lists. This is correct for noisy PGN text, but it does unnecessary work for already-clean canonical mainline strings where a lightweight token-prefix check is sufficient. The goal is to reduce CPU cost without changing query-visible behavior.

## Goals / Non-Goals

**Goals:**
- Add a conservative fast path for clearly clean mainline movetext.
- Keep `chess_moves_subset` SQL semantics unchanged, including NULL propagation.
- Keep fallback to existing parser-based behavior whenever inputs are uncertain.
- Prove equivalence with Rust unit tests and SQLLogicTests.

**Non-Goals:**
- Introducing new SQL functions or changing signatures.
- Rewriting movetext parsing for other `chess_moves_*` functions.
- Defining new malformed-input semantics in this change.

## Decisions

1) Two-phase evaluation: fast path first, parser fallback second
- Decision: Implement a lightweight detector that only accepts obviously clean mainline input. If both arguments pass, run fast token-prefix comparison; otherwise, run existing parser path.
- Rationale: Guarantees correctness by defaulting to proven parser behavior for ambiguous text.
- Alternative considered: Always use fast tokenizer with best effort. Rejected because comments/variations/NAGs and malformed text can subtly change semantics.

2) Conservative clean-input detector
- Decision: The detector rejects any input containing comment/variation/NAG markers or other uncertain syntax and only accepts simple SAN+move-number+result token shapes.
- Rationale: False negatives are acceptable (parser fallback), false positives are not.
- Alternative considered: Aggressive regex acceptance to maximize fast-path hit rate. Rejected due to risk of semantic drift.

3) Fast token comparator semantics
- Decision: The fast comparator strips move numbers and trailing result markers (`1-0`, `0-1`, `1/2-1/2`, `*`) before prefix comparison.
- Rationale: Matches current subset behavior where result markers are not part of move sequence comparison.
- Alternative considered: Include result tokens in comparison. Rejected as behavior change.

4) Preserve NULL behavior
- Decision: Keep current DuckDB NULL-in/NULL-out behavior unchanged.
- Rationale: Existing SQL behavior and tests rely on it.

## Risks / Trade-offs

- [Detector misclassifies dirty input as clean] -> Mitigation: keep detector strict and add equivalence tests for boundary cases.
- [Optimization benefit lower than expected due to conservative detector] -> Mitigation: accept parser fallback; prioritize correctness over hit rate.
- [Behavior drift in future edits] -> Mitigation: add explicit equivalence tests between fast path and parser path fixtures.

## Migration Plan

1. Add unit tests that pin current semantics (including result marker handling and fallback parity).
2. Implement detector + fast comparator helpers in `src/chess/moves.rs`.
3. Wire fast-path attempt in `check_moves_subset` with parser fallback.
4. Update SQLLogicTests for clean/dirty equivalence and run `make dev`.

## Open Questions

- Should we later expose a separate strictly-canonical subset function for users who want guaranteed fast path only?
