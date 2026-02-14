## Why

Current parsing and scalar hot paths allocate more than necessary, especially when decoding DuckDB strings, building movetext output, and re-parsing SAN tokens for JSON and counting helpers. These extra allocations increase memory pressure and reduce throughput on large PGN workloads, so we should optimize now before adding more pipeline-facing features.

## What Changes

- Introduce allocation-focused refactors for scalar and visitor hot paths while preserving current SQL-visible behavior.
- Replace always-owned DuckDB string decoding with borrow-first decoding (`Cow<str>`) so valid UTF-8 avoids heap allocation.
- Rework move-processing internals to stream where possible (JSON emission, ply counting, normalization) instead of materializing intermediate move vectors.
- Refactor `GameVisitor` header capture and movetext string building to reduce transient allocations and cloning.
- Add targeted tests and benchmarks/assertions to lock in behavior and validate memory/throughput improvements.

## Capabilities

### New Capabilities
- `allocation-efficiency`: Non-functional requirements for minimizing avoidable allocations in PGN parsing and move-analysis internals while preserving existing outputs and error semantics.

### Modified Capabilities
- `move-analysis`: Tighten implementation requirements for `chess_moves_json`, `chess_ply_count`, and `chess_moves_normalize` to prefer streaming/internal low-allocation execution without changing function contracts.
- `pgn-parsing`: Tighten visitor/internal parsing requirements to reduce header/movetext transient allocations while keeping malformed-game handling and chunked output behavior unchanged.

## Impact

- Affected code: `src/chess/duckdb_string.rs`, `src/chess/filter.rs`, `src/chess/moves.rs`, `src/chess/visitor.rs`, and parts of `src/chess/reader.rs` where string handling/output insertion paths interact with parsed records.
- Public API: No new SQL function required for this change; existing function signatures and schemas remain stable.
- Testing/docs: Add or update Rust unit tests and SQLLogic tests for behavior parity; update OpenSpec capability deltas and implementation tasks.
