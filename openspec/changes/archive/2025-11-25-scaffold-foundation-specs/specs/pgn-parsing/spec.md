# PGN Parsing Capability

## ADDED Requirements

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
The system SHALL handle malformed or invalid PGN games gracefully without failing the entire batch.

#### Scenario: Skip malformed games
- **WHEN** a PGN file contains games with parsing errors
- **THEN** the function logs a warning message for each malformed game and continues parsing remaining games

#### Scenario: Multiple games with mixed validity
- **WHEN** a file contains 10 games where 2 are malformed
- **THEN** the function returns 8 valid game records and logs 2 warnings

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
