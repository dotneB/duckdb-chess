# Data Schema Capability Delta

## MODIFIED Requirements

### Requirement: Lichess Schema Compatibility
The system SHALL expose PGN data through a schema that extends the Lichess database export format with additional diagnostic columns.

#### Scenario: Schema column count
- **WHEN** querying the `read_pgn` table function
- **THEN** the result contains exactly 17 columns (16 Lichess columns + 1 diagnostic column)

#### Scenario: Column names include parse_error
- **WHEN** describing the table structure
- **THEN** column names include all Lichess columns (Event, Site, White, Black, Result, WhiteTitle, BlackTitle, WhiteElo, BlackElo, UTCDate, UTCTime, ECO, Opening, Termination, TimeControl, movetext) plus the parse_error column

## ADDED Requirements

### Requirement: Parse Error Column
The system SHALL provide a `parse_error` column containing diagnostic information about parsing failures.

#### Scenario: Parse error column presence
- **WHEN** querying the `read_pgn` table function
- **THEN** a `parse_error` column (VARCHAR, nullable) is included as the 17th column

#### Scenario: Successful game parsing
- **WHEN** a game parses successfully without errors
- **THEN** the parse_error column contains NULL

#### Scenario: Failed game parsing
- **WHEN** a game encounters a parsing error
- **THEN** the parse_error column contains a descriptive error message string

#### Scenario: Query for problematic games
- **WHEN** user executes `SELECT * FROM read_pgn('file.pgn') WHERE parse_error IS NOT NULL`
- **THEN** only games that encountered parsing errors are returned

#### Scenario: Query for successful games
- **WHEN** user executes `SELECT * FROM read_pgn('file.pgn') WHERE parse_error IS NULL`
- **THEN** only games that parsed successfully without errors are returned

### Requirement: Partial Game Data Preservation
The system SHALL preserve successfully parsed data even when parsing fails at any stage (headers, movetext, or file reading).

#### Scenario: Header parsing succeeds, movetext fails
- **WHEN** a game's headers parse successfully but movetext parsing fails
- **THEN** the output row contains all successfully parsed header values (Event, Site, White, Black, etc.) and the parse_error column contains the movetext parsing error

#### Scenario: Header parsing fails early
- **WHEN** header parsing fails (e.g., malformed header syntax)
- **THEN** the output row contains any headers successfully parsed before the error and the parse_error column indicates the header parsing failure

#### Scenario: Partial data with error message
- **WHEN** a game has parsing errors at any stage
- **THEN** the game is still output as a row with available data fields populated and parse_error indicating what failed and at which stage

### Requirement: Error Message Format
The system SHALL provide clear, actionable error messages in the parse_error column that indicate the parsing stage and nature of the failure.

#### Scenario: Error message includes context
- **WHEN** a parsing error occurs
- **THEN** the parse_error message includes relevant context such as game number, file location, and error description

#### Scenario: Error message for movetext failures
- **WHEN** movetext parsing fails
- **THEN** the parse_error message clearly indicates that movetext parsing failed and the nature of the failure

#### Scenario: Error message for header failures
- **WHEN** header parsing fails
- **THEN** the parse_error message clearly indicates that header parsing failed and which header or what issue occurred

#### Scenario: Error stage identification
- **WHEN** any parsing error occurs
- **THEN** the parse_error message allows users to distinguish between header parsing errors, movetext parsing errors, and file reading errors
