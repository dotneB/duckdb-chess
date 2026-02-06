# Data Schema Capability

## Purpose
To define the database schema for representing chess games, ensuring compatibility with the Lichess export format and standard SQL types.
## Requirements
### Requirement: Lichess Schema Compatibility
The system SHALL expose PGN data through a schema that extends the Lichess database export format with additional diagnostic columns.

#### Scenario: Schema column count
- **WHEN** querying the `read_pgn` table function
- **THEN** the result contains exactly 18 columns (16 Lichess columns + 2 diagnostic columns)

#### Scenario: Column names include parse_error
- **WHEN** describing the table structure
- **THEN** column names include all Lichess columns (Event, Site, White, Black, Result, WhiteTitle, BlackTitle, WhiteElo, BlackElo, UTCDate, UTCTime, ECO, Opening, Termination, TimeControl, movetext) plus the parse_error column and the Source column

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
The system SHALL provide columns for player ELO ratings stored as `UINTEGER`.

#### Scenario: WhiteElo column parsed
- **WHEN** a game has a `WhiteElo` header with value "2100"
- **THEN** the `WhiteElo` column contains the unsigned integer value `2100`

#### Scenario: BlackElo column parsed
- **WHEN** a game has a `BlackElo` header with value "2150"
- **THEN** the `BlackElo` column contains the unsigned integer value `2150`

#### Scenario: Missing ELO ratings
- **WHEN** a game lacks ELO rating headers
- **THEN** the `WhiteElo` and `BlackElo` columns contain SQL `NULL`

#### Scenario: Invalid ELO ratings
- **WHEN** a game has a `WhiteElo` or `BlackElo` header that is not a valid unsigned integer
- **THEN** the corresponding column contains SQL `NULL`
- **AND** the `parse_error` column contains a conversion error message for the field

### Requirement: Date and Time Information
The system SHALL provide typed columns for game timestamp with fallback support, including normalization and partial-date handling for common PGN/Lichess conventions.

#### Scenario: UTC date column
- **WHEN** a game has a `UTCDate` header with value "2024.01.01" or "2024-01-01"
- **THEN** the `UTCDate` column contains a `DATE` representing 2024-01-01

#### Scenario: Date fallback
- **WHEN** a game lacks `UTCDate` but has a `Date` header with value "2024.01.01" or "2024-01-01"
- **THEN** the `UTCDate` column contains a `DATE` representing 2024-01-01

#### Scenario: EventDate fallback
- **WHEN** a game lacks `UTCDate` and lacks `Date` but has an `EventDate` header with value "2024.01.01" or "2024-01-01"
- **THEN** the `UTCDate` column contains a `DATE` representing 2024-01-01

#### Scenario: Prefer first complete date in fallback chain
- **WHEN** a game has multiple date headers among `UTCDate`, `Date`, and `EventDate`
- **AND** at least one header provides a complete date (does not contain `?`) that can be parsed
- **THEN** the `UTCDate` column uses a complete date value
- **AND** if multiple parseable complete date values exist, it uses the one from the earliest header in the chain `UTCDate` -> `Date` -> `EventDate`

#### Scenario: Prefer most complete partial date
- **WHEN** a game has multiple date headers among `UTCDate`, `Date`, and `EventDate`
- **AND** no parseable header provides a complete date
- **AND** one parseable header provides `YYYY-MM-??` while another provides `YYYY-??-??`
- **THEN** the `UTCDate` column uses the `YYYY-MM-??` value and defaults the day to `01`

#### Scenario: Unknown date
- **WHEN** a game has `UTCDate`/`Date` with value "????.??.??" (or the equivalent "????-??-??")
- **THEN** the `UTCDate` column contains SQL `NULL`

#### Scenario: Partial date (year only)
- **WHEN** a game has `UTCDate`/`Date` with value "2000.??.??" (or the equivalent "2000-??-??")
- **THEN** the `UTCDate` column contains a `DATE` representing 2000-01-01

#### Scenario: Partial date (year and month)
- **WHEN** a game has `UTCDate`/`Date` with value "2000.06.??" (or the equivalent "2000-06-??")
- **THEN** the `UTCDate` column contains a `DATE` representing 2000-06-01

#### Scenario: UTC time column
- **WHEN** a game has a `UTCTime` header with value "12:00:00" or "12:00:00Z"
- **THEN** the `UTCTime` column contains a `TIMETZ` representing 12:00:00+00:00

#### Scenario: Time fallback
- **WHEN** a game lacks `UTCTime` but has a `Time` header with value "12:00:00" or "12:00:00Z"
- **THEN** the `UTCTime` column contains a `TIMETZ` representing 12:00:00+00:00

#### Scenario: Time with explicit offset
- **WHEN** a game has `UTCTime`/`Time` with value "12:00:00+01:30" or "12:00:00-05:00"
- **THEN** the `UTCTime` column contains a `TIMETZ` representing the provided local time with the provided offset

#### Scenario: Invalid primary date falls back to valid secondary date
- **WHEN** a game has an invalid non-empty `UTCDate` value
- **AND** a parseable fallback date exists in `Date` or `EventDate`
- **THEN** the `UTCDate` column contains the parseable fallback `DATE` value
- **AND** the `parse_error` column contains a conversion error message for the invalid `UTCDate` value

#### Scenario: Invalid primary time falls back to valid secondary time
- **WHEN** a game has an invalid non-empty `UTCTime` value
- **AND** a parseable fallback time exists in `Time`
- **THEN** the `UTCTime` column contains the parseable fallback `TIMETZ` value
- **AND** the `parse_error` column contains a conversion error message for the invalid `UTCTime` value

#### Scenario: Invalid date/time values with no parseable fallback
- **WHEN** a game has `UTCDate`/`Date`/`EventDate` and/or `UTCTime`/`Time` headers with non-empty values
- **AND** no candidate for the corresponding typed field can be parsed into `DATE`/`TIMETZ`
- **THEN** the corresponding typed columns contain SQL `NULL`
- **AND** the `parse_error` column contains conversion error message(s) for the failed field candidate(s)

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
The system SHALL provide a column containing the parsed mainline movetext in a PGN-compatible format.

#### Scenario: Movetext with annotations
- **WHEN** a game includes moves with comments and annotations
- **THEN** the movetext column contains the moves in SAN format
- **AND** comments are included in `{ ... }` form
- **AND** leading/trailing whitespace inside comments is normalized

#### Scenario: Movetext format
- **WHEN** movetext is stored
- **THEN** it uses Standard Algebraic Notation with move numbers (e.g., `1. e4 e5 2. Nf3 Nc6`)
- **AND** variations are not included (mainline only)

#### Scenario: Movetext always present
- **WHEN** a game is parsed successfully
- **THEN** the movetext column always contains a non-NULL value (empty string if no mainline moves)

#### Scenario: Result marker is not embedded in movetext
- **WHEN** a game has a result (from movetext outcome marker or `Result` header)
- **THEN** the movetext column does NOT append a terminal result marker token (`1-0`, `0-1`, `1/2-1/2`, or `*`)
- **AND** game result metadata is exposed through the `Result` column

### Requirement: Parse Error Column
The system SHALL provide a `parse_error` column containing diagnostic information about parsing failures and non-fatal field conversion failures.

#### Scenario: Parse error column presence
- **WHEN** querying the `read_pgn` table function
- **THEN** a `parse_error` column (VARCHAR, nullable) is included as the 17th column

#### Scenario: Successful game parsing and conversion
- **WHEN** a game parses successfully without PGN parsing errors
- **AND** typed conversions for `UTCDate`, `UTCTime`, `WhiteElo`, and `BlackElo` succeed or the corresponding source headers are missing/empty
- **THEN** the `parse_error` column contains NULL

#### Scenario: Parser-stage failure
- **WHEN** parser-stage game parsing fails while processing a row
- **THEN** the row is still output with available partial data
- **AND** the `parse_error` column is non-NULL for that row

#### Scenario: Parser-stage context in parse_error
- **WHEN** a parser-stage failure is captured in `parse_error`
- **THEN** the message includes parser stage and source file context
- **AND** it includes game index context when available

#### Scenario: Non-fatal conversion failure
- **WHEN** a game parses successfully but a non-empty value for `UTCDate`, `UTCTime`, `WhiteElo`, or `BlackElo` fails to convert to the target type
- **THEN** the row is still output
- **AND** the affected typed column(s) contain SQL `NULL`
- **AND** the `parse_error` column contains a descriptive conversion error message

#### Scenario: Multiple error messages
- **WHEN** a row has more than one conversion failure and/or an existing PGN parsing error
- **THEN** the `parse_error` column contains a single string containing all applicable messages

#### Scenario: Parser and conversion diagnostics combined
- **WHEN** a row has a parser-stage error and one or more typed conversion failures
- **THEN** the `parse_error` column contains both parser and conversion diagnostics in one combined message

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
The system SHALL provide clear, actionable error messages in the `parse_error` column that indicate the stage and nature of the failure.

#### Scenario: Error message for conversion failures
- **WHEN** a typed conversion fails for `UTCDate`, `UTCTime`, `WhiteElo`, or `BlackElo`
- **THEN** the `parse_error` message clearly indicates which field failed conversion and includes the original value
- **AND** for date/time parsing failures it includes the underlying parser error details (e.g., `chrono` parse error)

### Requirement: Column Data Types
The system SHALL use `VARCHAR` type for all `read_pgn` columns EXCEPT `WhiteElo`, `BlackElo`, `UTCDate`, and `UTCTime`, to match Lichess dataset schemas.

#### Scenario: Mixed column types
- **WHEN** describing the `read_pgn` table schema
- **THEN** `WhiteElo` and `BlackElo` have type `UINTEGER`
- **AND** `UTCDate` has type `DATE`
- **AND** `UTCTime` has type `TIMETZ`
- **AND** all other columns (including `parse_error`) have type `VARCHAR`

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

