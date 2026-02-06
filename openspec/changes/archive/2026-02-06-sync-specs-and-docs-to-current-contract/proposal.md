## Why

Several OpenSpec documents and README sections have drifted from the current tested implementation, which creates confusion for contributors and increases the chance of incorrect future changes. We need to realign specs/docs to the actual runtime contract before additional feature work lands.

## What Changes

- Update OpenSpec build-system requirements to match the repository's current DuckDB/tooling/version constraints.
- Update OpenSpec code-structure requirements to reflect the real module layout under `src/chess/` and actual extension entrypoint wiring.
- Update OpenSpec parsing/schema requirements to match current movetext/result-marker behavior and existing tests.
- Update README dependency/tooling/version references to match `Cargo.toml`, `Makefile`, and current workflow.
- Keep this change docs/spec-first; avoid code changes unless a tiny code adjustment is clearly safer than changing docs.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `build-system`: align version/tooling requirements with current pinned DuckDB and build commands.
- `code-structure`: align module/file-path and entrypoint requirements with `src/chess/*` architecture.
- `pgn-parsing`: align parse-behavior requirements with current tested movetext/result handling.
- `data-schema`: align schema-level movetext/result-marker expectations with current implementation and tests.

## Impact

- Affected specs: `openspec/specs/build-system/spec.md`, `openspec/specs/code-structure/spec.md`, `openspec/specs/pgn-parsing/spec.md`, `openspec/specs/data-schema/spec.md`.
- Affected docs: `README.md` dependency/tooling/version notes and related usage clarifications.
- User-visible behavior: none intended; this change synchronizes docs/specs with existing tested behavior.
