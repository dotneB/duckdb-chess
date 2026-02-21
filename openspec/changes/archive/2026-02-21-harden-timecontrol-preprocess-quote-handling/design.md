## Context

`TimeControl` normalization already supports several quoted and apostrophe-style shorthands, but preprocessing remains fragile when inputs contain mixed quote wrappers, repeated quote noise, or unbalanced quoting around otherwise valid numeric/operator tokens. In those cases, the parser can receive corrupted intermediates, causing avoidable NULL outputs or inconsistent normalize/category/JSON behavior.

## Goals / Non-Goals

**Goals:**
- Make quote preprocessing deterministic across mixed single and double quote wrappers.
- Preserve semantic apostrophe tokens used by minute/second shorthand (`'`, `''`) while removing non-semantic quote noise.
- Ensure malformed quoted inputs degrade safely (`normalized = NULL`) without producing partially mangled numeric outputs.
- Keep normalize/category/JSON outputs aligned by reusing one hardened preprocessing path.

**Non-Goals:**
- No expansion of supported `TimeControl` grammars beyond quote-handling hardening.
- No change to speed-category thresholds or classification rules.
- No changes to `read_pgn` row-shaping behavior outside existing scalar function consumption.

## Decisions

- Introduce a quote-aware preprocessing pass that classifies quote characters by local context before downstream parsing.
  - Rule: apostrophes adjacent to numeric unit patterns remain tokens; quote characters acting as wrappers/delimiters are stripped.
  - Alternative considered: global quote removal. Rejected because it breaks valid apostrophe minute/second notation.
- Normalize wrapper quotes only when they form outer noise around a candidate expression; keep internal structural operators and digits untouched.
  - Alternative considered: strict balanced-quote requirement before any stripping. Rejected because many real PGNs include noisy but recoverable quote wrappers.
- On ambiguous quote residue that still pollutes structural tokenization, return safe parse failure instead of guessing.
  - Alternative considered: heuristic token stitching after failed parse. Rejected due to risk of inventing semantics for malformed values.
- Keep all scalar entry points (`chess_timecontrol_normalize`, `chess_timecontrol_category`, `chess_timecontrol_json`) on the same preprocessing/parsing pipeline.
  - Alternative considered: function-specific cleanup. Rejected to avoid cross-function divergence.

## Risks / Trade-offs

- [Context rules may misclassify edge apostrophes] -> Mitigation: add focused regressions for unit apostrophes and mixed wrappers.
- [More preprocessing branches increase maintenance cost] -> Mitigation: centralize quote handling in one helper and test via table-driven cases.
- [Being conservative may keep some noisy inputs as NULL] -> Mitigation: prefer deterministic safety; expand accepted forms only with explicit tests.

## Migration Plan

- No API or schema migration; scalar signatures remain unchanged.
- Implement quote-hardening in parser internals and extend unit/SQLLogicTest coverage.
- Validate with `just check` and `just test`; rollback remains a straightforward revert of this change set.

## Open Questions

- None; acceptance criteria are captured in the spec scenarios for recoverable vs ambiguous quoting.
