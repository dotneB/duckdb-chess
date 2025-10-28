# Data Schema Capability

## Purpose
To define the database schema for representing chess games, ensuring compatibility with the Lichess export format and standard SQL types.

## Requirements

### Requirement: Lichess Schema Compatibility
The system SHALL expose PGN data through a schema that matches the Lichess database export format for seamless integration.

#### Scenario: Schema column count
- **WHEN** querying the `read_pgn` table function
- **THEN** the result contains exactly 16 columns

#### Scenario: Column names match Lichess
- **WHEN** describing the table structure
- **THEN** column names match Lichess format: Event, Site, White, Black, Result, WhiteTitle, BlackTitle, WhiteElo, BlackElo, UTCDate, UTCTime, ECO, Opening, Termination, TimeControl, movetext

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

### Requirement: Column Data Types
The system SHALL use VARCHAR type for all columns to match Lichess format.

#### Scenario: All columns VARCHAR
- **WHEN** describing the table schema
- **THEN** all 16 columns have LogicalTypeId::Varchar type

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
