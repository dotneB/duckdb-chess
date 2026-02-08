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

#### Scenario: Out-of-range day is clamped to month end
- **WHEN** a game has `UTCDate`/`Date`/`EventDate` with valid year and month but day greater than the last day of that month (for example `2015.11.31` or `1997.02.29`)
- **THEN** the candidate date is normalized to the last valid day of that month (for example `2015-11-30` or `1997-02-28`)
- **AND** the normalized value is eligible for normal header precedence and fallback selection

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
- **WHEN** a game has an invalid non-empty `UTCDate` value that cannot be parsed after day-overflow normalization
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
- **AND** no candidate for the corresponding typed field can be parsed into `DATE`/`TIMETZ` after day-overflow normalization
- **THEN** the corresponding typed columns contain SQL `NULL`
- **AND** the `parse_error` column contains conversion error message(s) for the failed field candidate(s)
