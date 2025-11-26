# PGN Parsing Capability

## Purpose
To define the functionality for reading, parsing, and processing PGN (Portable Game Notation) files into structured queryable data.

## Requirements

### Requirement: PGN File Reading
The system SHALL provide a `read_pgn(path_pattern)` table function that parses PGN (Portable Game Notation) files and returns game data as SQL-queryable rows.

#### Scenario: Single file parsing
- **WHEN** user calls `read_pgn('path/to/file.pgn')` with a valid PGN file path
- **THEN** the function returns all games from that file with complete header and movetext data

#### Scenario: Glob pattern parsing
- **WHEN** user calls `read_pgn('path/*.pgn')` with a glob pattern
- **THEN** the function expands the pattern, reads all matching files, and returns combined game data from all files

#### Scenario: Empty result for non-existent files
- **WHEN** user calls `read_pgn('nonexistent.pgn')` with a path that doesn't exist
- **THEN** the function returns an error indicating the file could not be opened

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

### Requirement: Chunked Output
The system SHALL output parsed games in chunks to manage memory efficiently for large datasets.

#### Scenario: Large dataset processing
- **WHEN** parsing results in more than 2048 games
- **THEN** the function outputs games in chunks of up to 2048 rows per call

#### Scenario: Small dataset processing
- **WHEN** parsing results in fewer than 2048 games
- **THEN** the function outputs all games in a single chunk

### Requirement: Game Boundary Detection
The system SHALL correctly identify game boundaries in PGN files using the `[Event` header as a delimiter.

#### Scenario: Multiple games in single file
- **WHEN** a PGN file contains multiple games separated by `[Event ` headers
- **THEN** each game is parsed as a separate record

#### Scenario: Last game in file
- **WHEN** reaching the end of a file
- **THEN** the function parses the final accumulated game before closing the file

### Requirement: Thread Safety
The system SHALL ensure thread-safe access to shared parsing state across multiple table function calls.

#### Scenario: Atomic state management
- **WHEN** multiple DuckDB threads access the table function
- **THEN** atomic flags and mutexes protect shared game data and offset tracking

### Requirement: Visitor Pattern Implementation
The system SHALL use the pgn-reader library's Visitor trait for streaming PGN parsing.

#### Scenario: Header tag collection
- **WHEN** the visitor encounters PGN header tags
- **THEN** all key-value pairs are collected into the headers vector

#### Scenario: Movetext accumulation
- **WHEN** the visitor encounters chess moves
- **THEN** moves are formatted with move numbers and accumulated into the movetext buffer

#### Scenario: Comment preservation
- **WHEN** the visitor encounters comments in curly braces
- **THEN** comments are preserved in the movetext output with proper formatting

#### Scenario: Variation skipping
- **WHEN** the visitor encounters move variations (alternate move sequences)
- **THEN** variations are skipped to maintain the main game line

### Requirement: UTF-8 Handling
The reader MUST handle PGN files containing invalid UTF-8 sequences without failing or skipping lines.

#### Scenario: Invalid UTF-8 bytes in header
- **WHEN** `read_pgn` reads a PGN file containing a byte sequence invalid in UTF-8 (e.g., `0x90` in a name like "Djukin")
- **THEN** the invalid bytes are replaced with the Unicode replacement character
- **AND** the line is processed normally by the PGN parser
- **AND** the game data is extracted successfully (with the replacement character in the string)

#### Scenario: Valid UTF-8 content
- **WHEN** `read_pgn` reads a PGN file with valid UTF-8 content
- **THEN** the content is preserved exactly as is
- **AND** no replacement characters are introduced
