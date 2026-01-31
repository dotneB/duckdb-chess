# move-analysis Specification

## Purpose
To provide analytical functions for chess move sequences, enabling JSON extraction, normalization, hashing, and subset comparison of movetext to support game analysis and comparison workflows.
## Requirements
### Requirement: Move Extraction to JSON
The system SHALL provide scalar function overloads `chess_moves_json(movetext)` and `chess_moves_json(movetext, max_ply)` that parse PGN movetext and return a JSON array string containing details for moves.
Each JSON object SHALL include:
- `ply` (integer)
- `move` (SAN token string)
- `fen` (FEN string of the position after the move)
- `epd` (string) derived from `fen` by taking the first 4 FEN fields: `board side castling ep`

For `chess_moves_json(movetext, max_ply)`, the function SHALL return at most `max_ply` move objects.

For `chess_moves_json(movetext, max_ply)`:
- If `max_ply` is `NULL`, the function SHALL behave like `chess_moves_json(movetext)`.
- If `max_ply <= 0`, the function SHALL return an empty JSON array `'[]'`.

#### Scenario: Explode valid game
- **WHEN** user calls `chess_moves_json('1. e4 e5')`
- **THEN** the function returns a JSON string:
  ```json
  [
    {"ply": 1, "move": "e4", "fen": "rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 1", "epd": "rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3"},
    {"ply": 2, "move": "e5", "fen": "rnbqkbnr/pppp1ppp/8/4p3/4P3/8/PPPP1PPP/RNBQKBNR w KQkq e6 0 2", "epd": "rnbqkbnr/pppp1ppp/8/4p3/4P3/8/PPPP1PPP/RNBQKBNR w KQkq e6"}
  ]
  ```

#### Scenario: Pipeline opening join (recommended detection)
- **WHEN** user loads the Lichess openings dataset into DuckDB with columns including `epd`, `eco`, and `name`
- **AND** user explodes moves with `json_each(CAST(chess_moves_json(movetext) AS JSON))`
- **AND** joins on `epd`
- **THEN** the user can classify by selecting the matched row with the highest `ply` (scan backwards until a named position is found)

  Example:
  ```sql
  WITH params AS (
    SELECT 40 AS max_opening_ply
  ),
  pos AS (
    SELECT
      g.game_id,
      json_extract(m.value, '$.ply')::INTEGER AS ply,
      trim(json_extract_string(m.value, '$.epd')) AS epd
    FROM games g,
         params p,
         json_each(CAST(chess_moves_json(g.movetext, p.max_opening_ply) AS JSON)) m
  ),
  matches AS (
    SELECT
      p.game_id,
      p.ply,
      o.eco,
      o.name,
      array_length(string_split(o.uci, ' ')) AS opening_ply
    FROM pos p
    JOIN openings o
      ON trim(o.epd) = p.epd
  )
  SELECT game_id, ply, eco, name
  FROM (
    SELECT
      game_id,
      ply,
      eco,
      name,
      row_number() OVER (
        PARTITION BY game_id
        ORDER BY ply DESC, opening_ply ASC
      ) AS rn
    FROM matches
  )
  WHERE rn = 1;
  ```

#### Scenario: Max ply limit
- **WHEN** user calls `chess_moves_json('1. e4 e5 2. Nf3', 2)`
- **THEN** the function returns a JSON array containing 2 move objects.

#### Scenario: Max ply zero or negative
- **WHEN** user calls `chess_moves_json('1. e4 e5', 0)`
- **THEN** the function returns an empty JSON array `'[]'`

#### Scenario: SQL Integration
- **WHEN** used with DuckDB's JSON functions
- **THEN** the output can be unnested to rows:
  ```sql
  SELECT
    json_extract(m.value, '$.ply')::INTEGER AS ply,
    json_extract_string(m.value, '$.epd')  AS epd
  FROM json_each(CAST(chess_moves_json('1. e4') AS JSON)) m;
  ```

#### Scenario: Empty or Null input
- **WHEN** user calls `chess_moves_json('')` or `chess_moves_json(NULL)`
- **THEN** the function returns an empty JSON array `'[]'`

#### Scenario: Partial Validity
- **WHEN** the movetext contains an illegal move (e.g., `'1. e4 e5 2. Kxe8'`)
- **THEN** the function returns the JSON array for all valid moves processed up to the error.

#### Scenario: Annotation Filtering
- **WHEN** the movetext contains comments/annotations (e.g., `'1. e4 {Best by test} e5'`)
- **THEN** the annotations are ignored, and the moves are parsed correctly.

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

### Requirement: Ply Count
The system SHALL provide a scalar function `chess_ply_count(movetext)` that returns the number of plies parsed from PGN movetext according to `pgn-reader`.

#### Scenario: Count plies
- **WHEN** user calls `chess_ply_count('1. e4 e5 2. Nf3')`
- **THEN** the function returns `3`

#### Scenario: Ignore move numbers and result markers
- **WHEN** user calls `chess_ply_count('1. e4 e5 1-0')`
- **THEN** the function returns `2`

The function SHALL count syntactically valid SAN tokens and SHALL NOT validate move legality.

Unknown tokens in the input SHALL NOT stop counting if `pgn-reader` can continue parsing subsequent SAN tokens.

If the input cannot be parsed as PGN movetext, the function SHALL return `0`.

#### Scenario: Unknown token does not stop counting
- **WHEN** user calls `chess_ply_count('1. e4 e5 INVALID 2. Nf3')`
- **THEN** the function returns `3`

#### Scenario: Unknown token between moves
- **WHEN** user calls `chess_ply_count('1. e4 INVALID 2. Nf3')`
- **THEN** the function returns `2`

#### Scenario: Illegal but syntactically valid SAN still counts
- **WHEN** user calls `chess_ply_count('1. e4 e5 2. Kxe8')`
- **THEN** the function returns `3`

#### Scenario: Empty or Null input
- **WHEN** user calls `chess_ply_count('')` or `chess_ply_count(NULL)`
- **THEN** the function returns `0`

### Requirement: FEN to EPD
The system SHALL provide a scalar function `chess_fen_epd(fen)` that returns an EPD string compatible with the Lichess openings dataset.

The EPD format SHALL be the first 4 fields of a valid FEN string: `board side castling ep`.

#### Scenario: Convert FEN to EPD
- **WHEN** user calls `chess_fen_epd('rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 1')`
- **THEN** the function returns `'rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3'`

#### Scenario: Preserve en passant dash
- **WHEN** user calls `chess_fen_epd('rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1')`
- **THEN** the function returns `'rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq -'`

#### Scenario: Null or empty input
- **WHEN** user calls `chess_fen_epd(NULL)` or `chess_fen_epd('')`
- **THEN** the function returns `NULL`

#### Scenario: Invalid input
- **WHEN** user calls `chess_fen_epd('not a fen')`
- **THEN** the function returns `NULL`
