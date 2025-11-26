# move-analysis Specification

## Purpose
TBD - created by archiving change add-moves-function. Update Purpose after archive.
## Requirements
### Requirement: Move Extraction to JSON
The system SHALL provide a scalar function `moves_json(movetext)` that parses PGN movetext and returns a JSON array string containing details for every move.

#### Scenario: Explode valid game
- **WHEN** user calls `moves_json('1. e4 e5')`
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
  SELECT UNNEST(moves_json('1. e4')::JSON);
  -- Result: {"ply": 1, "move": "e4", "fen": "..."}
  ```

#### Scenario: Empty or Null input
- **WHEN** user calls `moves_json('')` or `moves_json(NULL)`
- **THEN** the function returns an empty JSON array `'[]'`

#### Scenario: Partial Validity
- **WHEN** the movetext contains an illegal move (e.g., `'1. e4 e5 2. Kxe8'`)
- **THEN** the function returns the JSON array for all valid moves processed up to the error (e.g., just the first two moves).

#### Scenario: Annotation Filtering
- **WHEN** the movetext contains comments/annotations (e.g., `'1. e4 {Best by test} e5'`)
- **THEN** the annotations are ignored, and the moves are parsed correctly.

