# DuckDB Chess Extension

A DuckDB extension for parsing and analyzing chess games in PGN format. This extension provides SQL-based access to chess game data, compatible with the Lichess database schema.

## Features

- **Parse PGN files**: Read chess games from PGN files with `read_pgn()`
- **Query chess data**: Filter games by opening, player, rating, time control, etc.
- **Movetext manipulation**: Normalize, hash, and extract move sequences
- **Glob pattern support**: Process multiple PGN files at once
- **Lichess compatibility**: Schema matches Lichess database exports

## Quick Start

### Prerequisites

- **Rust toolchain** (1.88.0 or newer): Install from [rustup.rs](https://rustup.rs/)
- **cargo-duckdb-ext-tools**: `cargo install cargo-duckdb-ext-tools`
- **DuckDB** (1.4.3+): For testing the extension

That's it! No Python, Make, or other dependencies required.

### Building

#### Debug Build
```shell
cargo duckdb-ext-build
```

This will:
1. Compile the extension with `cargo build`
2. Append DuckDB metadata to create `target/debug/duckdb_chess.duckdb_extension`

#### Release Build
```shell
cargo duckdb-ext-build -- --release
```

Or use the convenient Makefile wrapper:
```shell
make release
```

The extension will be created at `target/release/duckdb_chess.duckdb_extension`.

#### Manual Packaging (Advanced)

If you need explicit control over extension metadata:

```shell
# Build the library
cargo build --release

# Package with specific parameters
cargo duckdb-ext-pack \
  -i target/release/duckdb_chess.dll \
  -o target/release/duckdb_chess.duckdb_extension \
  -v v0.1.0 \
  -p windows_amd64 \
  -d v1.4.3
```

### Loading the Extension

Start DuckDB with the `-unsigned` flag to load local extensions:

```shell
duckdb -unsigned
```

Then load the extension:

```sql
LOAD './target/release/duckdb_chess.duckdb_extension';
```

### Example Usage

```sql
-- Load the extension
LOAD './target/release/duckdb_chess.duckdb_extension';

-- Read a PGN file
SELECT * FROM read_pgn('games.pgn') LIMIT 5;

-- Read multiple PGN files with glob pattern
SELECT COUNT(*) FROM read_pgn('lichess_db_*.pgn');

-- Filter by opening
SELECT Event, White, Black, Result 
FROM read_pgn('games.pgn')
WHERE Opening LIKE '%Sicilian%';

-- Normalize movetext (remove annotations)
SELECT chess_moves_normalize('1. e4 e5 2. Nf3 {A comment} Nc6') AS clean_moves;
-- Result: "e4 e5 Nf3 Nc6"

-- Hash movetext for deduplication
SELECT chess_moves_hash('e4 e5 Nf3 Nc6');

-- Convert to JSON
SELECT chess_moves_json('e4 e5 Nf3 Nc6');

-- Extract move subset
SELECT chess_moves_subset('e4 e5 Nf3 Nc6 Bb5', 3); -- First 3 moves
```

## Testing

### Rust Unit Tests

```shell
cargo test
```

This runs 20 unit tests covering the core chess parsing and filtering logic.

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
│   ├── moves.rs        # Movetext functions (hash, JSON, subset)
│   ├── filter.rs       # chess_moves_normalize() function
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

This extension is built for **DuckDB 1.4.3**. The extension includes version metadata and should work with DuckDB 1.4.3.

**Note**: The old template used `USE_UNSTABLE_C_API=1` which required exact version matching. The modern build system aims for better compatibility, but version matching may still be required depending on DuckDB API changes.

## Functions Reference

### Table Functions

#### `read_pgn(path_pattern: VARCHAR)`

Reads chess games from PGN files and returns a table with the Lichess schema.

**Columns**: Event, Site, White, Black, Result, WhiteElo, BlackElo, WhiteRatingDiff, BlackRatingDiff, ECO, Opening, TimeControl, Termination, UTCDate, UTCTime, moves

**Example**:
```sql
SELECT * FROM read_pgn('games.pgn');
SELECT * FROM read_pgn('lichess_db_2024-*.pgn'); -- Glob pattern
```

### Scalar Functions

#### `chess_moves_normalize(movetext: VARCHAR) -> VARCHAR`

Removes annotations, comments, variations, and numeric glyphs from movetext, returning a clean move sequence.

#### `chess_moves_hash(movetext: VARCHAR) -> VARCHAR`

Computes a hash of the normalized move sequence for deduplication.

#### `chess_moves_json(movetext: VARCHAR) -> JSON`

Converts movetext to JSON array format.

#### `chess_moves_subset(movetext: VARCHAR, count: INTEGER) -> VARCHAR`

Extracts the first `count` moves from the movetext.

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

See LICENSE file for details.

## Acknowledgments

- Built on DuckDB's extension framework
- Uses the modern [duckdb-ext-rs-template](https://github.com/redraiment/duckdb-ext-rs-template) by [@redraiment](https://github.com/redraiment)
- PGN parsing by [pgn-reader](https://github.com/niklasf/pgn-reader)
- Chess logic by [shakmaty](https://github.com/niklasf/shakmaty)
