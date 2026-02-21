## Why

The `BindInfo` to `duckdb_bind_info` pointer cast currently works but is brittle across `duckdb-rs` upgrades because it relies on internal layout assumptions in multiple places. We need a single explicit FFI boundary so future dependency bumps are safer and easier to validate without risking regressions in `read_pgn` bind behavior.

## What Changes

- Isolate the `BindInfo` -> `duckdb_bind_info` cast behind a dedicated adapter with a narrow API.
- Document explicit `SAFETY` invariants for the adapter and keep the unsafe cast in one location.
- Add upgrade guidance/checklist for `duckdb-rs` version bumps so maintainers can re-validate the boundary.
- Validate that named-parameter handling behavior remains unchanged, and use upstream accessors instead of casting when they become available.

## Capabilities

### New Capabilities
- None.

### Modified Capabilities
- `code-structure`: Add/extend requirements to enforce a single isolated BindInfo FFI cast boundary, explicit safety invariants, and dependency-upgrade guidance while preserving existing bind-time behavior.

## Impact

- Affected code: `src/chess/reader.rs` bind path and a new/existing adapter module that owns the cast.
- Affected docs: maintenance guidance for `duckdb-rs` upgrades.
- Runtime/API impact: no SQL API changes expected; named-parameter handling remains behaviorally identical.
- Testing: unit and SQL tests covering bind/named-parameter behavior continue to pass.
