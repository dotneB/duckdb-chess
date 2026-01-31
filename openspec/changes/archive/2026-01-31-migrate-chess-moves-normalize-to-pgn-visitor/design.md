## Context

`chess_moves_normalize(movetext)` is currently implemented via custom character scanning in `src/chess/filter.rs`. It strips comments (`{...}`), recursive variations (`(...)`), and NAGs (`$n`, `!`, `?`, etc.) and emits a canonical mainline string with standardized spacing.

The repository already uses `pgn-reader` with a `Visitor` (see `src/chess/visitor.rs`) for streaming PGN parsing in `read_pgn`. That parser already handles PGN syntax details (variations, comments, outcomes) more robustly than bespoke string processing.

Constraints:
- Preserve the SQL API and the visible behavior of `chess_moves_normalize` as much as possible.
- Keep the implementation portable (Windows/macOS/Linux) and avoid new dependencies.
- Avoid panics; return best-effort output.

## Goals / Non-Goals

**Goals:**
- Implement `chess_moves_normalize` using `pgn-reader` as the parser of record.
- Strip comments, variations, and NAGs via parse events rather than manual scanning.
- Emit a stable canonical movetext format with consistent spacing and move numbers (e.g., `1. e4 e5 2. Nf3`).
- Preserve result markers (`1-0`, `0-1`, `1/2-1/2`, `*`) at the end when present.
- Keep behavior resilient: malformed input should not crash; ideally return a reasonable normalization.

**Non-Goals:**
- Legal-move validation or position tracking for normalization.
- Changing specs / user-facing semantics beyond tightening edge-case handling.
- Reworking `read_pgn` parsing; this change is scoped to `chess_moves_normalize`.

## Decisions

1) Use a dedicated `pgn-reader` Visitor for normalization

- Decision: Add a small Visitor (e.g., `NormalizeVisitor`) that collects only mainline SAN tokens and an optional game outcome.
- Rationale: `pgn-reader` already understands PGN token boundaries, nested comments, and variation structure. Using visitor events eliminates fragile brace/paren depth bookkeeping.
- Alternatives considered:
  - Keep custom parser and incrementally harden it: continues to duplicate PGN parsing logic and risks drift.
  - Reuse `GameVisitor` output from `src/chess/visitor.rs`: it currently preserves comments in movetext (by design for `read_pgn`) and would require invasive changes or post-processing.

2) Skip variations at the parser level

- Decision: Implement `begin_variation` to return `Skip(true)` so the parser ignores entire variation subtrees.
- Rationale: Matches existing behavior (normalize should keep mainline only) and avoids manual variation depth tracking.
- Alternative: Track variation depth and ignore events when depth > 0: more code and easier to get wrong.

3) Drop comments and NAGs by ignoring events

- Decision: Implement `comment` and NAG-related callbacks (if emitted by `pgn-reader`) as no-ops.
- Rationale: Avoid string-level stripping; rely on the parser to classify these tokens.
- Alternative: Allow comments through and strip later: adds post-processing complexity and can reintroduce nesting/whitespace issues.

4) Canonical serialization owned by us

- Decision: Reconstruct the output string from collected SAN tokens using a fixed formatting rule:
  - Insert move numbers for white plies: `1. <w> <b> 2. <w> ...`
  - Separate tokens with single spaces.
  - If an outcome is present, append it as the final token.
- Rationale: Produces stable output independent of input spacing and independent of whether the input included move numbers.
- Alternative: Preserve original move-number tokens from input: conflicts with visitor-based parsing and reduces normalization value.

5) No fallback; parse failure returns empty/NULL output

- Decision: Do not keep the legacy string-based normalizer as a fallback. If `pgn-reader` cannot parse the input, return empty output (recommended: empty string for non-NULL input) or `NULL` (if the SQL wrapper chooses to model parse failure as NULL).
- Rationale: Maintaining two independent normalization code paths is a long-term maintenance cost and undermines having a single source of truth.
- Alternatives considered:
  - Keep fallback for compatibility: reduces regression risk but requires maintaining (and testing) two implementations.

## Risks / Trade-offs

- [Parser strictness causes regressions on unusual input] -> Add regression tests for known problematic inputs; clearly specify parse-failure behavior in specs so consumers can handle it.
- [Differences in formatting vs legacy output] -> Ensure unit + SQLLogicTest assertions cover canonical formatting and common edge cases (result markers, check/mate suffixes, whitespace).
- [Outcome handling differs from legacy token preservation] -> Explicitly append outcome token at end when present; keep tests for `1-0` style markers.

## Migration Plan

1. Implement `NormalizeVisitor` using `pgn-reader::{Reader, Visitor, SanPlus, Outcome, Skip}` in a suitable module (likely `src/chess/visitor.rs` or a sibling module referenced by it).
2. Update `normalize_movetext` to use the visitor-based parse; on parse error, return empty/NULL output per the decided semantics.
3. Add/adjust tests:
   - Keep existing unit tests in `src/chess/filter.rs` passing.
   - Add tests for nested comments/variations and Lichess-style annotation comments.
   - Ensure result markers are preserved at the end.
4. Run `make dev` (fmt/clippy/build/tests) and `make test-release-rs` before merging.

## Open Questions

- Should parse-failure fallback remain permanently, or be removed after gaining confidence (with a broader test corpus)?
Should cleaned up
- Do we need to preserve the `NULL` behavior for `chess_moves_normalize(NULL)` explicitly in the scalar wrapper (vs empty string)?
