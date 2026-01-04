## MODIFIED Requirements

### Requirement: Movetext Column
The system SHALL provide a column containing the complete move sequence in valid PGN format, including move numbers, moves in Standard Algebraic Notation, comments, and the game result marker.

#### Scenario: Movetext with annotations
- **WHEN** a game includes moves with comments and annotations
- **THEN** the movetext column contains the moves in SAN format
- **AND** all comments are preserved exactly as they appear in the original PGN
- **AND** comment formatting (braces, whitespace, annotations) is maintained

#### Scenario: Movetext format
- **WHEN** movetext is stored
- **THEN** it uses Standard Algebraic Notation with move numbers (e.g., "1. e4 e5 2. Nf3 Nc6")
- **AND** it includes the result marker at the end (e.g., "1. e4 e5 1-0")

#### Scenario: Movetext always present
- **WHEN** a game is parsed successfully
- **THEN** the movetext column always contains a non-NULL value (empty string if no moves)

#### Scenario: Result marker in movetext
- **WHEN** a game has a result (from Result header or movetext)
- **THEN** the movetext includes the result marker as the final token (1-0, 0-1, 1/2-1/2, or *)
- **AND** the movetext is valid PGN format that can be parsed by standard chess tools

#### Scenario: Movetext exactly matches original PGN
- **WHEN** a valid game is parsed successfully
- **THEN** the movetext output is identical to the original PGN movetext
- **AND** all comments appear at their exact original positions
- **AND** the formatting of comments (including annotations like [%eval], [%clk]) is preserved exactly
- **AND** the result marker appears in the same position as the original

#### Scenario: Incomplete game movetext
- **WHEN** a game has no result information
- **THEN** the movetext contains only the moves without a result marker
- **AND** this represents an incomplete or ongoing game

### Requirement: Error Message Format
The system SHALL provide clear, actionable error messages in the parse_error column that indicate the parsing stage, nature of the failure, and specific move or position context.

#### Scenario: Error message includes context
- **WHEN** a parsing error occurs
- **THEN** the parse_error message includes relevant context such as game number, file location, and error description

#### Scenario: Error message for movetext failures
- **WHEN** movetext parsing fails
- **THEN** the parse_error message clearly indicates that movetext parsing failed and the nature of the failure

#### Scenario: Error message for illegal moves
- **WHEN** a move validation error occurs
- **THEN** the parse_error message includes:
  - The specific move that was illegal (e.g., "Illegal move 'Nf7'")
  - The ply number at which the error occurred (e.g., "at ply 5")
  - The position FEN for debugging context (e.g., "from position rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq -")

#### Scenario: Error message for header failures
- **WHEN** header parsing fails
- **THEN** the parse_error message clearly indicates that header parsing failed and which header or what issue occurred

#### Scenario: Error stage identification
- **WHEN** any parsing error occurs
- **THEN** the parse_error message allows users to distinguish between header parsing errors, movetext parsing errors, move validation errors, and file reading errors
