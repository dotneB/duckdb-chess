## Context

`read_pgn` and `chess_*` scalar helpers are used in lakehouse-style pipelines that process large PGN volumes, so avoidable allocations in hot paths create measurable memory pressure and reduce throughput. Current hotspots include always-owned DuckDB string decoding, intermediate `Vec<String>` materialization in movetext paths, repeated header scans/clones in `GameVisitor`, and formatting-heavy movetext construction.

The change must preserve all current SQL-visible behavior, including `parse_error` semantics, malformed-game continuation, and existing function contracts in `move-analysis` and `pgn-parsing` specs.

## Goals / Non-Goals

**Goals:**
- Reduce avoidable heap allocations in scalar and visitor hot paths.
- Keep current SQL signatures, null behavior, and output schema unchanged.
- Improve parser/scalar throughput for large datasets by eliminating duplicate parsing and transient allocations.
- Add tests that verify behavior parity while enabling future performance regression checks.

**Non-Goals:**
- Adding new SQL entrypoints or changing `read_pgn` output columns.
- Changing malformed-game handling, chunk sizes, or continuation behavior.
- Introducing heavyweight dependencies or engine/opening-book features.

## Decisions

1. Use borrow-first DuckDB string decoding.
   - Decision: update shared string decoding helper to return `Cow<str>` so valid UTF-8 is borrowed and lossy paths allocate only when required.
   - Alternatives considered:
     - Keep `String` return type everywhere: simplest but preserves unnecessary allocation cost.
     - Return `&str` only: fastest for valid UTF-8 but cannot represent lossy decoding safely.

2. Stream move analysis visitors instead of materializing SAN vectors for hot operations.
   - Decision: implement/extend visitor-based paths for `chess_moves_json`, `chess_ply_count`, and `chess_moves_normalize` so processing can stop early (`max_ply`) and avoid intermediate `Vec<String>` allocations.
   - Alternatives considered:
     - Keep current parse-then-reparse flow and only preallocate vectors: partial improvement but still duplicates parsing and retains overhead.

3. Replace header `Vec<(String, String)>` scans with structured capture in `GameVisitor`.
   - Decision: map known tags directly into a typed header struct during `tag()` and only keep optional extras when needed.
   - Alternatives considered:
     - Keep vector and optimize `get_header()` lookups with indexing map: improves lookups but still allocates/clones all header pairs.

4. Reduce transient movetext-string allocations.
   - Decision: replace `format!` heavy paths with `write!`/`push_str`, and apply light capacity planning for movetext buffers.
   - Alternatives considered:
     - Keep formatting macros for readability: easier to read but allocates temporary strings per token/comment.

5. Preserve semantics through parity tests before micro-optimizations.
   - Decision: add/extend unit tests and SQLLogic tests first for null handling, malformed input behavior, and output parity so refactors remain safe.
   - Alternatives considered:
     - Optimize first and backfill tests: faster upfront but higher regression risk.

6. Defer Criterion benchmark harness setup to a follow-up change.
   - Decision: do not add Criterion in this change because it is not a good fit for DuckDB extension workflows in this project.
   - Alternatives considered:
     - Add lightweight Criterion benchmarks now: provides standardized benchmark APIs but introduces friction and lower reliability for this extension context.
     - Add no measurement at all: lowest effort but removes objective validation of allocation/throughput impact.

## Risks / Trade-offs

- [Borrowed string lifetimes become more complex] -> Keep unsafe boundaries localized in shared decoder, document `SAFETY` invariants, and convert to owned values only at long-lived boundaries.
- [Visitor-based rewrites can introduce subtle output drift] -> Add parity fixtures for comments, variations, malformed SAN, and `max_ply` truncation.
- [Performance gains may vary by workload] -> Capture baseline vs. post-change benchmark snapshots in local dev notes/tests where feasible.
- [Refactor breadth spans multiple modules] -> Stage implementation in small commits/tasks (decoder, move-analysis visitors, visitor headers/movetext, reader helpers).

## Migration Plan

No user-facing migration is required. Roll out as an internal refactor, validate with `just dev`/`just full`, and keep rollback simple by reverting the change set if regressions are found.

## Open Questions

- For `read_pgn` vector insertion helpers, should interior-NUL sanitization + `parse_error` annotation be included in this change or split to a separate hardening change?
