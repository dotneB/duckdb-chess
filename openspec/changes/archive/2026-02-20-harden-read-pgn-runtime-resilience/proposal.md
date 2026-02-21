## Why

`read_pgn` is expected to stay non-panicking in runtime paths while still exposing recoverable ingestion issues. The current reader path can panic on poisoned mutex locks and may silently drop glob entry errors, which weakens resilience and observability during multi-file ingestion.

## What Changes

- Replace runtime `lock().unwrap()` usage in `reader` with poison-safe handling or structured error propagation that avoids panics.
- Preserve existing ingestion semantics: explicit single-file paths fail hard on open/read failures, while glob-based ingestion skips unreadable files and continues.
- Surface warnings for glob entry iteration errors instead of silently dropping failed entries.
- Add/extend tests to cover poison-safe behavior and glob-entry warning behavior where practical.

## Capabilities

### New Capabilities
- None.

### Modified Capabilities
- `pgn-parsing`: tighten runtime resilience and warning semantics for mutex poisoning and glob entry iteration failures without changing the `read_pgn` external contract.

## Impact

- Affected code: `src/chess/reader.rs` and related reader/runtime tests.
- Affected behavior: warning observability during glob ingestion and panic-avoidance in runtime error paths.
- Validation: existing SQLLogicTests and unit tests remain green, with targeted new coverage for poison/glob-error paths.
