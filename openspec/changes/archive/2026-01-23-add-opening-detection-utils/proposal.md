# Change: Add Opening Detection Utility Functions

## Why
Opening/ECO detection is best performed in the data pipeline (e.g., DuckDB SQL) by joining game positions against the Lichess openings dataset (available as Parquet). Implementing opening detection inside the extension would add unnecessary dependency/licensing risk and make the extension heavier than necessary.

The extension already provides `chess_moves_json(movetext)` which emits per-move FENs. To make pipeline-side opening detection straightforward and consistent, the extension should provide minimal utilities that align with the Lichess openings dataset key (`epd`).

## What Changes
- **MODIFIED** capability `move-analysis`:
  - Extend `chess_moves_json(movetext)` output objects to include `epd` derived from the FEN (first 4 fields: `board side castling ep`).
  - Add an overload `chess_moves_json(movetext, max_ply)` to cap expansion for pipeline use.
  - Add a new scalar function `chess_fen_epd(fen)` that returns the EPD string used by the Lichess openings dataset.
  - Add a new scalar function `chess_ply_count(movetext)` that quickly counts plies in PGN movetext.
## Dataset Notes / Decision
The Lichess openings dataset recommends classifying games by scanning positions from the end of the game backwards until a named position is found. In SQL, this is implemented by generating an EPD per ply, joining to the dataset on `epd`, and selecting the match with the highest ply.

This change chooses to include `epd` directly in the `chess_moves_json` output for two reasons:
- Performance: avoids per-row `chess_fen_epd(fen)` calls when exploding many plies.
- Ergonomics: keeps the opening join query short and readable.

To further reduce pipeline cost, `chess_moves_json` is extended with an overload that limits the number of returned plies (e.g., limit to the maximum opening ply present in the reference dataset).

`chess_ply_count(movetext)` is added to support quick ply counting and to help compute bounds in SQL (e.g., `least(chess_ply_count(movetext), max_opening_ply)`).
`chess_fen_epd(fen)` is still provided as a standalone utility and as a single canonical definition of the EPD format.
- **NO CHANGE** to `read_pgn()` schema:
  - `ECO` and `Opening` columns remain the raw PGN header values.
  - Opening detection is performed in downstream SQL/pipeline logic.

## Impact
- Affected specs: `move-analysis`
- Affected code (future implementation): `src/chess/moves.rs`
- Breaking change: **YES** (behavioral) — `chess_moves_json` JSON schema changes by adding a new field `epd` to each element.
- Migration: pipeline SQL that expects only `{ply, move, fen}` should tolerate extra keys (most JSON processing will). If any consumer validates the JSON schema strictly, it must be updated.

## Out of Scope
- Performing ECO/opening detection inside the extension
- Shipping an embedded opening book dataset
- Creating a first-class `openings` table function (DuckDB can read the Parquet dataset directly)
