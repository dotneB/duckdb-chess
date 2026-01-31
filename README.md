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

- **Rust toolchain** (1.89+): https://rustup.rs/
- **DuckDB** built for 1.4.4
- Run `make install-tools` to install:
  - **cargo-duckdb-ext-tools**: A Rust-based toolkit for building and packaging DuckDB extensions without Python dependencies
  - **duckdb-slt**: A Rust-based sqllogictest runner for DuckDB.

#### Note on Python:
This project migrated to use the [duckdb-ext-rs-template](https://github.com/redraiment/duckdb-ext-rs-template) by [@redraiment](https://github.com/redraiment). This toolset only requires Rust to be installed to build the extension.  
I do keep the `extension-ci-tools` submodule and the original commands from the [official community template](https://github.com/duckdb/extension-template-rs) in `Makefile` to be able to test in CI if the same build would succeed when submitting to the [Community Extensions Repository](https://github.com/duckdb/community-extensions/). Running any of these commands do require python and python-env.

### Build

Debug:

```shell
cargo duckdb-ext-build
# or use the wrapper to run (build + check + tests)
make dev
```

Release:

```shell
cargo duckdb-ext-build -- --release
# or use the wrapper
make release-rs
```

Artifacts:

- `target/debug/chess.duckdb_extension`
- `target/release/chess.duckdb_extension`

#### Load

Local builds are unsigned; start DuckDB with `-unsigned`:

```shell
duckdb -unsigned
```

Then:

```sql
LOAD './target/release/chess.duckdb_extension';
```

### Rust Unit Tests

```shell
cargo test
```

This runs Rust unit tests covering the core chess parsing and move analysis logic.

### Integration Tests

```shell
duckdb-slt.exe -e ./target/debug/$(EXTENSION_NAME).duckdb_extension -u -w "$(CURDIR)" "$(CURDIR)/test/sql/*.test"
```
This runs the SQLLogicTest against the tests in `test/sql/`

### Version Compatibility

This extension is built for **DuckDB 1.4.4**.

**Note**: The old template used `USE_UNSTABLE_C_API=1` which required exact version matching. The modern build system aims for better compatibility, but version matching may still be required depending on DuckDB API changes.

### Architecture

This extension uses:
- **Rust 2024 Edition** for modern language features
- **duckdb-ext-macros** (0.1.0) for extension macros
- **cargo-duckdb-ext-tools** for packaging
- **pgn-reader** (0.28) for PGN parsing
- **shakmaty** (0.29) for chess logic

The build system is pure Rust with no Python or Make dependencies required for building (though the Makefile is provided for convenience).

### Contributing

1. Make changes to the source code
2. Run dev wrapper: `make dev`
3. Bun release wrapper: `make test-release-rs`
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
- `movetext` is the mainline only; variations are skipped, `{ ... }` comments are preserved.
- If a game fails to parse, you still get a row with `parse_error` set.
- When reading multiple files (via glob), unreadable files are skipped with a warning; a single explicit file path fails hard.

### Clean / Hash / Count Moves

```sql
SELECT chess_moves_normalize('1. e4! {comment} e5?? $1 2. Nf3') AS clean;
-- clean = '1. e4 e5 2. Nf3'

SELECT chess_moves_hash('e4 e5 Nf3 Nc6') AS h;          -- UBIGINT
SELECT chess_ply_count('1. e4 e5 2. Nf3') AS ply_count;  -- BIGINT
```

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

| Column | Type | Notes |
|---|---|---|
| Event | VARCHAR | PGN tag |
| Site | VARCHAR | PGN tag |
| White | VARCHAR | PGN tag |
| Black | VARCHAR | PGN tag |
| Result | VARCHAR | PGN tag |
| WhiteTitle | VARCHAR | PGN tag (nullable) |
| BlackTitle | VARCHAR | PGN tag (nullable) |
| WhiteElo | UINTEGER | PGN tag (nullable) |
| BlackElo | UINTEGER | PGN tag (nullable) |
| UTCDate | DATE | PGN tag (nullable) |
| UTCTime | TIMETZ | PGN tag (nullable, treated as UTC) |
| ECO | VARCHAR | PGN tag |
| Opening | VARCHAR | PGN tag |
| Termination | VARCHAR | PGN tag |
| TimeControl | VARCHAR | PGN tag |
| movetext | VARCHAR | Raw mainline, includes `{...}` comments |
| parse_error | VARCHAR | NULL on success; error message on failure |

### Scalar Functions

| Function | Returns | Notes |
|---|---|---|
| `chess_moves_normalize(movetext)` | VARCHAR | Removes comments/variations/NAGs and normalizes move numbers |
| `chess_moves_hash(movetext)` | UBIGINT | Zobrist hash of the final mainline position (comments/variations/NAGs ignored); NULL for empty/unparseable input |
| `chess_ply_count(movetext)` | BIGINT | Ply count (NULL-safe macro) |
| `chess_moves_json(movetext, max_ply := NULL)` | VARCHAR | JSON string of `{ply, move, fen, epd}` (NULL-safe macro) |
| `chess_fen_epd(fen)` | VARCHAR | Converts FEN to EPD join key (board/side/castling/ep) |
| `chess_moves_subset(short_movetext, long_movetext)` | BOOLEAN | True if normalized `short` is a prefix of normalized `long` |


## License

MIT. See `LICENSE`.

## Acknowledgments

- Built on DuckDB's extension framework
- Uses the modern [duckdb-ext-rs-template](https://github.com/redraiment/duckdb-ext-rs-template) by [@redraiment](https://github.com/redraiment)
- PGN parsing by [pgn-reader](https://github.com/niklasf/pgn-reader)
- Chess logic by [shakmaty](https://github.com/niklasf/shakmaty)
