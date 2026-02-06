## Context

`read_pgn` currently captures parser errors by calling `finalize_game_with_error` when `pgn-reader::Reader::read_game` returns `Err`. This preserves rows, but parser-stage diagnostics are hard to validate in SQL because existing fixtures mostly trigger recoverable parsing behavior. The result is weak confidence that parser-stage failures remain observable through `parse_error` and that downstream users can distinguish parser failures from conversion failures.

## Goals / Non-Goals

**Goals:**
- Make parser-stage failures deterministically observable in SQL tests.
- Enrich parser-stage `parse_error` messages with actionable context (stage/file/game index where available).
- Preserve ingestion resilience: continue processing subsequent games/files according to existing behavior.
- Keep conversion diagnostics aggregation behavior unchanged.

**Non-Goals:**
- Changing `read_pgn` schema or column ordering.
- Changing conversion rules for typed fields (`UTCDate`, `UTCTime`, `WhiteElo`, `BlackElo`).
- Treating all malformed games as hard failures.

## Decisions

1) Add deterministic parser-error fixtures and SQL assertions
- Decision: introduce at least one PGN fixture that reliably makes `read_game` return `Err` and add SQLLogicTests asserting `parse_error IS NOT NULL`.
- Rationale: this closes observability gaps and prevents regressions in parser-stage error capture.
- Alternative considered: rely only on unit tests with in-memory strings. Rejected because SQL-level behavior is the user-facing contract.

2) Include parser-stage context in error messages
- Decision: standardize parser-stage `parse_error` text to include stage identifier and source location context (file path and game index when available).
- Rationale: users need enough context to triage bad input in multi-file ingestion.
- Alternative considered: keep current generic message. Rejected because it is harder to diagnose at scale.

3) Preserve existing continuation behavior
- Decision: continue emitting partial/error rows and continue scanning subsequent games/files as currently specified.
- Rationale: avoids breaking batch ingestion workflows and aligns with graceful degradation requirements.

4) Keep message aggregation format stable
- Decision: continue appending additional conversion diagnostics to parser-stage messages using existing separator behavior.
- Rationale: preserves existing downstream parsing/inspection patterns for `parse_error`.

## Risks / Trade-offs

- [Fixture fails to trigger parser `Err` on future parser versions] -> Mitigation: keep fixture minimal and add fallback unit test that verifies the fixture actually triggers parser-stage error path.
- [Longer parse_error messages] -> Mitigation: keep a consistent, compact format while preserving stage/file/game context.
- [Behavioral drift in continue-on-error flow] -> Mitigation: add SQL test that confirms rows after an errored game/file are still returned when current behavior requires continuation.

## Migration Plan

1. Add deterministic malformed fixture(s) under `test/pgn_files/` and SQL test coverage under `test/sql/`.
2. Update parser error message construction in `src/chess/reader.rs` (and related visitor finalization wiring) to include stage/file/game context.
3. Verify parser-stage and conversion diagnostics are both preserved in `parse_error` when both occur.
4. Run `make dev` and fix any regressions.

## Open Questions

- If game index cannot be determined for a failure, should the message explicitly emit `game_index=unknown` or omit that field?
