## 1. Tests and contract updates

- [x] 1.1 Update `src/chess/reader.rs` tests that currently assert fixed chunk size (`2048`) to assert runtime vector-capacity-driven chunking behavior.
- [x] 1.2 Add or adjust targeted unit coverage to confirm interior-NUL sanitization and `parse_error` behavior remain unchanged after string insertion refactor.

## 2. Length-based ChunkWriter string insertion

- [x] 2.1 Refactor `ChunkWriter` VARCHAR writes in `src/chess/reader.rs` to use DuckDB `Inserter<&str>` instead of `CString::new(...)`.
- [x] 2.2 Keep existing sanitization helpers in the write path and ensure all optional/string columns preserve current NULL and error semantics.
- [x] 2.3 Remove now-unneeded `CString` hot-path usage from `ChunkWriter` while preserving compile-time and clippy cleanliness.

## 3. Runtime chunk-capacity-driven row limits

- [x] 3.1 Update `ChunkWriter` construction to capture `max_rows` from `output.flat_vector(0).capacity()`.
- [x] 3.2 Update chunk-full checks to use runtime `max_rows` instead of `ROWS_PER_CHUNK`.
- [x] 3.3 Adjust related reader-flow code and tests so chunking contract follows DuckDB runtime vector capacity.

## 4. Verification

- [x] 4.1 Run `just test` and resolve any failures.
- [x] 4.2 Run `just check` and resolve formatting/lint failures.
- [x] 4.3 Run `just dev` to validate the full debug workflow.
- [x] 4.4 Run `just full` to ensure release also works.
