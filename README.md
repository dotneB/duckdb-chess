# DuckDB Chess Extension

A DuckDB extension for parsing and analyzing chess games in PGN format.

The core idea: load raw PGNs into DuckDB via `read_pgn()`, then do opening detection, deduplication, and move/position analysis directly in SQL.


## Features

- **Parse PGN files** with `read_pgn()` (single file or glob patterns)
- **Lichess-style columns** (Event/Site/players/elos/opening/time control/movetext)
- **Movetext utilities**: normalize, hash, ply count
- **Position tracing**: convert movetext into per-ply JSON including FEN/EPD (useful for joining to openings datasets)

## Quick Start

```sql
-- Read a pgn file
SELECT Event, White, Black, Result, Termination, TimeControl FROM read_pgn('test/pgn_files/sample.pgn');

-- Read multiple pgn files
SELECT COUNT(*) FROM read_pgn('test/pgn_files/*.pgn');

-- How many games started with 1. e4 e5
SELECT COUNT_IF(chess_moves_subset('1. e4 e5', movetext))  FROM read_pgn('test/pgn_files/sample.pgn');

-- Removes comments/variations/NAGs and normalizes move numbers
SELECT chess_moves_normalize(movetext) FROM read_pgn('test/pgn_files/sample.pgn');

-- Zobrist hash of the final mainline position
SELECT chess_moves_hash('1. e4 e5 2. Nf3 Nc6') AS hash;

-- Ply count
SELECT chess_ply_count('1. e4 e5 2. Nf3') AS ply;

-- Converts FEN to EPD
SELECT chess_fen_epd('rnbq1rk1/1pp1bppp/p3pn2/8/2pP4/2N2NP1/PP2PPBP/R1BQ1RK1 w - - 0 8') AS epd;

-- Get the first 40 moves of a game as ply, san and epd
WITH g AS (
  SELECT movetext
  FROM read_pgn('test/pgn_files/sample.pgn')
  WHERE parse_error IS NULL
  LIMIT 1
)
SELECT
  json_extract(m.value, '$.ply')::INT AS ply,
  json_extract_string(m.value, '$.move') AS san,
  json_extract_string(m.value, '$.epd') AS epd
FROM g,
     json_each(CAST(chess_moves_json(g.movetext, 40) AS JSON)) m;
```

## Development

### Prerequisites

- **Rust toolchain**: repo toolchain is `1.93` (`rust-toolchain.toml`), minimum supported Rust version for this crate (MSRV) is `1.89`
- **DuckDB** `1.4.4`
- Run `just install-tools` to install:
  - **cargo-duckdb-ext-tools**: A Rust-based toolkit for building and packaging DuckDB extensions without Python dependencies
  - **duckdb-slt**: A Rust-based sqllogictest runner for DuckDB.

### Build and Test Commands

```shell
# format + lint
just check

# build
just debug # (debug)
just release # (release)

# tests (unit + SQLLogicTest)
just test # (debug)
just test-release # (release)

# main dev loop
just dev # (debug)
```

The `just` recipes call Rust-first commands (`cargo duckdb-ext-build`, `cargo test`, `cargo fmt`, `cargo clippy`).

### Template Compatibility Note

`extension-ci-tools/` is kept for DuckDB community extension template compatibility. Local Rust-first targets above do not require Python/venv, but template/CI compatibility targets may.

### Load

Local builds are unsigned; start DuckDB with `-unsigned`:

```shell
duckdb -unsigned
```

Then:

```sql
LOAD './target/release/chess.duckdb_extension';
```

### Contributing

1. Make changes to the source code
2. Run the main workflow: `just dev`
3. Run release checks: `just test-release`
4. Test manually with DuckDB CLI

## Usage

### Read PGN

```sql
LOAD './target/release/chess.duckdb_extension';

SELECT Event, White, Black, Result, Opening
FROM read_pgn('games.pgn')
WHERE parse_error IS NULL
LIMIT 10;

-- Glob patterns work too
SELECT count(*)
FROM read_pgn('lichess_db_2024-*.pgn');
```

Notes:

- Glob expansion currently triggers when `path_pattern` contains `*` or `?`.
- `movetext` is mainline only; variations are skipped, `{ ... }` comments are preserved.
- Terminal result markers are not appended to `movetext`; use the `Result` column for game result metadata.
- If a game fails to parse, you still get a row with `parse_error` set.
- When reading multiple files (via glob), unreadable files are skipped with a warning; a single explicit file path fails hard.

### Clean / Hash / Count Moves

```sql
SELECT chess_moves_normalize('1. e4! {comment} e5?? $1 2. Nf3') AS clean;
-- clean = '1. e4 e5 2. Nf3'

SELECT chess_moves_hash('e4 e5 Nf3 Nc6') AS h;          -- UBIGINT
SELECT chess_ply_count('1. e4 e5 2. Nf3') AS ply_count;  -- BIGINT
```

### Subset Filtering Patterns

Use the pattern that matches your data quality and workload.

| Data shape                                                        | Recommended pattern                  | Why                              |
| ----------------------------------------------------------------- | ------------------------------------ | -------------------------------- |
| Raw/noisy movetext (comments, NAGs, variations, mixed formatting) | `chess_moves_subset(short, long)`    | Parser-backed subset semantics   |
| Canonical/materialized movetext (already normalized)              | `starts_with(long_norm, short_norm)` | Faster repeated prefix filtering |

Raw/noisy workflow:

```sql
SELECT *
FROM read_pgn('games/*.pgn')
WHERE chess_moves_subset('1. e4 e5 2. Nf3', movetext);
```

Canonical/materialized workflow:

```sql
CREATE OR REPLACE TABLE games_norm AS
SELECT
  *,
  chess_moves_normalize(movetext) AS movetext_norm
FROM read_pgn('games/*.pgn')
WHERE parse_error IS NULL;

WITH needle AS (
  SELECT chess_moves_normalize('1. e4 e5 2. Nf3') AS short_norm
)
SELECT g.*
FROM games_norm g, needle n
WHERE starts_with(g.movetext_norm, n.short_norm);
```

`contains` is not subset-prefix semantics:

```sql
SELECT
  contains('1. d4 d5 2. e4 e5', 'e4 e5') AS contains_match,
  chess_moves_subset('e4 e5', '1. d4 d5 2. e4 e5') AS subset_match;
-- contains_match = true, subset_match = false
```

Caveats:

- `read_pgn(...).movetext` does not include a terminal result marker (`1-0`, `0-1`, `1/2-1/2`, `*`).
- `starts_with` is only safe when both sides are canonicalized with the same normalization pipeline.
- If input quality is uncertain, prefer `chess_moves_subset`.

### Turn Movetext Into Positions (FEN/EPD)

`chess_moves_json()` returns a JSON string (cast to `JSON` if you want to use JSON functions).

```sql
WITH g AS (
  SELECT movetext
  FROM read_pgn('test/pgn_files/sample.pgn')
  WHERE parse_error IS NULL
  LIMIT 1
)
SELECT
  json_extract(m.value, '$.ply')::INT AS ply,
  json_extract_string(m.value, '$.move') AS san,
  json_extract_string(m.value, '$.fen') AS fen,
  json_extract_string(m.value, '$.epd') AS epd
FROM g,
     json_each(CAST(chess_moves_json(g.movetext, 40) AS JSON)) m;
```

### Opening Detection Join (Example)

Assumes an `openings` table with columns `epd`, `eco`, `name`, and a per-opening mainline in `uci`.

If your openings dataset is a Parquet file, a typical setup looks like:

```sql
CREATE OR REPLACE TABLE openings AS
SELECT * FROM read_parquet('openings.parquet');
```

```sql
WITH params AS (
  SELECT max(array_length(string_split(uci, ' '))) AS max_opening_ply
  FROM openings
),
games AS (
  SELECT row_number() OVER () AS game_id, movetext
  FROM read_pgn('games.pgn')
  WHERE parse_error IS NULL
),
pos AS (
  SELECT
    g.game_id,
    json_extract(m.value, '$.ply')::INT AS ply,
    trim(json_extract_string(m.value, '$.epd')) AS epd
  FROM games g,
       params p,
       json_each(CAST(chess_moves_json(g.movetext, p.max_opening_ply) AS JSON)) m
),
matches AS (
  SELECT p.game_id, p.ply, o.eco, o.name
  FROM pos p
  JOIN openings o ON trim(o.epd) = p.epd
)
SELECT game_id, eco, name
FROM (
  SELECT *, row_number() OVER (PARTITION BY game_id ORDER BY ply DESC) AS rn
  FROM matches
)
WHERE rn = 1;
```

## API Reference

### Table Functions

#### `read_pgn(path_pattern: VARCHAR)`

Reads chess games from one or more PGN files.

`path_pattern` can be a single path or a glob pattern (e.g. `lichess_db_2024-*.pgn`).

Returned columns:

| Column      | Type     | Notes                                                               |
| ----------- | -------- | ------------------------------------------------------------------- |
| Event       | VARCHAR  | PGN tag                                                             |
| Site        | VARCHAR  | PGN tag                                                             |
| White       | VARCHAR  | PGN tag                                                             |
| Black       | VARCHAR  | PGN tag                                                             |
| Result      | VARCHAR  | PGN tag                                                             |
| WhiteTitle  | VARCHAR  | PGN tag (nullable)                                                  |
| BlackTitle  | VARCHAR  | PGN tag (nullable)                                                  |
| WhiteElo    | UINTEGER | PGN tag (nullable)                                                  |
| BlackElo    | UINTEGER | PGN tag (nullable)                                                  |
| UTCDate     | DATE     | PGN tag (nullable)                                                  |
| UTCTime     | TIMETZ   | PGN tag (nullable, treated as UTC)                                  |
| ECO         | VARCHAR  | PGN tag                                                             |
| Opening     | VARCHAR  | PGN tag                                                             |
| Termination | VARCHAR  | PGN tag                                                             |
| TimeControl | VARCHAR  | PGN tag                                                             |
| movetext    | VARCHAR  | Mainline only, includes `{...}` comments, no terminal result marker |
| parse_error | VARCHAR  | NULL on success; error message on failure                           |
| Source      | VARCHAR  | PGN tag (nullable)                                                  |

### Scalar Functions

| Function                                            | Returns | Notes                                                                                                            |
| --------------------------------------------------- | ------- | ---------------------------------------------------------------------------------------------------------------- |
| `chess_moves_normalize(movetext)`                   | VARCHAR | Removes comments/variations/NAGs and normalizes move numbers                                                     |
| `chess_moves_hash(movetext)`                        | UBIGINT | Zobrist hash of the final mainline position (comments/variations/NAGs ignored); NULL for empty/unparseable input |
| `chess_ply_count(movetext)`                         | BIGINT  | Ply count (NULL-safe macro)                                                                                      |
| `chess_moves_json(movetext, max_ply := NULL)`       | VARCHAR | JSON string of `{ply, move, fen, epd}` (NULL-safe macro)                                                         |
| `chess_fen_epd(fen)`                                | VARCHAR | Converts FEN to EPD join key (board/side/castling/ep)                                                            |
| `chess_moves_subset(short_movetext, long_movetext)` | BOOLEAN | True if `short` mainline is a prefix of `long` mainline                                                          |


## License

MIT. See `LICENSE`.

## Acknowledgments

- Built on DuckDB's extension framework
- Uses the modern [duckdb-ext-rs-template](https://github.com/redraiment/duckdb-ext-rs-template) by [@redraiment](https://github.com/redraiment)
- PGN parsing by [pgn-reader](https://github.com/niklasf/pgn-reader)
- Chess logic by [shakmaty](https://github.com/niklasf/shakmaty)
