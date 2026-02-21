## Why

`src/chess/timecontrol.rs` is functionally strong but now combines strict parsing, heuristic inference, JSON rendering, and extensive tests in one large unit. That coupling increases maintenance cost and raises regression risk when evolving one concern (for example inference rules) because unrelated logic is co-located.

## What Changes

- Reorganize TimeControl internals into focused modules (for example `strict`, `inference`, `json`, and targeted test modules) behind a stable public facade.
- Keep all public SQL behavior and function names unchanged, including NULL/error behavior and JSON shape.
- Preserve warning taxonomy and inference semantics exactly across existing fixtures and SQL-visible outputs.
- Improve navigability by making module responsibilities explicit and reducing file size/concentration in `timecontrol` implementation.
- Add/adjust tests to guard behavior equivalence during the refactor.

## Capabilities

### New Capabilities
- None.

### Modified Capabilities
- `code-structure`: Extend module-boundary requirements so TimeControl parsing logic is decomposed into clear, focused modules while preserving existing API and behavior.

## Impact

- Affected code: `src/chess/timecontrol.rs` (split), plus new `src/chess/timecontrol/*` modules and related tests.
- API/runtime impact: no expected SQL API or output changes; behavior remains backward compatible.
- Validation impact: existing fixtures and SQLLogicTests remain authoritative; run `just check` and `just test` to confirm no regressions.
