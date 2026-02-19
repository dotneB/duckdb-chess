## 1. API Contract and Argument Validation

- [x] 1.1 Add failing SQLLogicTests for `read_pgn(..., compression := 'zstd')`, unsupported compression values, and explicit `compression := NULL` behavior.
- [x] 1.2 Update `read_pgn` bind logic to accept the optional `compression` argument in named form while preserving existing one-argument calls.
- [x] 1.3 Implement compression-mode validation so only omitted/NULL and `zstd` are accepted, with clear bind-time error messages for invalid values (including empty string).

## 2. Streaming Reader and Zstd Integration

- [x] 2.1 Add zstd decompression dependency and wire build configuration for cross-platform extension builds.
- [x] 2.2 Refactor reader state/input handling to support both plain `File` streams and zstd-decoded streams via a unified `Read` abstraction.
- [x] 2.3 Implement file-open/decode initialization paths that preserve existing single-path fail-hard and glob skip-with-warning behavior.
- [x] 2.4 Verify parser-stage runtime failures from compressed streams continue to surface through existing warning + `parse_error` flows.

## 3. Test Fixtures and Regression Coverage

- [x] 3.1 Add deterministic `.pgn.zst` fixture(s) under `test/pgn_files/` derived from existing PGN samples.
- [x] 3.2 Extend SQLLogicTests to assert row parity between plain PGN and zstd-compressed PGN for representative queries.
- [x] 3.3 Add coverage for glob reads over multiple `.pgn.zst` files, including unreadable-file skip behavior for multi-file inputs.

## 4. Documentation and Verification

- [x] 4.1 Update `README.md` API docs and examples to document `read_pgn(path_pattern, compression := NULL)` and `compression := 'zstd'` usage.
- [x] 4.2 Run `just dev` and fix any issues so lint, build, unit tests, and SQLLogicTests pass with compression support enabled.
