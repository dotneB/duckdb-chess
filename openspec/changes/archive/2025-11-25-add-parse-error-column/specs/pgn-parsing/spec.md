# PGN Parsing Capability Delta

## MODIFIED Requirements

### Requirement: Malformed Game Handling
The system SHALL handle malformed or invalid PGN games gracefully by outputting partial game data with error information instead of skipping them entirely.

#### Scenario: Capture malformed games
- **WHEN** a PGN file contains games with parsing errors
- **THEN** the function outputs the game with available header data and an error message in the parse_error column, and logs a warning message for backward compatibility

#### Scenario: Multiple games with mixed validity
- **WHEN** a file contains 10 games where 2 are malformed
- **THEN** the function returns all 10 game records, where 8 have NULL parse_error and 2 have error messages in parse_error

#### Scenario: Partial data recovery
- **WHEN** a game fails to parse movetext but headers are valid
- **THEN** the function outputs the game with all header fields populated, an empty or partial movetext, and the parsing error in parse_error column

## ADDED Requirements

### Requirement: Error Message Capture
The system SHALL capture parsing error details and include them in the output for diagnostic purposes.

#### Scenario: Movetext parsing error captured
- **WHEN** movetext parsing fails for a game
- **THEN** the error message is stored in the game's parse_error field and included in the output

#### Scenario: Error context preservation
- **WHEN** a parsing error occurs
- **THEN** the error message includes sufficient context (e.g., "Error parsing game #5: invalid move notation") to identify the problematic game

#### Scenario: Error logging continues
- **WHEN** a parsing error occurs
- **THEN** the error is both logged to stderr (for backward compatibility) and captured in the parse_error column

### Requirement: Graceful Degradation
The system SHALL maximize data recovery by outputting whatever valid information was successfully parsed before an error occurred, regardless of which parsing stage failed.

#### Scenario: Headers valid, movetext invalid
- **WHEN** all headers parse successfully but movetext parsing fails
- **THEN** the game is output with all header fields populated and parse_error containing the movetext error

#### Scenario: Partial header parsing
- **WHEN** some headers parse successfully before a header parsing error occurs
- **THEN** the successfully parsed headers are included in the output with parse_error indicating which header or stage caused the failure

#### Scenario: Header parsing error with minimal data
- **WHEN** header parsing fails early (e.g., malformed Event header)
- **THEN** the game is output with whatever minimal data could be extracted and parse_error describing the header parsing failure

#### Scenario: Continue after error
- **WHEN** one game fails to parse at any stage
- **THEN** parsing continues with the next game, and both the failed and subsequent successful games are included in output

#### Scenario: Multiple error types
- **WHEN** a file contains games with different types of parsing errors (header errors, movetext errors)
- **THEN** each game is output with its specific error type indicated in the parse_error column
