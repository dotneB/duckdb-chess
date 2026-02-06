## Context

Two scalar modules currently implement their own unsafe DuckDB string decoding helper (`read_duckdb_string`), which duplicates low-level pointer handling and spreads unsafe assumptions across the codebase. This refactor is internal-only: we want a single audited unsafe boundary with clear `SAFETY` documentation and unchanged SQL behavior.

## Goals / Non-Goals

**Goals:**
- Centralize DuckDB string decoding in one shared helper.
- Document the unsafe contract once with explicit `SAFETY` comments.
- Ensure scalar invoke paths check NULL rows before reading argument strings.
- Preserve all existing function semantics and output values.

**Non-Goals:**
- Adding new SQL functions or changing signatures.
- Changing parse/normalization logic in movetext functions.
- Performance tuning beyond eliminating duplicate helper code.

## Decisions

1) Introduce a shared helper module for string decoding
- Decision: add a dedicated helper module under `src/chess/` (for example `duckdb_string.rs`) exposing one unsafe decoder used by scalar wrappers.
- Rationale: consolidates unsafe pointer/C-string handling in one location for easier review.
- Alternative considered: keep duplicated helpers and improve comments only. Rejected because duplication still risks divergence.

2) Preserve explicit unsafe-at-call-site pattern
- Decision: keep unsafe blocks at call sites where DuckDB C API values are consumed, but route decoding through the shared helper.
- Rationale: maintains local visibility of unsafe use while removing duplicated implementation details.
- Alternative considered: hide all unsafe calls behind fully safe wrappers. Rejected because call sites still need to enforce preconditions (NULL-row checks).

3) Require null checks before argument reads
- Decision: ensure each scalar `invoke` path performs null-row guards before decoding input strings.
- Rationale: this aligns with existing behavior and avoids accidental unsafe reads from NULLs.
- Alternative considered: central helper internally handling all nullability cases. Rejected because NULL detection belongs to vector-row context.

## Risks / Trade-offs

- [Missed call site still uses old helper] -> Mitigation: remove duplicated helper functions and compile-break stale references.
- [Accidental behavior change during refactor] -> Mitigation: keep existing tests and add targeted unit coverage for helper usage if needed.
- [Unsafe contract misunderstood] -> Mitigation: add concise, explicit `SAFETY` docs on helper and at critical call sites.

## Migration Plan

1. Add shared DuckDB string decoding helper module with `SAFETY` docs.
2. Replace duplicated helpers in scalar modules and delete old copies.
3. Verify null guards remain in all relevant invoke paths.
4. Run `make check` and `make test-rs` to confirm no behavioral regressions.

## Open Questions

- Should additional DuckDB scalar modules adopt the same helper now, or only the currently duplicated ones in this change?
