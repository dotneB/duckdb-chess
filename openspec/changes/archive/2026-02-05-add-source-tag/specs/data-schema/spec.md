## MODIFIED Requirements

### Requirement: Lichess Schema Compatibility
The system SHALL expose PGN data through a schema that extends the Lichess database export format with additional diagnostic columns.

#### Scenario: Schema column count
- **WHEN** querying the `read_pgn` table function
- **THEN** the result contains exactly 18 columns (16 Lichess columns + 2 diagnostic columns)

#### Scenario: Column names include parse_error
- **WHEN** describing the table structure
- **THEN** column names include all Lichess columns (Event, Site, White, Black, Result, WhiteTitle, BlackTitle, WhiteElo, BlackElo, UTCDate, UTCTime, ECO, Opening, Termination, TimeControl, movetext) plus the parse_error column and the Source column
