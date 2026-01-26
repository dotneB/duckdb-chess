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

- Add "Running the extension" section to README
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
- Release 0.1.0
- Fix gh token usage
- Fix extension names in release
