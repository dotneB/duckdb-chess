## 1. Sync OpenSpec Requirements to Current Contract

- [x] 1.1 Update `openspec/specs/build-system/spec.md` to match current DuckDB/tooling/version constraints (`1.4.4`, Rust-first workflow, template compatibility)
- [x] 1.2 Update `openspec/specs/code-structure/spec.md` to match `src/chess/*` module layout and entrypoint location in `src/chess/mod.rs`
- [x] 1.3 Update `openspec/specs/pgn-parsing/spec.md` to reflect current result-marker handling (captured as result metadata, not appended to movetext)
- [x] 1.4 Update `openspec/specs/data-schema/spec.md` movetext expectations to match implementation/tests (mainline movetext without terminal result marker)

## 2. Sync README to Repository Reality

- [x] 2.1 Update README dependency/tooling version references to match `Cargo.toml` and `Makefile` pins
- [x] 2.2 Update README build/tooling notes to clarify Rust-first workflows and template compatibility constraints
- [x] 2.3 Ensure README movetext/result wording is consistent with current tested behavior

## 3. Validate and Guard Against Drift

- [x] 3.1 Run `openspec validate sync-specs-and-docs-to-current-contract`
- [x] 3.2 Run `make check` to ensure docs/spec updates do not introduce formatting/lint regressions
