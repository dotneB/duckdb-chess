# DuckDB Chess Extension

A DuckDB extension for parsing and analyzing chess games in PGN format.

The core idea: load raw PGNs into DuckDB via `read_pgn()`, then do opening detection, deduplication, and move/position analysis directly in SQL.

Typical use cases:

- Explore a PGN archive with SQL (filters, aggregates, sampling)
- Join games to an openings dataset via EPD keys (from `chess_moves_json()` or `chess_fen_epd()`)
- Deduplicate by normalized mainline (`chess_moves_hash()`)

## Contents

- Quick Start
- Usage
- API Reference
- Testing
- Development

## Features

- **Parse PGN files** with `read_pgn()` (single file or glob patterns)
- **Lichess-style columns** (Event/Site/players/elos/opening/time control/movetext)
- **Parse diagnostics** via a `parse_error` column (keep going on bad games)
- **Movetext utilities**: normalize, hash, ply count
- **Position tracing**: convert movetext into per-ply JSON including FEN/EPD (useful for joining to openings datasets)

## Quick Start

### Prerequisites

- **Rust toolchain** (1.88.0+): https://rustup.rs/
- **cargo-duckdb-ext-tools**: `cargo install cargo-duckdb-ext-tools`
- **DuckDB** (built for 1.4.3; CLI/Python/etc. all work)

### Build

Debug:

```shell
cargo duckdb-ext-build
```

Release:

```shell
cargo duckdb-ext-build -- --release
```

Makefile wrapper (optional):

```shell
make release
```

Artifacts:

- `target/debug/duckdb_chess.duckdb_extension`
- `target/release/duckdb_chess.duckdb_extension`

### Load

Local builds are unsigned; start DuckDB with `-unsigned`:

```shell
duckdb -unsigned
```

Then:

```sql
LOAD './target/release/duckdb_chess.duckdb_extension';
```

If you're using DuckDB from another runtime (Python/R/Node), you still load the same extension file; only the connection setup changes.

### Try It (Repo Sample PGNs)

```sql
LOAD './target/release/duckdb_chess.duckdb_extension';

SELECT Event, White, Black, Result, Opening
FROM read_pgn('test/pgn_files/sample.pgn')
WHERE parse_error IS NULL;

-- Inspect parse errors (if any)
SELECT count_if(parse_error IS NULL) AS ok, count_if(parse_error IS NOT NULL) AS bad
FROM read_pgn('test/pgn_files/parse_errors.pgn');
```

## Usage

### Read PGN

```sql
LOAD './target/release/duckdb_chess.duckdb_extension';

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
-- clean = 'e4 e5 Nf3'

SELECT chess_moves_hash('e4 e5 Nf3 Nc6') AS h;          -- BIGINT
SELECT chess_ply_count('1. e4 e5 2. Nf3') AS ply_count;  -- BIGINT
```

### Turn Movetext Into Positions (FEN/EPD)

`chess_moves_json()` returns a JSON string (cast to `JSON` if you want to use JSON functions).

```sql
WITH g AS (
  SELECT movetext
  FROM read_pgn('games.pgn')
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

### DuckDB Python (Example)

```python
import duckdb

con = duckdb.connect()
con.execute("LOAD './target/release/duckdb_chess.duckdb_extension'")
df = con.execute("""
  SELECT White, Black, Result
  FROM read_pgn('test/pgn_files/sample.pgn')
  WHERE parse_error IS NULL
""").df()
print(df.head())
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
| WhiteElo | VARCHAR | PGN tag (nullable) |
| BlackElo | VARCHAR | PGN tag (nullable) |
| UTCDate | VARCHAR | PGN tag |
| UTCTime | VARCHAR | PGN tag |
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
| `chess_moves_hash(movetext)` | BIGINT | Hash of the normalized movetext |
| `chess_ply_count(movetext)` | BIGINT | Ply count (NULL-safe macro) |
| `chess_moves_json(movetext, max_ply := NULL)` | VARCHAR | JSON string of `{ply, move, fen, epd}` (NULL-safe macro) |
| `chess_fen_epd(fen)` | VARCHAR | Converts FEN to EPD join key (board/side/castling/ep) |
| `chess_moves_subset(short_movetext, long_movetext)` | BOOLEAN | True if normalized `short` is a prefix of normalized `long` |

## Testing

### Rust Unit Tests

```shell
cargo test
```

This runs Rust unit tests covering the core chess parsing and move analysis logic.

### Manual Testing

Build the extension and test it manually with DuckDB:

```shell
# Build the extension
cargo duckdb-ext-build -- --release

# Test with DuckDB CLI
duckdb -unsigned
```

Then in DuckDB:
```sql
LOAD './target/release/duckdb_chess.duckdb_extension';
SELECT * FROM read_pgn('test/pgn_files/sample.pgn') LIMIT 5;
SELECT chess_moves_normalize('1. e4 e5 2. Nf3 {comment} Nc6');
```

Test PGN files are available in `test/pgn_files/`.

## Development

### Project Structure

```
src/
├── chess/
│   ├── mod.rs          # Extension entry point
│   ├── reader.rs       # read_pgn() table function
│   ├── visitor.rs      # PGN parsing logic
│   ├── moves.rs        # Move analysis (JSON, hash, ply count, subset, FEN->EPD)
│   ├── filter.rs       # Movetext normalization
│   └── types.rs        # Shared types
├── lib.rs              # Crate root
└── wasm_lib.rs         # WASM target (experimental)
```

### Available Make Targets

The Makefile provides convenient wrappers around cargo commands:

```shell
make build          # Build debug extension
make release        # Build release extension
make test           # Run Rust unit tests
make clean          # Clean build artifacts
make install-tools  # Install cargo-duckdb-ext-tools
make help           # Show all available targets
```

### Direct Cargo Commands

You can also use cargo directly:

```shell
cargo duckdb-ext-build                    # Debug build
cargo duckdb-ext-build -- --release       # Release build
cargo test                                # Run tests
cargo clean                               # Clean artifacts
```

## Version Compatibility

This extension is built for **DuckDB 1.4.3**.

**Note**: The old template used `USE_UNSTABLE_C_API=1` which required exact version matching. The modern build system aims for better compatibility, but version matching may still be required depending on DuckDB API changes.

## Functions Reference

This section is kept for historical links; see `API Reference` for the current, accurate function signatures.

## Architecture

This extension uses:
- **Rust 2024 Edition** for modern language features
- **duckdb-ext-macros** (0.1.0) for extension macros
- **cargo-duckdb-ext-tools** for packaging
- **pgn-reader** (0.28) for PGN parsing
- **shakmaty** (0.29) for chess logic

The build system is pure Rust with no Python or Make dependencies required for building (though the Makefile is provided for convenience).

## Contributing

1. Make changes to the source code
2. Run tests: `cargo test`
3. Build the extension: `cargo duckdb-ext-build -- --release`
4. Test manually with DuckDB CLI

## License

MIT. See `LICENSE`.

## Acknowledgments

- Built on DuckDB's extension framework
- Uses the modern [duckdb-ext-rs-template](https://github.com/redraiment/duckdb-ext-rs-template) by [@redraiment](https://github.com/redraiment)
- PGN parsing by [pgn-reader](https://github.com/niklasf/pgn-reader)
- Chess logic by [shakmaty](https://github.com/niklasf/shakmaty)
