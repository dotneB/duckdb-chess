## MODIFIED Requirements

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
The system SHALL provide typed columns for game timestamp with fallback support.

#### Scenario: UTC date column
- **WHEN** a game has a `UTCDate` header with value "2024.01.01"
- **THEN** the `UTCDate` column contains a `DATE` representing 2024-01-01

#### Scenario: Date fallback
- **WHEN** a game lacks `UTCDate` but has a `Date` header with value "2024.01.01"
- **THEN** the `UTCDate` column contains a `DATE` representing 2024-01-01

#### Scenario: UTC time column
- **WHEN** a game has a `UTCTime` header with value "12:00:00"
- **THEN** the `UTCTime` column contains a `TIMETZ` representing 12:00:00+00:00

#### Scenario: Time fallback
- **WHEN** a game lacks `UTCTime` but has a `Time` header with value "12:00:00"
- **THEN** the `UTCTime` column contains a `TIMETZ` representing 12:00:00+00:00

#### Scenario: Invalid or unknown date/time values
- **WHEN** a game has `UTCDate`/`Date` or `UTCTime`/`Time` headers with values that cannot be parsed into `DATE`/`TIMETZ`
- **THEN** the corresponding columns contain SQL `NULL`
- **AND** the `parse_error` column contains a conversion error message for the field(s)

### Requirement: Parse Error Column
The system SHALL provide a `parse_error` column containing diagnostic information about parsing failures and non-fatal field conversion failures.

#### Scenario: Parse error column presence
- **WHEN** querying the `read_pgn` table function
- **THEN** a `parse_error` column (VARCHAR, nullable) is included as the 17th column

#### Scenario: Successful game parsing and conversion
- **WHEN** a game parses successfully without PGN parsing errors
- **AND** typed conversions for `UTCDate`, `UTCTime`, `WhiteElo`, and `BlackElo` succeed or the corresponding source headers are missing/empty
- **THEN** the `parse_error` column contains NULL

#### Scenario: Non-fatal conversion failure
- **WHEN** a game parses successfully but a non-empty value for `UTCDate`, `UTCTime`, `WhiteElo`, or `BlackElo` fails to convert to the target type
- **THEN** the row is still output
- **AND** the affected typed column(s) contain SQL `NULL`
- **AND** the `parse_error` column contains a descriptive conversion error message

#### Scenario: Multiple error messages
- **WHEN** a row has more than one conversion failure and/or an existing PGN parsing error
- **THEN** the `parse_error` column contains a single string containing all applicable messages

### Requirement: Error Message Format
The system SHALL provide clear, actionable error messages in the `parse_error` column that indicate the stage and nature of the failure.

#### Scenario: Error message for conversion failures
- **WHEN** a typed conversion fails for `UTCDate`, `UTCTime`, `WhiteElo`, or `BlackElo`
- **THEN** the `parse_error` message clearly indicates which field failed conversion and includes the original value

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
