## MODIFIED Requirements

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
