## Context

`read_pgn(path_pattern)` streams PGN parsing and exposes a fixed, Lichess-style set of columns plus `parse_error`. Many PGN exports include a `Source` header tag (provenance / attribution / import pipeline), but the extension currently ignores it.

This change adds `Source` as a new nullable column, appended to the end of the `read_pgn` output, and wires the tag through parsing, record storage, and DuckDB vector output.

## Goals / Non-Goals

**Goals:**

- Parse the PGN header tag `Source` and expose it as a new `Source` column in `read_pgn` output.
- Preserve existing behavior for malformed games: still output a row and populate `parse_error` while keeping any successfully parsed header fields (including `Source` if seen before the error).
- Keep streaming characteristics unchanged (no whole-file buffering; no additional per-game heavy allocations beyond one optional string field).

**Non-Goals:**

- No attempt to infer/derive `Source` from file paths or other headers.
- No normalization or canonicalization of `Source` values beyond the existing header-value handling.
- No new scalar functions; this is strictly a schema + parsing addition for `read_pgn`.

## Decisions

- Append the `Source` column at the end of the `read_pgn` column list.
  - Rationale: minimizes disruption to consumers selecting columns positionally; still a breaking schema change, but avoids reordering existing columns.

- Treat `Source` like other text header fields:
  - Missing tag => SQL `NULL`.
  - Present but empty => empty string (preserve the distinction the project already makes between missing vs empty).

- Parse tag name matching consistent with existing header extraction:
  - Recognize the `Source` tag by the same comparison strategy used for other known tags (avoid introducing special-case casing behavior for a single field).

- Update schema documentation + tests as part of the change:
  - Data schema capability spec and README should reflect 18 columns and include `Source` in the documented list.
  - SQLLogicTests that validate schema shape must be updated for the added column.

## Risks / Trade-offs

- Breaking change for queries that rely on `SELECT *` column count or positional unpacking.
  - Mitigation: append-only; update docs/tests; recommend explicit column lists in examples.

