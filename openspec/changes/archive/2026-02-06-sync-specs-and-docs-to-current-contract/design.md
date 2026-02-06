## Context

OpenSpec and README currently contain multiple contract drifts relative to the tested implementation. The largest mismatches are in build/tooling version targets, module/entrypoint paths, and movetext result-marker expectations. These mismatches make contributor decisions harder and increase the chance of implementing against stale requirements.

## Goals / Non-Goals

**Goals:**
- Align OpenSpec requirements with current code and tests for build tooling, module layout, and movetext behavior.
- Align README version/tooling references with `Cargo.toml` and `Makefile`.
- Keep this as a documentation/spec synchronization change with no intentional runtime behavior changes.

**Non-Goals:**
- Introducing new runtime features or changing SQL function semantics.
- Large code refactors to match stale docs.
- Rewriting unrelated sections of specs/docs.

## Decisions

1) Prefer implementation-plus-tests as source of truth
- Decision: where specs/docs conflict with current tested behavior, update specs/docs to match implementation.
- Rationale: this change is explicitly a contract-sync pass, not a behavior change.
- Alternative considered: update code to match stale spec text. Rejected because it would introduce unnecessary behavioral churn.

2) Sync build-system contract to pinned repository constraints
- Decision: update build-system requirements to reflect DuckDB `1.4.4`, Rust-first workflows, and template compatibility (`extension-ci-tools`/optional Python targets).
- Rationale: current specs overstate "Rust-only/no-template-tooling" constraints and pin an outdated DuckDB target.
- Alternative considered: keep idealized "cargo-only everywhere" spec language. Rejected because it does not reflect this repository.

3) Sync code-structure contract to actual module layout
- Decision: update code-structure paths and entrypoint requirements to `src/chess/*` with a thin `src/lib.rs` root and extension registration in `src/chess/mod.rs`.
- Rationale: existing path expectations (`src/*.rs` entrypoint assumptions) are stale.

4) Sync movetext/result-marker contract to tested behavior
- Decision: update pgn/data-schema requirements so movetext is mainline SAN without appended terminal result marker, while result remains in the `Result` column.
- Rationale: current tests and visitor behavior already enforce this.

5) Keep docs-only unless tiny code tweak is clearly safer
- Decision: avoid runtime code changes; only allow tiny code adjustment if needed to remove dangerous ambiguity.
- Rationale: minimizes regression risk and keeps scope aligned with contract synchronization.

## Risks / Trade-offs

- [Potential conflict with in-flight feature changes] -> Mitigation: keep changes narrowly scoped to documented current behavior and avoid speculative future behavior.
- [Missed drift in a less-visible section] -> Mitigation: update directly impacted capabilities and README sections tied to known mismatches.
- [Contributors may expect behavior changes from updated text] -> Mitigation: state explicitly that this change is sync-only and non-functional.

## Migration Plan

1. Update OpenSpec deltas for `build-system`, `code-structure`, `pgn-parsing`, and `data-schema`.
2. Update README dependency/tooling/version references to match `Cargo.toml` and `Makefile`.
3. Run `openspec validate` for this change and ensure consistency.

## Open Questions

- Should follow-up change(s) archive stale historical wording that intentionally differed for future intent, or keep current-contract sync as the default style?
