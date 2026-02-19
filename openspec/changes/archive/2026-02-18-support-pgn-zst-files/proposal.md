## Why

`read_pgn` currently reads plain PGN files, but many PGN datasets are stored as Zstandard-compressed files (`.pgn.zst`) to reduce storage and transfer costs. Adding optional zstd decompression support now enables users to query compressed archives directly in DuckDB without manual preprocessing.

## What Changes

- Extend `read_pgn` to accept an optional parameter `compression`.
- Support `compression = 'zstd'` so `read_pgn` can parse zstd-compressed PGN input streams.
- Preserve existing behavior when `compression` is omitted (plain PGN input remains the default).
- Return clear errors when an unsupported compression value is provided.
- Add SQLLogicTests and fixtures covering successful `.pgn.zst` reads and invalid compression argument handling.

## Capabilities

### New Capabilities
- None.

### Modified Capabilities
- `pgn-parsing`: Update `read_pgn` requirements to support optional compression mode selection, including `compression='zstd'` and validation/error behavior for unsupported compression values.

## Impact

- Affected code: `src/chess/reader.rs`, `src/chess/mod.rs`, and related argument parsing/registration logic for `read_pgn`.
- Affected tests: `test/sql/*.test` and compressed PGN fixtures under `test/pgn_files/`.
- Dependencies/system impact: likely introduction of zstd decompression support in the PGN read path while preserving streaming behavior and current non-compression defaults.
