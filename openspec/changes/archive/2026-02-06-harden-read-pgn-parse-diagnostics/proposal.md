## Why

`read_pgn` exposes a `parse_error` column, but parser-stage failures are not reliably surfaced by deterministic SQL tests today. We need stronger diagnostics and coverage so users can distinguish conversion issues from true PGN parser failures and trust batch-ingestion observability.

## What Changes

- Improve parser-stage error reporting in `read_pgn` so `parse_error` carries useful context (stage/file/game index where available).
- Add deterministic malformed PGN fixtures that trigger parser-stage failures instead of only recoverable malformed inputs.
- Add SQLLogicTests that assert `parse_error IS NOT NULL` for true parser-stage failures.
- Preserve current batch resilience behavior: continue with subsequent games/files where existing behavior requires continuation.
- Keep existing conversion diagnostics and message aggregation behavior intact.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `pgn-parsing`: strengthen parser-stage error capture/observability requirements and contextual error content.
- `data-schema`: clarify `parse_error` behavior for parser-stage failures and mixed parser+conversion diagnostics.

## Impact

- Affected code: `src/chess/reader.rs` (read loop diagnostics), `src/chess/visitor.rs` (`finalize_game_with_error` message composition).
- Affected tests/fixtures: `test/pgn_files/*.pgn` and `test/sql/read_pgn_*.test` with deterministic parser-error coverage.
- User-visible effect: parser failures become easier to identify and audit through `parse_error` content while ingestion remains resilient.
