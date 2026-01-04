## MODIFIED Requirements

### Requirement: Visitor Pattern Implementation
The system SHALL use the pgn-reader library's `Reader` and `Visitor` trait for streaming PGN parsing, maintaining a `shakmaty::Chess` position object to track game state and validate moves during parsing.

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

#### Scenario: Position initialization
- **WHEN** the visitor begins processing movetext
- **THEN** it initializes a `shakmaty::Chess` position to the standard starting position
- **AND** it clears the moves vector to store validated moves

#### Scenario: Move validation and tracking
- **WHEN** the visitor encounters a chess move in SAN notation
- **THEN** it attempts to parse and validate the move against the current position
- **AND** if valid, it applies the move to the position and adds the SanPlus to the moves vector
- **AND** if invalid, it captures the error with context (move text, ply number, position FEN)

#### Scenario: Movetext generation
- **WHEN** the visitor completes parsing a game
- **THEN** it generates movetext by iterating through the validated moves vector
- **AND** it formats moves with proper move numbers (1., 2., etc. for white moves)
- **AND** black moves omit move numbers (e.g., "1. e4 e5" not "1. e4 1... e5")
- **AND** it includes the result marker (1-0, 0-1, 1/2-1/2, or *) at the end if present
- **AND** movetext uses standard PGN format compatible with chess tools

#### Scenario: Comment handling
- **WHEN** the visitor encounters comments in curly braces
- **THEN** comments are parsed and stored with their ply position
- **AND** comments are preserved exactly as they appear in the original PGN (including braces and whitespace)
- **AND** comments are included in the generated movetext at their original positions

#### Scenario: Variation skipping
- **WHEN** the visitor encounters move variations (alternate move sequences)
- **THEN** variations are skipped to maintain the main game line
- **AND** the position tracking continues with the main line only

#### Scenario: Result marker capture
- **WHEN** the visitor's `outcome()` method is called with an Outcome enum
- **THEN** it converts the Outcome to string representation ("1-0", "0-1", "1/2-1/2", or "*")
- **AND** it stores the result marker for inclusion in the generated movetext
- **AND** the result marker is appended to the movetext string after all moves

#### Scenario: EOF detection
- **WHEN** `read_game()` is called and the file has been fully consumed
- **THEN** it returns `Ok(None)` indicating EOF
- **AND** the parser moves to the next file or completes

#### Scenario: Illegal move detection
- **WHEN** `read_game()` encounters a move that is illegal for the current position
- **THEN** the visitor captures an error message indicating which move failed and at which ply
- **AND** it includes the position FEN for debugging context
- **AND** the table function calls `visitor.finalize_game_with_error()` with the detailed error

#### Scenario: Error during parsing
- **WHEN** `read_game()` encounters a parsing error (e.g., malformed tag, unterminated comment)
- **THEN** it returns `Err(e)` with a descriptive error message
- **AND** the table function calls `visitor.finalize_game_with_error()` to capture partial game data

## ADDED Requirements

### Requirement: Move Validation During Parsing
The visitor SHALL validate each move against the current chess position as moves are encountered, providing immediate error detection and detailed error context.

#### Scenario: Valid move sequence
- **WHEN** parsing a game with all legal moves
- **THEN** each move is successfully validated and applied to the position
- **AND** the final movetext matches the original game notation with result marker included

#### Scenario: Invalid move in sequence
- **WHEN** parsing encounters an illegal move (e.g., "Nf7" when knight can't reach f7)
- **THEN** the visitor captures the error with specific details: "Illegal move 'Nf7' at ply 5 from position [FEN]"
- **AND** the game is output with all moves up to the error point
- **AND** the parse_error column contains the detailed validation error

#### Scenario: Move validation error context
- **WHEN** a move validation error occurs
- **THEN** the error message includes:
  - The exact move text that failed (e.g., "Nf7")
  - The ply number where the error occurred (e.g., "ply 5")
  - The position FEN at the time of the error
  - A human-readable explanation if available

### Requirement: Result Marker Preservation
The visitor SHALL preserve game result markers (1-0, 0-1, 1/2-1/2, *) in the generated movetext to produce valid PGN output.

#### Scenario: Result marker in movetext
- **WHEN** a game ends with a result marker in the movetext
- **THEN** the generated movetext includes the result marker as the final token
- **AND** the result marker is separated from the last move by a space

#### Scenario: Result marker from header only
- **WHEN** a game has a Result header but no result marker in movetext
- **THEN** the generated movetext includes the result marker from the Result header
- **AND** the movetext is valid PGN format

#### Scenario: Movetext without result marker
- **WHEN** a game has neither a movetext result marker nor a Result header
- **THEN** the generated movetext contains only the moves without a trailing result marker
- **AND** the movetext is still valid (incomplete game scenario)

### Requirement: Comment Preservation
The visitor SHALL preserve all comments from the original PGN exactly as they appear, maintaining their position relative to moves and their original formatting.

#### Scenario: Comment after move
- **WHEN** a comment appears after a move (e.g., "1. e4 { best move } e5")
- **THEN** the comment is stored with the ply number of that move
- **AND** the generated movetext includes the comment at the same position
- **AND** the comment formatting (braces, whitespace) is preserved exactly

#### Scenario: Multiple comments in game
- **WHEN** a game contains multiple comments at different positions
- **THEN** all comments are stored with their respective ply numbers
- **AND** the generated movetext includes all comments in their original order
- **AND** the final movetext exactly matches the original PGN comment structure

#### Scenario: Comments with annotations
- **WHEN** comments contain chess annotations like [%eval 0.25] or [%clk 1:30:00]
- **THEN** these annotations are preserved exactly as written
- **AND** the generated movetext maintains the exact annotation format

#### Scenario: Comment before first move
- **WHEN** a comment appears before the first move (ply 0)
- **THEN** the comment is stored with ply 0
- **AND** the generated movetext includes the comment before the first move

#### Scenario: Empty game with comments
- **WHEN** a game has no moves but contains comments
- **THEN** the comments are preserved in the movetext
- **AND** the result marker (if present) appears after the comments
