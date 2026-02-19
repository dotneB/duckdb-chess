## MODIFIED Requirements

### Requirement: PGN File Reading
The system SHALL provide a `read_pgn(path_pattern, compression := NULL)` table function that parses PGN (Portable Game Notation) files and returns game data as SQL-queryable rows.

#### Scenario: Single file parsing
- **WHEN** user calls `read_pgn('path/to/file.pgn')` with a valid PGN file path
- **THEN** the function returns all games from that file with complete header and movetext data

#### Scenario: Glob pattern parsing
- **WHEN** user calls `read_pgn('path/*.pgn')` with a glob pattern
- **THEN** the function expands the pattern, reads all matching files, and returns combined game data from all files

#### Scenario: Empty result for non-existent files
- **WHEN** user calls `read_pgn('nonexistent.pgn')` with a path that doesn't exist
- **THEN** the function returns an error indicating the file could not be opened

#### Scenario: Explicit NULL compression uses plain input mode
- **WHEN** user calls `read_pgn('path/to/file.pgn', compression := NULL)`
- **THEN** the function behaves the same as when `compression` is omitted
- **AND** it reads the file as plain PGN input

#### Scenario: Zstd-compressed single file parsing
- **WHEN** user calls `read_pgn('path/to/file.pgn.zst', compression := 'zstd')` with a valid zstd-compressed PGN file
- **THEN** the function decompresses the file as a stream and parses PGN games
- **AND** it returns all games from that file with complete header and movetext data

#### Scenario: Zstd-compressed glob parsing
- **WHEN** user calls `read_pgn('path/*.pgn.zst', compression := 'zstd')` with a glob pattern
- **THEN** the function expands the pattern, streams decompression for each matching file, and returns combined game data from all files

#### Scenario: Unsupported compression value
- **WHEN** user calls `read_pgn('path/to/file.pgn', compression := 'gzip')`
- **THEN** the function returns an error indicating `compression` is unsupported
- **AND** the error message lists supported values (`zstd`) and NULL/omitted for plain input
