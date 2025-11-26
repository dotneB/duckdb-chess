## MODIFIED Requirements

### Requirement: Move Extraction to JSON
The system SHALL provide a scalar function `chess_moves_json(movetext)` that parses PGN movetext and returns a JSON array string containing details for every move.

#### Scenario: Explode valid game
- **WHEN** user calls `chess_moves_json('1. e4 e5')`
- **THEN** the function returns a JSON string:
  ```json
  [
    {"ply": 1, "move": "e4", "fen": "rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 1"},
    {"ply": 2, "move": "e5", "fen": "rnbqkbnr/pppp1ppp/8/4p3/4P3/8/PPPP1PPP/RNBQKBNR w KQkq e6 0 2"}
  ]
  ```

#### Scenario: SQL Integration
- **WHEN** used with DuckDB's JSON functions
- **THEN** the output can be unnested to rows:
  ```sql
  SELECT UNNEST(chess_moves_json('1. e4')::JSON);
  -- Result: {"ply": 1, "move": "e4", "fen": "..."}
  ```

#### Scenario: Empty or Null input
- **WHEN** user calls `chess_moves_json('')` or `chess_moves_json(NULL)`
- **THEN** the function returns an empty JSON array `'[]'`

#### Scenario: Partial Validity
- **WHEN** the movetext contains an illegal move (e.g., `'1. e4 e5 2. Kxe8'`)
- **THEN** the function returns the JSON array for all valid moves processed up to the error (e.g., just the first two moves).

#### Scenario: Annotation Filtering
- **WHEN** the movetext contains comments/annotations (e.g., `'1. e4 {Best by test} e5'`)
- **THEN** the annotations are ignored, and the moves are parsed correctly.

## ADDED Requirements

### Requirement: Moves Normalization
The system SHALL provide a scalar function `chess_moves_normalize(movetext)` that returns a canonical string representation of the move sequence, removing comments, recursive variations, and NAGs, while standardizing spacing.

#### Scenario: Normalize complex movetext
- **WHEN** user calls `chess_moves_normalize('1. e4 {comment} (1. d4) e5?!')`
- **THEN** the function returns `'1. e4 e5'` (or similar standard format, ensuring clean SAN moves).

#### Scenario: Handle empty input
- **WHEN** user calls `chess_moves_normalize(NULL)`
- **THEN** the function returns `NULL` or empty string (design choice: NULL -> NULL).

### Requirement: Moves Hashing
The system SHALL provide a scalar function `chess_moves_hash(movetext)` that returns a consistent hash value (e.g., 64-bit integer or hex string) representing the sequence of moves, independent of formatting or annotations.

#### Scenario: Hash consistency
- **WHEN** user calls `chess_moves_hash` on two games with identical moves but different comments/formatting
- **THEN** the returned hash values are identical.

#### Scenario: Hash discrimination
- **WHEN** user calls `chess_moves_hash` on two games with different moves
- **THEN** the returned hash values are different (with high probability).

### Requirement: Subsumption Detection
The system SHALL provide a scalar function `chess_moves_subset(short_movetext, long_movetext)` that returns true if the moves in `short_movetext` are an exact prefix of the moves in `long_movetext`.

#### Scenario: Exact subset
- **WHEN** user calls `chess_moves_subset('1. e4', '1. e4 e5')`
- **THEN** the function returns `TRUE`.

#### Scenario: Not a subset
- **WHEN** user calls `chess_moves_subset('1. d4', '1. e4 e5')`
- **THEN** the function returns `FALSE`.

#### Scenario: Same game
- **WHEN** user calls `chess_moves_subset('1. e4', '1. e4')`
- **THEN** the function returns `TRUE`.

#### Scenario: Short is longer than long
- **WHEN** user calls `chess_moves_subset('1. e4 e5', '1. e4')`
- **THEN** the function returns `FALSE`.
