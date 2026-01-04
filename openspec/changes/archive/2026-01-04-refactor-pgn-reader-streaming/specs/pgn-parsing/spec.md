# PGN Parsing Spec Delta

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
- **AND** the visitor's methods (`begin_tags`, `tag`, `begin_movetext`, `san`, `comment`, `end_game`) are invoked by the reader

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

#### Scenario: EOF detection
- **WHEN** `read_game()` is called and the file has been fully consumed
- **THEN** it returns `Ok(None)` indicating EOF
- **AND** the parser moves to the next file or completes

#### Scenario: Error during parsing
- **WHEN** `read_game()` encounters a parsing error (e.g., malformed tag, unterminated comment)
- **THEN** it returns `Err(e)` with a descriptive error message
- **AND** the table function calls `visitor.finalize_game_with_error()` to capture partial game data

## REMOVED Requirements

### Requirement: Game Boundary Detection
**Reason**: The pgn-reader library's `read_game()` method handles game boundary detection internally through the Visitor pattern. Manual detection by searching for `[Event ` headers is redundant and error-prone.

**Migration**: Code relying on manual `[Event ` detection should be removed. The `read_game()` method correctly identifies game boundaries and calls `begin_tags()` at the start of each new game.

**Implementation Note**: The removal of manual boundary detection eliminates:
- Line-by-line reading with `read_until(b'\n')`
- `game_buffer: String` accumulation
- `line_buffer: Vec<u8>` temporary storage
- Manual detection of `[Event ` prefix
- UTF-8 lossy conversion at the line level
