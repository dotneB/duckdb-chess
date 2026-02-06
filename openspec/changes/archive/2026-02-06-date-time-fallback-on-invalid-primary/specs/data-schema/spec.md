## MODIFIED Requirements

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

### Requirement: Parse Error Column
The system SHALL provide a `parse_error` column containing diagnostic information about parsing failures and non-fatal field conversion failures.

#### Scenario: Parse error column presence
- **WHEN** querying the `read_pgn` table function
- **THEN** a `parse_error` column (VARCHAR, nullable) is included as the 17th column

#### Scenario: Successful game parsing and conversion
- **WHEN** a game parses successfully without PGN parsing errors
- **AND** typed conversions for `UTCDate`, `UTCTime`, `WhiteElo`, and `BlackElo` succeed or the corresponding source headers are missing/empty
- **THEN** the `parse_error` column contains `NULL`

#### Scenario: Non-fatal conversion failure
- **WHEN** a game parses successfully but a non-empty value for `UTCDate`, `UTCTime`, `WhiteElo`, or `BlackElo` fails to convert to the target type
- **THEN** the row is still output
- **AND** the `parse_error` column contains a descriptive conversion error message
- **AND** the affected typed column contains SQL `NULL` only when no fallback candidate for that typed field can be converted

#### Scenario: Conversion failure with successful date/time fallback
- **WHEN** a higher-priority non-empty `UTCDate` or `UTCTime` candidate fails conversion
- **AND** a lower-priority fallback candidate for the same typed field converts successfully
- **THEN** the typed field contains the fallback converted value
- **AND** the `parse_error` column still contains the conversion error message for the failed higher-priority candidate

#### Scenario: Multiple error messages
- **WHEN** a row has more than one conversion failure and/or an existing PGN parsing error
- **THEN** the `parse_error` column contains a single string containing all applicable messages
