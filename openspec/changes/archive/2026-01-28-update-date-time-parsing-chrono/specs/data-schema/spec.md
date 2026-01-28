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
 - **AND** at least one header provides a complete date (does not contain `?`)
 - **THEN** the `UTCDate` column uses a complete date value
 - **AND** if multiple complete date values exist, it uses the one from the earliest header in the chain `UTCDate` -> `Date` -> `EventDate`

#### Scenario: Prefer most complete partial date
- **WHEN** a game has multiple date headers among `UTCDate`, `Date`, and `EventDate`
- **AND** no header provides a complete date
- **AND** one header provides `YYYY-MM-??` while another provides `YYYY-??-??`
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

#### Scenario: Invalid date/time values
- **WHEN** a game has `UTCDate`/`Date` or `UTCTime`/`Time` headers with non-empty values that cannot be parsed into `DATE`/`TIMETZ`
- **THEN** the corresponding columns contain SQL `NULL`
- **AND** the `parse_error` column contains a conversion error message for the field(s)

### Requirement: Error Message Format
The system SHALL provide clear, actionable error messages in the `parse_error` column that indicate the stage and nature of the failure.

#### Scenario: Error message for conversion failures
- **WHEN** a typed conversion fails for `UTCDate`, `UTCTime`, `WhiteElo`, or `BlackElo`
- **THEN** the `parse_error` message clearly indicates which field failed conversion and includes the original value
- **AND** for date/time parsing failures it includes the underlying parser error details (e.g., `chrono` parse error)
