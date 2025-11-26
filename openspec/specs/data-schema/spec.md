# Data Schema Capability

## Purpose
To define the database schema for representing chess games, ensuring compatibility with the Lichess export format and standard SQL types.

## Requirements

### Requirement: Lichess Schema Compatibility
The system SHALL expose PGN data through a schema that extends the Lichess database export format with additional diagnostic columns.

#### Scenario: Schema column count
- **WHEN** querying the `read_pgn` table function
- **THEN** the result contains exactly 17 columns (16 Lichess columns + 1 diagnostic column)

#### Scenario: Column names include parse_error
- **WHEN** describing the table structure
- **THEN** column names include all Lichess columns (Event, Site, White, Black, Result, WhiteTitle, BlackTitle, WhiteElo, BlackElo, UTCDate, UTCTime, ECO, Opening, Termination, TimeControl, movetext) plus the parse_error column

### Requirement: Core Game Information
The system SHALL provide columns for essential game metadata.

#### Scenario: Event column
- **WHEN** a game has an Event header
- **THEN** the Event column contains the event name or NULL if not present

#### Scenario: Site column
- **WHEN** a game has a Site header
- **THEN** the Site column contains the site URL or identifier or NULL if not present

#### Scenario: Result column
- **WHEN** a game has a Result header
- **THEN** the Result column contains one of: "1-0" (white wins), "0-1" (black wins), "1/2-1/2" (draw), "*" (ongoing), or NULL

### Requirement: Player Information
The system SHALL provide columns for player names and titles.

#### Scenario: White player column
- **WHEN** a game has a White header
- **THEN** the White column contains the white player's name or NULL if not present

#### Scenario: Black player column
- **WHEN** a game has a Black header
- **THEN** the Black column contains the black player's name or NULL if not present

#### Scenario: Player title columns
- **WHEN** a game has WhiteTitle or BlackTitle headers
- **THEN** the respective columns contain titles (GM, IM, FM, etc.) or NULL if not present

### Requirement: Player Rating Information
The system SHALL provide columns for player ELO ratings stored as VARCHAR.

#### Scenario: WhiteElo column format
- **WHEN** a game has a WhiteElo header with value "2100"
- **THEN** the WhiteElo column contains the string "2100"

#### Scenario: BlackElo column format
- **WHEN** a game has a BlackElo header with value "2150"
- **THEN** the BlackElo column contains the string "2150"

#### Scenario: Missing ELO ratings
- **WHEN** a game lacks ELO rating headers
- **THEN** the WhiteElo and BlackElo columns contain NULL

### Requirement: Date and Time Information
The system SHALL provide columns for game timestamp with fallback support.

#### Scenario: UTC date column
- **WHEN** a game has a UTCDate header
- **THEN** the UTCDate column contains the date string

#### Scenario: Date fallback
- **WHEN** a game lacks UTCDate but has a Date header
- **THEN** the UTCDate column contains the Date header value

#### Scenario: UTC time column
- **WHEN** a game has a UTCTime header
- **THEN** the UTCTime column contains the time string

#### Scenario: Time fallback
- **WHEN** a game lacks UTCTime but has a Time header
- **THEN** the UTCTime column contains the Time header value

### Requirement: Opening Information
The system SHALL provide columns for chess opening classification.

#### Scenario: ECO code column
- **WHEN** a game has an ECO header
- **THEN** the ECO column contains the Encyclopedia of Chess Openings code (e.g., "C41", "D30")

#### Scenario: Opening name column
- **WHEN** a game has an Opening header
- **THEN** the Opening column contains the opening name (e.g., "Sicilian Defense", "Queen's Gambit Declined")

### Requirement: Game Details
The system SHALL provide columns for game metadata.

#### Scenario: Termination column
- **WHEN** a game has a Termination header
- **THEN** the Termination column contains the termination reason (Normal, Time forfeit, Abandoned, etc.)

#### Scenario: Time control column
- **WHEN** a game has a TimeControl header
- **THEN** the TimeControl column contains the time control format (e.g., "180+2", "60+0")

### Requirement: Movetext Column
The system SHALL provide a column containing the complete move sequence.

#### Scenario: Movetext with annotations
- **WHEN** a game includes moves with comments and annotations
- **THEN** the movetext column contains the full movetext including all annotations

#### Scenario: Movetext format
- **WHEN** movetext is stored
- **THEN** it uses Standard Algebraic Notation with move numbers (e.g., "1. e4 e5 2. Nf3 Nc6")

#### Scenario: Movetext always present
- **WHEN** a game is parsed successfully
- **THEN** the movetext column always contains a non-NULL value (empty string if no moves)

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

### Requirement: Column Data Types
The system SHALL use VARCHAR type for all columns to match Lichess format.

#### Scenario: All columns VARCHAR
- **WHEN** describing the table schema
- **THEN** all 17 columns have LogicalTypeId::Varchar type

#### Scenario: Nullable columns
- **WHEN** describing the table schema
- **THEN** all columns are marked as nullable (YES in DESCRIBE output)

### Requirement: NULL Value Handling
The system SHALL properly represent missing data using SQL NULL values instead of empty strings.

#### Scenario: NULL for missing headers
- **WHEN** a PGN game lacks a specific header
- **THEN** the corresponding column contains SQL NULL rather than an empty string

#### Scenario: Empty string vs NULL distinction
- **WHEN** a header is present but empty
- **THEN** the system distinguishes between empty string values and missing (NULL) values

#### Scenario: Vector validity masks
- **WHEN** outputting data to DuckDB
- **THEN** the system sets appropriate validity masks for NULL values in each column
