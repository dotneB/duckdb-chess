## MODIFIED Requirements

### Requirement: Malformed Game Handling
The system SHALL handle malformed or invalid PGN games gracefully by outputting partial game data with error information instead of skipping them entirely.

#### Scenario: Capture malformed games
- **WHEN** a PGN file contains games with parsing errors
- **THEN** the function outputs the game with available header data and an error message in the `parse_error` column
- **AND** parser-stage failures are observable as non-NULL `parse_error` values in SQL output

#### Scenario: Multiple games with mixed validity
- **WHEN** a file contains 10 games where 2 are malformed
- **THEN** the function returns all 10 game records, where 8 have NULL `parse_error` and 2 have error messages in `parse_error`

#### Scenario: Partial data recovery
- **WHEN** a game fails to parse movetext but headers are valid
- **THEN** the function outputs the game with all header fields populated, an empty or partial movetext, and the parsing error in `parse_error`

#### Scenario: Continue after parser-stage failure
- **WHEN** a parser-stage error occurs for one game
- **THEN** parsing continues with subsequent games/files according to existing continuation behavior
- **AND** the errored game is still represented with partial data where available

### Requirement: Error Message Capture
The system SHALL capture parsing error details and include them in the output for diagnostic purposes.

#### Scenario: Movetext parsing error captured
- **WHEN** movetext parsing fails for a game
- **THEN** the error message is stored in the game's `parse_error` field and included in the output

#### Scenario: Error context preservation
- **WHEN** a parsing error occurs
- **THEN** the error message includes sufficient context to identify the problematic game
- **AND** context includes parser stage and source file path
- **AND** context includes game index when available

#### Scenario: Error logging continues
- **WHEN** a parsing error occurs
- **THEN** the error is logged to stderr for backward compatibility
- **AND** the same parsing failure is captured in the `parse_error` column

#### Scenario: Deterministic parser-stage failure observability
- **WHEN** test fixtures intentionally trigger `read_game()` parser-stage errors
- **THEN** SQLLogicTests can assert `parse_error IS NOT NULL` for those rows
