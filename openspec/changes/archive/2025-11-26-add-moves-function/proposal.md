# Change: Add moves_json function

## Why
Users need to analyze individual moves and positions within a game (e.g., for blunder checks, opening exploration, or position indexing). The current `read_pgn` function returns the entire game as a single row. There is a need to "explode" the `movetext` column into individual moves with their resulting FEN positions.

## What Changes
- Add a new **scalar function** `moves_json(movetext)` that accepts a PGN movetext string.
- Returns a **JSON string** (VARCHAR) containing an array of move objects.
  - JSON Schema: `[{"ply": int, "move": string, "fen": string}, ...]`
- Uses the `shakmaty` library to parse SAN moves and generate FEN strings.
- Handles standard chess moves and standard FEN output.
- **Why JSON?** returning a JSON string is the most robust way to pass complex structured data from a Rust extension to DuckDB without relying on unstable or complex vector type ABIs. DuckDB has excellent native JSON support (`::JSON`, `UNNEST`, `->>`) to process this output.

## Impact
- **New Capability**: `move-analysis`
- **Affected Code**: 
  - `src/chess/moves.rs` (Implementation of the scalar function)
  - `src/chess/mod.rs` (Registration)

## Usage Example
```sql
-- Explode games into individual moves
SELECT 
    -- Game Metadata
    Event,
    White,
    Black,
    
    -- Move Data extracted from the JSON object
    move_data.ply,
    move_data.move as move_san,
    move_data.fen
FROM (
    SELECT 
        Event, 
        White, 
        Black, 
        -- 1. Convert movetext to JSON string using our extension
        -- 2. Cast string to a specific STRUCT array type so UNNEST knows the schema
        UNNEST(moves_json(movetext)::STRUCT(ply INTEGER, move VARCHAR, fen VARCHAR)[]) as move_data
    FROM read_pgn('test/pgn_files/*.pgn')
);
```
```sql
SELECT 
    -- Take only the first part of the FEN (the piece placement)
    split_part(move_data.fen, ' ', 1) as board_position,
    COUNT(*) as frequency
FROM (
    SELECT 
        UNNEST(moves_json(movetext)::STRUCT(ply INTEGER, move VARCHAR, fen VARCHAR)[]) as move_data
    FROM read_pgn('test/pgn_files/*.pgn')
)
GROUP BY board_position
ORDER BY frequency DESC
LIMIT 10;
```
```sql
WITH games AS (
    SELECT 
        -- Create a unique ID for each game to keep moves sequences separate
        row_number() OVER () as game_id,
        UNNEST(moves_json(movetext)::STRUCT(ply INTEGER, move VARCHAR, fen VARCHAR)[]) as data
    FROM read_pgn('test/pgn_files/*.pgn')
),
position_analysis AS (
    SELECT 
        -- Extract the board position (everything before the first space)
        split_part(data.fen, ' ', 1) as board_fen,
        
        -- Look ahead to find the move played immediately AFTER this position
        LEAD(data.move) OVER (PARTITION BY game_id ORDER BY data.ply) as next_move
    FROM games
)
SELECT 
    board_fen,
    COUNT(*) as frequency,
    -- Creates a map of { move: count } showing distribution of next moves
    histogram(next_move) as next_moves_distribution
FROM position_analysis
WHERE next_move IS NOT NULL
GROUP BY board_fen
ORDER BY frequency DESC
LIMIT 10;
```