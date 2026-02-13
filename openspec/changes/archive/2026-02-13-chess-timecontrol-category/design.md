## Context

The project already normalizes and parses PGN `TimeControl` values through `chess_timecontrol_normalize` and `chess_timecontrol_json`. Users currently must reimplement speed bucketing in SQL for each dataset, which creates drift and inconsistent interpretations of increment-heavy controls.

This change adds a single SQL scalar that derives a Lichess-compatible category from normalized parsing output while preserving existing NULL-safe behavior for ambiguous inputs.

## Goals / Non-Goals

**Goals:**
- Add `chess_timecontrol_category(VARCHAR) -> VARCHAR` to classify controls as `ultra-bullet`, `bullet`, `blitz`, `rapid`, or `classical`.
- Base classification on estimated duration `base_seconds + 40 * increment_seconds`.
- Reuse existing parsing/normalization logic so inferred forms (for example `3+2`) are categorized the same as canonical forms (`180+2`).
- Return NULL when no reliable category can be derived.
- Cover thresholds and inference behavior with Rust and SQLLogic tests.

**Non-Goals:**
- Changing `read_pgn` output schema.
- Introducing Chess.com/FIDE-specific category modes.
- Categorizing unlimited (`-`), unknown (`?`), or sandclock (`*N`) controls into Lichess speed buckets.

## Decisions

1. **Add a dedicated scalar wrapper in `src/chess/timecontrol.rs`**
   - Implement `ChessTimecontrolCategoryScalar` and register it in `src/chess/mod.rs` as `chess_timecontrol_category`.
   - Rationale: matches existing scalar architecture and keeps all TimeControl logic co-located.
   - Alternative considered: SQL macro composition over existing functions. Rejected because category logic depends on parsed period fields not exposed as columns.

2. **Derive category from parsed first period in normal mode**
   - Parse input with `parse_timecontrol`.
   - If `mode != Normal` or no period exists, return NULL.
   - Use first period `base_seconds` and `increment_seconds.unwrap_or(0)` for estimate.
   - Honor existing minute-shorthand inference (`N+I` with small `N`) before classification; explicit second notation is required for unambiguous sub-minute inputs.
   - Rationale: parser already normalizes strict and inferred forms; first period maps to the starting clock for practical online controls.
   - Alternative considered: require exactly one stage with no move-count metadata. Rejected to avoid unnecessary NULLs on valid multi-stage normalized values.

3. **Use fixed Lichess thresholds and canonical labels**
   - `<= 29` -> `ultra-bullet`
   - `<= 179` -> `bullet`
   - `<= 479` -> `blitz`
   - `<= 1499` -> `rapid`
   - `>= 1500` -> `classical`
   - Rationale: aligns with Lichess FAQ definitions and user expectation for direct comparability.

4. **Fail safely with NULL, never panic**
   - Any parse failure, missing data, or non-normal mode yields NULL.
   - Rationale: consistent with existing normalization utilities and extension error-handling conventions.

## Risks / Trade-offs

- **[Ambiguous non-standard controls]** Some unusual controls may return NULL rather than a best-effort bucket -> Mitigation: keep behavior explicit in spec/tests; users can inspect `chess_timecontrol_json` for details.
- **[Assumed 40-move estimate]** Classification can differ from actual game duration -> Mitigation: document formula in README/function docs and tests.
- **[Ambiguous small-base notation]** Inputs like `29+0` are interpreted as minutes by current normalization and therefore do not represent 29 seconds -> Mitigation: document this explicitly and add tests for both shorthand and explicit-seconds notation.
- **[Label compatibility expectations]** Consumers may expect alternate casing -> Mitigation: use stable documented labels and keep future aliasing additive if needed.

## Migration Plan

- Additive change only: register new scalar, add tests, and document function in README.
- No data migration or backward-incompatible behavior changes.

## Open Questions

- None.
