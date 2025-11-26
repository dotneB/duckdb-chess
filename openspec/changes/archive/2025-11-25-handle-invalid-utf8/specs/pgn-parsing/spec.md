## ADDED Requirements

### Requirement: UTF-8 Handling
The reader MUST handle PGN files containing invalid UTF-8 sequences without failing or skipping lines.

#### Scenario: Invalid UTF-8 bytes in header
Given a PGN file containing a byte sequence invalid in UTF-8 (e.g., `0x90` in a name like "Djukin")
When `read_pgn` reads the file
Then the invalid bytes are replaced with the Unicode replacement character ()
And the line is processed normally by the PGN parser
And the game data is extracted successfully (with the replacement character in the string)

#### Scenario: Valid UTF-8 content
Given a PGN file with valid UTF-8 content
When `read_pgn` reads the file
Then the content is preserved exactly as is
And no replacement characters are introduced
