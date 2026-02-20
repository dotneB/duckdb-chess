## Context

The extension defines multiple DuckDB scalar functions across `src/chess/moves.rs`, `src/chess/filter.rs`, and `src/chess/timecontrol.rs`. Today, each scalar duplicates similar `invoke()` boilerplate:

- flat-vector access for inputs/outputs
- per-row NULL checks
- `duckdb_string_t` decoding via `decode_duckdb_string`
- output writes (primitive slice assignment vs `CString` insertion)

This duplication increases the surface area of `unsafe` code and makes it easy for scalars to drift in subtle ways (NULL handling, error fallbacks, logging). Logging/diagnostics are also inconsistent (e.g., `eprintln!` in scalar fallbacks, ad-hoc warning formatting in the reader).

Constraints:

- Preserve all SQL-visible behavior and NULL/`[]`/empty-string fallbacks.
- Keep `unsafe` boundaries minimal and documented.
- Keep logging best-effort and consistent with DuckDB extension usage.

## Goals / Non-Goals

**Goals:**

- Extract shared scalar invocation helpers to eliminate duplicated input decoding, NULL checks, and output insertion patterns.
- Ensure scalar modules use a consistent, centralized warning/error reporting interface.
- Remove direct `eprintln!` usage from scalar fallback paths while keeping existing fallback outputs unchanged.

**Non-Goals:**

- Changing scalar signatures, output formats, or existing NULL semantics.
- Changing move/timecontrol parsing logic.
- Introducing new external dependencies solely for logging.

## Decisions

1. Create a dedicated internal module for scalar boilerplate (e.g., `src/chess/scalar.rs`).
   - Provide small, reusable helpers that cover the common shapes in this codebase:
     - unary `VARCHAR -> VARCHAR` (e.g., normalize, timecontrol normalize/category/json)
     - unary `VARCHAR -> BIGINT/UBIGINT/BOOLEAN` (e.g., ply count, hash)
     - binary `VARCHAR, VARCHAR -> BOOLEAN` (subset)
     - `VARCHAR, BIGINT -> VARCHAR` with per-row optional argument handling (moves_json_impl)
   - Helpers encapsulate:
     - flat vector lookup and typed slice access
     - per-row NULL checks (including multi-arg NULL policies)
     - `duckdb_string_t` decoding (reusing `duckdb_string::decode_duckdb_string`)
     - output writes (including `set_null` and primitive slice assignment)
   - Keep `unsafe` limited to the helper module and to calls that must cross DuckDB ABI boundaries.

2. Centralize warning/error reporting behind a single module (e.g., `src/chess/log.rs`).
   - Expose `warn(msg)` / `error(msg)` helpers.
   - Use a single formatting policy for messages.
   - Scalar fallbacks call the centralized logger at most once per invocation (or not at all), avoiding per-row stderr spam.

3. Preserve existing fallback behaviors by making them explicit at each call site.
   - The helper API should make the NULL/error output policy obvious (e.g., propagate NULL vs default value vs static string like `[]`).
   - Avoid "magical" defaults that could change behavior when refactoring.

## Risks / Trade-offs

- Refactor risk: small wrapper changes can alter NULL handling or validity masks. Mitigation: keep helpers narrow, cover each scalar shape explicitly, and rely on `just test` (unit + SQLLogicTest).
- Logging semantics: changing where/how warnings are emitted can affect user expectations. Mitigation: keep reader warnings visible and ensure scalar fallbacks remain non-fatal and output-stable.
- Complexity vs reuse: overly-generic helpers can become hard to use (lifetimes/HRTBs). Mitigation: prefer a small set of concrete helpers matching the project's scalar shapes.
