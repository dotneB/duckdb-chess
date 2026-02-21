## Context

`read_pgn` bind-time named-parameter parsing currently relies on `bind_info_ptr()` in `src/chess/reader.rs`, which casts `&BindInfo` to `duckdb_bind_info` using `unsafe`. The cast is valid for the current `duckdb-rs` version but ties correctness to wrapper layout assumptions that are not guaranteed across future upgrades. This is a maintainability and safety boundary issue, not a product behavior issue.

## Goals / Non-Goals

**Goals:**
- Isolate the `BindInfo` -> `duckdb_bind_info` cast in exactly one adapter location with explicit `SAFETY` invariants.
- Keep the adapter API narrow so `reader.rs` does not depend on FFI pointer details.
- Preserve existing named-parameter behavior (`compression` omitted/NULL/value handling) with no SQL-visible regressions.
- Add upgrade guidance for `duckdb-rs` bumps, including a checklist to re-validate the boundary and switch to upstream accessors when available.

**Non-Goals:**
- Changing `read_pgn` SQL signature, return schema, or compression semantics.
- Refactoring unrelated reader parsing/chunking logic.
- Introducing new optional dependencies.

## Decisions

### Decision: Introduce a dedicated BindInfo FFI adapter module
Create a focused module (for example `src/chess/duckdb/bind_info_ffi.rs`) that owns all `BindInfo`/`duckdb_bind_info` interop and exposes a small safe surface for the reader path.

- **Rationale:** Localizes unsafe assumptions, prevents copy/paste casts, and makes audits straightforward.
- **Alternative considered:** Keep cast in `reader.rs` with stronger comments. Rejected because it does not prevent future spread and weakens reviewability.

### Decision: Preserve behavior via adapter-level parity tests
Keep existing semantics for named parameter access and compression mode resolution by routing existing helper logic through the adapter without changing decision logic.

- **Rationale:** Acceptance criteria require zero behavior regressions in named-parameter handling.
- **Alternative considered:** Broader bind-path rewrite. Rejected as unnecessary risk for this focused change.

### Decision: Prefer upstream accessors when available
During implementation, explicitly check whether current `duckdb-rs` exposes stable BindInfo accessors that remove the need for layout casting. If available, use them in the adapter and avoid raw cast.

- **Rationale:** Reduces long-term unsafety and upgrade risk.
- **Alternative considered:** Keep cast permanently. Rejected because upstream APIs may now or later provide safer options.

### Decision: Add a dependency-upgrade checklist
Document a short checklist tied to `duckdb-rs` upgrades (where to inspect wrapper definitions, what tests to run, and how to validate named-parameter behavior).

- **Rationale:** Makes future upgrades deliberate and repeatable.
- **Alternative considered:** Rely on tribal knowledge and code comments alone. Rejected for maintainability.

## Risks / Trade-offs

- **[Risk] Adapter abstraction drifts from actual use sites** -> **Mitigation:** Keep API minimal and only expose functions required by `reader.rs` bind path.
- **[Risk] Upstream crate changes invalidate cast assumptions** -> **Mitigation:** Centralize cast, document invariants, and enforce upgrade checklist on dependency bumps.
- **[Risk] Subtle named-parameter behavior regression** -> **Mitigation:** Add/retain targeted tests for omitted/NULL/valid/invalid `compression` named parameter handling.

## Migration Plan

1. Add adapter module and move the cast there with explicit `SAFETY` contract text.
2. Rewire `reader.rs` named-parameter helper(s) to use adapter APIs only.
3. Add/refresh tests that cover named parameter behavior parity.
4. Add upgrade guidance/checklist in repo docs or module-level maintenance notes.
5. Run `just full` to validate lint/build/tests in debug and release paths.

Rollback strategy: revert the adapter wiring commit; no persisted data or SQL contract migration is involved.

## Open Questions

- Does the current pinned `duckdb-rs` already expose a stable `BindInfo` raw-pointer accessor we can adopt immediately?
- Should upgrade guidance live in module docs (`src/chess/...`) or in a dedicated maintainer doc under `docs/`?
