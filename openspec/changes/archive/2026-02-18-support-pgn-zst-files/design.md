## Context

`read_pgn` currently opens each matched path as a plain `File` and feeds it directly into `pgn_reader::Reader`, so compressed `.pgn.zst` datasets must be decompressed outside DuckDB before querying. For large archives, that adds storage overhead and extra pipeline steps.

The change must preserve existing parsing contracts: same 18 output columns, chunked output (2048 rows), malformed-game continuation, and single-path-vs-glob error behavior.

## Goals / Non-Goals

**Goals:**
- Add an optional `compression` parameter to `read_pgn` with supported value `zstd`.
- Keep current default behavior unchanged when `compression` is omitted.
- Preserve streaming behavior and existing multi-file parallel work assignment.
- Return clear validation errors for unsupported compression modes.

**Non-Goals:**
- Auto-detect compression by extension or file magic.
- Add support for additional compression formats (gzip, bzip2, xz) in this change.
- Change output schema, parse_error semantics, or chunk size.

## Decisions

1. Add optional compression argument at table-function bind time.
   - Decision: extend `read_pgn` binding to accept an optional `compression` argument (usable as named argument in SQL and compatible with positional form), normalized case-insensitively.
   - Accepted values: omitted/NULL (plain input) and `zstd`.
   - Unsupported non-NULL values fail fast with a bind-time error that lists allowed values.
   - Alternatives considered:
     - Infer compression from filename suffix: convenient but ambiguous and harder to validate.
     - Auto-detect from bytes: more robust but adds probe complexity and extra I/O.

2. Generalize reader input from `File` to a stream abstraction.
   - Decision: refactor `PgnReaderState` to hold `pgn_reader::Reader<Box<dyn std::io::Read + Send>>` so each file can be backed by either plain `File` or a zstd decoder stream.
   - Rationale: this keeps one reader state type while supporting mixed input wrappers without duplicating parsing logic.
   - Alternatives considered:
     - Generic `PgnReaderState<R>`: cleaner static typing but awkward for storage in shared state queues.
     - Enum wrapper implementing `Read`: avoids dynamic dispatch but adds custom plumbing for little practical gain.

3. Keep file-open/error behavior aligned with existing invariants.
   - Decision: retain current policy boundaries:
     - Single explicit path: open/decode initialization failures fail the query.
     - Multi-file glob result: unreadable or undecodable files are warned and skipped.
   - Runtime decompression/read failures encountered while parsing a game continue to be surfaced through the existing parser-stage `parse_error` path and warning logs.
   - Alternatives considered:
     - Fail entire glob query on first zstd error: stricter but breaks current continuation expectations.

4. Introduce explicit coverage for compression mode behavior.
   - Decision: add SQLLogicTests for:
     - Successful read from `.pgn.zst` with `compression='zstd'`.
     - Backward compatibility when compression is omitted on plain PGN.
     - Unsupported compression value error.
   - Add compressed test fixture(s) under `test/pgn_files/` generated from existing sample PGN content to keep expected rows deterministic.

5. Update user-facing API docs with the optional parameter.
   - Decision: update README function signature and examples to document `read_pgn(path_pattern, compression := NULL)` with `compression := 'zstd'` usage.
   - Alternatives considered:
     - Delay documentation update: avoids immediate docs churn but leaves public API ambiguous.

## Risks / Trade-offs

- [Trait-object reader introduces dynamic dispatch] -> Accept minimal overhead because PGN parsing dominates cost; keep hot loops unchanged.
- [Zstd dependency increases build footprint] -> Use a single well-supported crate and avoid optional codec matrix in this change.
- [Bind-time argument handling may differ for named vs positional calls in DuckDB extension APIs] -> Cover both call styles in SQLLogicTests and keep parser tolerant to either binding shape.
- [Corrupt compressed inputs may fail mid-stream] -> Preserve existing per-game parse_error logging and continuation behavior, and add fixture-based failure coverage where feasible.

## Migration Plan

No data migration is required. Existing one-argument `read_pgn(path_pattern)` calls remain valid and unchanged. Rollout is additive: ship with tests/docs updates and validate via `just dev` (and `just full` before release).

## Open Questions

- Should the implementation treat empty-string `compression` values as invalid input (recommended) or as equivalent to omitted/NULL?
Yes, should be treated as an invalid input
