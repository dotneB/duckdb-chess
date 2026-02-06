## MODIFIED Requirements

### Requirement: Visitor Pattern Implementation
The system SHALL use the pgn-reader library's `Reader` and `Visitor` trait for streaming PGN parsing, calling `read_game()` directly on a `Reader<File>` instance without intermediate buffering layers.

#### Scenario: Direct streaming from file
- **WHEN** the table function opens a PGN file for parsing
- **THEN** it creates a `Reader<File>` instance directly from the file handle
- **AND** it does NOT wrap the file in a `BufReader` (pgn-reader handles buffering internally)

#### Scenario: Game parsing via read_game
- **WHEN** the table function needs to parse the next game
- **THEN** it calls `read_game(&mut visitor)` on the `Reader<File>` instance
- **AND** the visitor methods (`begin_tags`, `tag`, `begin_movetext`, `san`, `outcome`, `comment`, `end_game`) are invoked by the reader

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
- **THEN** it converts the Outcome to string representation (`1-0`, `0-1`, `1/2-1/2`, or `*`)
- **AND** it stores the result marker for game-level result metadata
- **AND** the result marker is NOT appended to the movetext output string

#### Scenario: EOF detection
- **WHEN** `read_game()` is called and the file has been fully consumed
- **THEN** it returns `Ok(None)` indicating EOF
- **AND** the parser moves to the next file or completes

#### Scenario: Error during parsing
- **WHEN** `read_game()` encounters a parsing error (e.g., malformed tag, unterminated comment)
- **THEN** it returns `Err(e)` with a descriptive error message
- **AND** the table function calls `visitor.finalize_game_with_error()` to capture partial game data
