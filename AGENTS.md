DuckDB loadable extension (Rust 2024) for chess PGN analysis.

Primary entrypoint: `read_pgn(path_pattern)` table function.
Helper scalar functions: `chess_*` (movetext normalization, hashing, ply count, FEN->EPD, JSON position tracing).


Hard constraints (do not drift)

- DuckDB target: 1.4.4 (`duckdb` + `libduckdb-sys` pinned to `=1.4.4`)
- Rust edition: 2024
- Rust MSRV: 1.89.0 (`Cargo.toml: rust-version = "1.89"`)
- Repo toolchain: Rust 1.93 (`rust-toolchain.toml`)
- Extension build tooling:
  - `duckdb-ext-macros = 0.1.0`
  - `cargo-duckdb-ext-tools` pinned to branch `fix/windows-build` (see `Makefile: install-tools`)


Build / lint / test commands (golden paths)

- Install tooling (run once): `make install-tools`
- Format + lint: `make check` (runs `cargo fmt --check` + `cargo clippy -- -D warnings`)
- Dev loop (lint + debug build + tests): `make dev`
- Build:
  - Debug: `make build-rs` (wraps `cargo duckdb-ext-build`)
  - Release: `make release-rs` (wraps `cargo duckdb-ext-build -- --release`)
- Tests:
  - Debug (unit + SQLLogicTest): `make test-rs`
  - Release (unit + SQLLogicTest): `make test-release-rs`


Running a single test

Rust tests

- Substring match: `cargo test <substring>`
- Fully qualified: `cargo test chess::visitor::tests::test_visitor_basic_parsing`
- With stdout/stderr: `cargo test <substring> -- --nocapture`

SQLLogicTest (DuckDB integration)

- One file (debug build):
  - `make build-rs && duckdb-slt.exe -e ./target/debug/chess.duckdb_extension -u -w "%CD%" "test/sql/read_pgn.test"`
- One file (release build):
  - `make release-rs && duckdb-slt.exe -e ./target/release/chess.duckdb_extension -u -w "%CD%" "test/sql/read_pgn.test"`

Notes

- `make test-rs` / `make test-release-rs` run all `test/sql/*.test`.
- On non-Windows environments the runner binary is `duckdb-slt` (no `.exe`).


Runtime / local loading

- Local builds are unsigned; start DuckDB with: `duckdb -unsigned`
- Load the extension from the repo root:
  - `LOAD './target/release/chess.duckdb_extension';`


Code map

- Extension entrypoint / registration: `src/chess/mod.rs`
  - registers `read_pgn` and `chess_*` scalars
  - uses SQL macros for NULL-safe wrappers (`chess_moves_json`, `chess_ply_count`)
- PGN I/O: `src/chess/reader.rs` (DuckDB VTab)
- PGN parsing visitor: `src/chess/visitor.rs` (pgn-reader Visitor)
- Movetext utilities: `src/chess/filter.rs`, `src/chess/moves.rs`
- Lichess-compatible record: `src/chess/types.rs`
- SQLLogicTests: `test/sql/*.test`
- PGN fixtures: `test/pgn_files/*.pgn`
- Specs: `openspec/specs/**` (design intent), change history: `openspec/changes/**`


extension-ci-tools submodule (do not remove)

- `extension-ci-tools/` is required by the official DuckDB community extension template wiring.
- Some template Make targets use Python/venv; local Rust-only workflows do not.


Behavioral invariants (do not break)

- `read_pgn` outputs the documented Lichess-style columns (see `README.md`).
- `parse_error` is NULL on success; set on parse/conversion failures and still return the row.
- Glob expansion triggers when `path_pattern` contains `*` or `?`.
- When reading multiple files (glob result), unreadable files are skipped with a warning.
- When reading a single explicit path, failures opening the file fail hard.
- Output is chunked: 2048 rows per output chunk to control memory usage.
- Malformed games: log warning and continue (do not fail the entire batch).
- `movetext` returned by `read_pgn` is mainline only; variations are skipped; `{...}` comments are preserved.
- `chess_moves_*` normalizers may strip comments/variations/NAGs to canonical main line.


Code style guidelines (Rust)

Formatting / lint

- Always run `make check` before committing.
- Use rustfmt defaults; do not hand-format around rustfmt.
- Clippy warnings are treated as errors (`-D warnings`).

Imports

- Group imports roughly as: std -> external crates -> crate/super.
- Prefer explicit imports over `use foo::*` (tests may use `use super::*`).
- Keep `use` lists sorted and formatted by rustfmt; avoid unused imports.

Types and NULL behavior

- DuckDB-facing functions typically return `Result<_, Box<dyn std::error::Error>>`.
- Represent nullable PGN tags as `Option<String>` and nullable numeric tags as `Option<u32>`.
- Use DuckDB C ABI structs for temporal columns:
  - `duckdb_date` for `DATE`
  - `duckdb_time_tz` for `TIMETZ`
- For public SQL functions that should be NULL-safe, prefer SQL macros (`coalesce(...)`) to avoid DuckDB's default NULL-in/NULL-out behavior.

Naming conventions

- SQL naming:
  - Table function: `read_pgn` (no prefix)
  - All scalar function names MUST be prefixed `chess_`
  - Single-move helpers: `chess_move_*`
  - Movetext/sequence helpers: `chess_moves_*`
- Rust naming:
  - DuckDB scalar wrappers are `*Scalar` implementing `duckdb::vscalar::VScalar`.
  - Prefer clear, domain-specific names (`GameRecord`, `GameVisitor`, `ParsedMovetext`).

Error handling

- Do not panic in extension code paths.
- For per-row parse/conversion issues:
  - append a human-readable message into `parse_error` (use `; ` separator)
  - return NULL for the specific field where appropriate
- For movetext utilities, parse failures should degrade safely (empty string, empty JSON, NULL output) as per existing behavior/tests.

Unsafe code

- Keep `unsafe` blocks minimal and localized.
- Add a `SAFETY:` comment when calling DuckDB C APIs or doing pointer-based string reads.
- Unit tests run without DuckDB initializing its C API; use `#[cfg(test)]` fallbacks where needed.

Performance / streaming

- PGN parsing is streaming; do not load whole files into memory.
- Do not wrap the underlying PGN reader in `BufReader` (pgn-reader already buffers internally).
- Prefer writing into DuckDB vectors; avoid per-row allocations where feasible.


Testing guidelines

- Add/extend SQLLogicTests in `test/sql/` when behavior is user-visible in SQL.
- Add Rust unit tests for parser/visitor logic, edge cases, and conversion rules.
- When adding a new SQL function:
  - register it in `src/chess/mod.rs`
  - add at least one `.test` case covering NULL behavior
  - update `README.md` if it changes the public API
