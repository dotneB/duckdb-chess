## Context

`read_pgn` writes many string fields per row from `ChunkWriter` in `src/chess/reader.rs`, and the current implementation constructs `CString` values before insertion into DuckDB vectors. DuckDB's Rust bindings already expose `Inserter<&str>` that writes by pointer+length, so the current path is doing extra work in a hot loop. In the same area, chunk fullness is currently tied to a fixed `ROWS_PER_CHUNK = 2048`, while DuckDB exposes runtime vector capacity through `FlatVector::capacity()`.

The proposed change intentionally updates the chunk-size contract from a fixed number to runtime DuckDB vector capacity. This requires synchronized updates across implementation, tests, and OpenSpec requirements.

## Goals / Non-Goals

**Goals:**
- Remove per-value `CString` construction from `ChunkWriter` VARCHAR writes by using length-based `&str` insertion.
- Preserve interior-NUL sanitization and `parse_error` behavior exactly as today.
- Drive chunk fullness from runtime output vector capacity instead of a hardcoded constant.
- Keep existing `read_pgn` schema, malformed-game continuation, and single-path-vs-glob failure semantics unchanged.

**Non-Goals:**
- No changes to `read_pgn` output columns or SQL function naming.
- No changes to reader acquisition/threading architecture.
- No new dependencies or parser architecture changes.

## Decisions

1. **Use `Inserter<&str>` for all `ChunkWriter` VARCHAR inserts.**
   - Replace `CString::new(...)?` insertion calls with direct `vector.insert(row_idx, value_as_str)`.
   - Keep interior-NUL sanitization behavior (implemented via `sanitize_interior_nul` helpers) to preserve existing safety and diagnostics semantics.
   - Since these inserts are infallible, simplify row-write helpers to return `()` instead of `Result` and remove now-unneeded `?` propagation in the row-write path.
   - Alternative considered: call raw DuckDB C FFI directly. Rejected because the `duckdb` crate already wraps the same length-based API and keeps the code simpler.

2. **Derive per-call chunk limit from runtime vector capacity.**
   - Capture `max_rows` from `output.flat_vector(0).capacity()` when creating `ChunkWriter` and use it in `is_full()`.
   - Remove fixed-size assumptions from the writer-loop contract and tests.
   - Alternative considered: keep fixed 2048 with runtime assertion. Rejected because this change intentionally updates the contract to track DuckDB runtime configuration.

3. **Update specs and tests as part of the same change.**
   - Replace fixed-2048 chunk requirement assertions with vector-capacity-driven assertions.
   - Add/adjust unit tests to lock in (a) no behavior drift for sanitization/parse_error and (b) chunk-limit behavior keyed to runtime capacity.

## Risks / Trade-offs

- [DuckDB vector capacity assumption from first column] -> Mitigation: use the same chunk's vector capacity source consistently and retain conservative loop guards.
- [Spec/test churn from contract change] -> Mitigation: update all affected OpenSpec requirements and reader tests together in one change.
- [Potential subtle behavior drift in string sanitation path] -> Mitigation: preserve existing sanitize helpers and keep/extend targeted unit coverage around interior NUL handling and parse_error output.
