# PGN Parsing Capability

## Purpose
To define the functionality for reading, parsing, and processing PGN (Portable Game Notation) files into structured queryable data, including error handling and performance characteristics.
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

### Requirement: Thread Safety
The system SHALL ensure thread-safe access to shared parsing state across multiple table function calls.

#### Scenario: Atomic state management
- **WHEN** multiple DuckDB threads access the table function
- **THEN** atomic flags and mutexes protect shared game data and offset tracking

### Requirement: Visitor Pattern Implementation
The system SHALL use the pgn-reader library's `Reader` and `Visitor` trait for streaming PGN parsing, calling `read_game()` directly on a `Reader<File>` instance without intermediate buffering layers.

#### Scenario: Direct streaming from file
- **WHEN** the table function opens a PGN file for parsing
- **THEN** it creates a `Reader<File>` instance directly from the file handle
- **AND** it does NOT wrap the file in a `BufReader` (pgn-reader handles buffering internally)

#### Scenario: Game parsing via read_game
- **WHEN** the table function needs to parse the next game
- **THEN** it calls `read_game(&mut visitor)` on the `Reader<File>` instance
- **AND** the visitor's methods (`begin_tags`, `tag`, `begin_movetext`, `san`, `outcome`, `comment`, `end_game`) are invoked by the reader

#### Scenario: Header tag collection
- **WHEN** the visitor encounters PGN header tags
- **THEN** all key-value pairs are collected into the headers vector

#### Scenario: Movetext accumulation
- **WHEN** the visitor encounters chess moves
- **THEN** moves are formatted with move numbers and accumulated into the movetext buffer
- **AND** variations are skipped to maintain the main line

#### Scenario: Comment handling
- **WHEN** the visitor encounters comments in curly braces
- **THEN** comments are included in the movetext output in `{ ... }` form
- **AND** leading/trailing whitespace inside comments is normalized

#### Scenario: Result marker capture
- **WHEN** the visitor's `outcome()` method is called with an Outcome enum
- **THEN** it converts the Outcome to string representation ("1-0", "0-1", "1/2-1/2", or "*")
- **AND** it stores the result marker for inclusion in movetext output
- **AND** the result marker is appended to the movetext string after all moves

#### Scenario: EOF detection
- **WHEN** `read_game()` is called and the file has been fully consumed
- **THEN** it returns `Ok(None)` indicating EOF
- **AND** the parser moves to the next file or completes

#### Scenario: Error during parsing
- **WHEN** `read_game()` encounters a parsing error (e.g., malformed tag, unterminated comment)
- **THEN** it returns `Err(e)` with a descriptive error message
- **AND** the table function calls `visitor.finalize_game_with_error()` to capture partial game data

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

### Requirement: Streaming & Parallel Execution
The PGN reader MUST use a streaming architecture that supports parallel processing across multiple files to maximize throughput on multi-core systems while maintaining constant memory usage.

#### Scenario: Multi-file parallel processing
- **WHEN** a glob pattern matches multiple PGN files (e.g., `data/*.pgn`)
- **AND** the system has multiple CPU cores available
- **THEN** multiple files are processed concurrently
- **AND** the memory usage does not scale linearly with the dataset size

#### Scenario: Large dataset processing
- **WHEN** a glob pattern matches files larger than available RAM
- **THEN** the query completes successfully without Out-Of-Memory errors
- **AND** the system reads files sequentially or concurrently in chunks


