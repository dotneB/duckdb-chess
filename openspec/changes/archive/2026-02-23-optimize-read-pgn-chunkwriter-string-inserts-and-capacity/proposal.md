## Why

`read_pgn` writes multiple VARCHAR columns per row in a hot path, and `ChunkWriter` currently routes each string through `CString::new(...)`, adding avoidable scans and allocations before handing data to DuckDB. The same code path also assumes a fixed 2048-row chunk limit, which couples behavior to a hardcoded value instead of DuckDB's runtime vector capacity.

## What Changes

- Replace `CString`-based string insertion in `ChunkWriter` with length-based `&str` insertion via DuckDB `Inserter`.
- Preserve existing interior-NUL sanitization and `parse_error` accumulation semantics for all emitted string columns.
- Change chunk row limiting from a fixed 2048 constant to runtime vector capacity (`output.flat_vector(0).capacity()`).
- Update tests and specs so chunking behavior is defined by DuckDB vector capacity rather than a hardcoded row count.

## Capabilities

### New Capabilities
- None.

### Modified Capabilities
- `allocation-efficiency`: Add requirements that `read_pgn` row emission avoids per-value `CString` allocation when writing VARCHAR columns, while preserving current sanitization and SQL-visible behavior.
- `pgn-parsing`: Change chunked output requirements from fixed 2048-row chunks to runtime DuckDB vector-capacity-driven chunk sizing.
- `code-structure`: Update reader chunking structure requirements so row-limit control comes from runtime output vector capacity rather than a constant value of 2048.

## Impact

- Affected code: `src/chess/reader.rs` (`ChunkWriter` string insertion path and chunk-full logic), plus related tests in `src/chess/reader.rs` and SQL/unit tests that assert chunk sizing behavior.
- User-facing SQL contract: `read_pgn` schema and `parse_error` semantics remain unchanged; chunk sizing contract is updated to DuckDB vector capacity.
- Dependencies/APIs: no new dependencies; continue using existing `duckdb` crate vector inserter APIs.
