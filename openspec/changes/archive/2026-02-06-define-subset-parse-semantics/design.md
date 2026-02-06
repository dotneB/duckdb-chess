## Context

`chess_moves_subset(short_movetext, long_movetext)` is used for deduplication/subsumption checks and currently parses both inputs with `parse_movetext_mainline`. The parser returns SAN tokens plus a `parse_error` flag, but subset logic currently compares token vectors without considering parse failures. As a result, some invalid non-empty inputs are treated like empty move lists, which can yield unexpected `TRUE` results.

## Goals / Non-Goals

**Goals:**
- Define deterministic semantics for invalid non-empty movetext in `chess_moves_subset`.
- Eliminate false-positive subset matches caused by parse failures.
- Preserve existing behavior for empty-string and NULL inputs.
- Add unit and SQLLogicTest coverage for invalid short/long/both-invalid cases.

**Non-Goals:**
- Introducing a new SQL function or changing the existing signature.
- Performance optimization/fast-path work for clean canonical movetext.
- Changing behavior of other `chess_moves_*` functions.

## Decisions

1) Treat invalid non-empty input as not a subset candidate
- Decision: If either argument is non-empty and parsing fails, `chess_moves_subset` returns `FALSE`.
- Rationale: Invalid movetext should not silently match as an empty prefix.
- Alternative considered: Keep best-effort comparison of partial tokens. Rejected because it preserves false-positive risk and makes outcomes hard to reason about.

2) Preserve empty-string semantics
- Decision: Keep current empty-string behavior (`""` is subset of any parseable input; non-empty short vs empty long is `FALSE`).
- Rationale: Existing SQL and unit tests rely on this behavior, and it remains mathematically consistent with prefix semantics.
- Alternative considered: Treat empty string as invalid. Rejected as unnecessary breaking change.

3) Preserve NULL propagation semantics
- Decision: Keep DuckDB NULL-in/NULL-out behavior for `chess_moves_subset` unchanged.
- Rationale: Current behavior is already validated in SQL tests and aligns with DuckDB scalar defaults.
- Alternative considered: Coerce NULL to `FALSE`. Rejected because it changes SQL tri-valued logic expectations.

4) Use existing parser result metadata
- Decision: Use `ParsedMovetext.parse_error` and input emptiness checks to classify parse failure vs empty input.
- Rationale: Minimal change with no new dependencies and no duplicated parsing logic.
- Alternative considered: Add a separate validator/parser. Rejected as unnecessary complexity.

## Risks / Trade-offs

- [Behavior change for malformed input] -> Mitigation: document in spec delta and add explicit SQL tests for invalid inputs.
- [Ambiguity on partially recoverable malformed movetext] -> Mitigation: define single rule: any non-empty parse failure returns `FALSE`.

## Migration Plan

1. Update `check_moves_subset` logic to gate on parse errors for non-empty inputs.
2. Add/adjust unit tests in `src/chess/moves.rs`.
3. Add/adjust SQLLogicTests in `test/sql/chess_moves_subset.test`.
4. Run `make dev` and ensure behavior is stable across existing test suite.

## Open Questions

- Should this stricter parse-failure behavior be mirrored in any future sequence-comparison helpers for consistency?
