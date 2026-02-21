## [0.5.0] - 2026-02-21

### 🚀 Features

- Support for zstd compressed files

### 🚜 Refactor

- Cache epoch date as static
- Use SmallVec preallocated for MoveList
- Attempt to reduce string clones
- Macro for visitor boilerplate
- Consolidate error handling
- Read-pgn vtab
- Minor optimization read pgn allocation
- Duckdb scalar helpers
- Harden read_pgn runtime resilience and surface glob warnings
- Harden inferred timecontrol overflow handling
- Isolate bindinfo ffi boundary and regroup duckdb helpers
- Modularize timecontrol parser into strict, inference, and json modules
- Harden timecontrol quote preprocessing
- Extend TimeControl normalization

### 📚 Documentation

- Update README.md

### ⚡ Performance

- Static pre-compile Regexes

### ⚙️ Miscellaneous Tasks

- Add support for a devcontainer
- Update extension-ci-tools submodule
- Zstd compiling in mingw

## [0.4.0] - 2026-02-15

### 🚀 Features

- Add TimeControl tag normalization utilities
- Add chess_timecontrol_category

### 🐛 Bug Fixes

- Attempting to reduce the amount of memory allocations
- Increased coverage for timecontrol normalization of LumbraGigabase

### ⚙️ Miscellaneous Tasks

- More portable justfile

## [0.3.1] - 2026-02-08

### 🐛 Bug Fixes

- Clamp out of range day

## [0.3.0] - 2026-02-07

### 🚀 Features

- Add parsing of the Source header
- Chess_moves_subset early out
- Improve parsing error handling

### 🚜 Refactor

- Centralize duckdb_string handling

### ⚙️ Miscellaneous Tasks

- Update specs and README.md
- Split Makefile and justfile

## [0.2.0] - 2026-02-03

### 🚀 Features

- Use chrono lib instead of custom date/time parsing
- Chess_moves_normalize uses pgn-reader Visitor instead of custom logic
- Chess_moves_hash uses shakmaty zobrist hashing

### 🐛 Bug Fixes

- Chess_moves_normalized shouldn't remove move numbers

### ⚡ Performance

- Faster debug builds

### ⚙️ Miscellaneous Tasks

- Improve CHANGELOG.md generation
- Bump pgn-reader to 0.29, shakmaty to 0.30
- Update openspec

## [0.1.2] - 2026-01-27

### 🚀 Features

- Bump duckdb to 1.4.4

## [0.1.1] - 2026-01-26

### 🚀 Features

- Duckdb-slt@0.1.3 now support require keyword

### 🐛 Bug Fixes

- Returned types for WhiteElo, BlackElo, UTCDate, UTCTime as their proper types
- UTCtime parsing

### 🚜 Refactor

- Renamed extension to chess

### 📚 Documentation

- LICENSE in readme
- Simplify README.md

### 🧪 Testing

- Fix expected columns
- Change expected value for the UTCTime TIMETZ includes the tz
- Communit flow with tests enabled

### ⚙️ Miscellaneous Tasks

- Generate notes for the release

## [0.1.0] - 2026-01-24

### 🚀 Features

- Add parse error column
- Handle invalid utf8
- Optimize memory usage
- Refactor lib modules
- Add a function to export moves from a movetext
- Add game deduplication
- Migrate to redraiment duckdb extensions helpers
- Use PNG Reader read_games
- Use shakmaty to keep position
- Use duckdb-slt
- Add opening detection utilities

### 🚜 Refactor

- Test suite
- Filter movetext
- Into module

### 📚 Documentation

- Update README.md

### ⚙️ Miscellaneous Tasks

- Scaffold foundation specs
- Clean up
- Formatting
- Cleanup
- Restore community extension-ci-tools
- Add LICENSE
- Pin dependencies
- Community github workflow is optional
- Release process
- Fix gh token usage
- Fix extension names in release

