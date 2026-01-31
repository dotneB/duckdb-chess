## MODIFIED Requirements

### Requirement: Moves Normalization
The system SHALL provide a scalar function `chess_moves_normalize(movetext)` that returns a canonical string representation of the move sequence, removing comments, recursive variations, and NAGs, while standardizing spacing.

If `movetext` is not `NULL` but cannot be parsed as PGN movetext, the function SHALL return an empty string.

#### Scenario: Normalize complex movetext
- **WHEN** user calls `chess_moves_normalize('1. e4 {comment} (1. d4) e5?!')`
- **THEN** the function returns `'1. e4 e5'` (or similar standard format, ensuring clean SAN moves).

#### Scenario: Handle NULL input
- **WHEN** user calls `chess_moves_normalize(NULL)`
- **THEN** the function returns `NULL`.

#### Scenario: Parse failure returns empty string
- **WHEN** user calls `chess_moves_normalize('this is not movetext')`
- **THEN** the function returns an empty string.

### Requirement: Ply Count
The system SHALL provide a scalar function `chess_ply_count(movetext)` that returns the number of plies parsed from PGN movetext according to `pgn-reader`.

The function SHALL count syntactically valid SAN tokens and SHALL NOT validate move legality.

Unknown tokens in the input SHALL NOT stop counting if `pgn-reader` can continue parsing subsequent SAN tokens.

If the input cannot be parsed as PGN movetext, the function SHALL return `0`.

#### Scenario: Unknown token does not stop counting
- **WHEN** user calls `chess_ply_count('1. e4 e5 INVALID 2. Nf3')`
- **THEN** the function returns `3`.

#### Scenario: Unknown token between moves
- **WHEN** user calls `chess_ply_count('1. e4 INVALID 2. Nf3')`
- **THEN** the function returns `2`.

#### Scenario: Illegal but syntactically valid SAN still counts
- **WHEN** user calls `chess_ply_count('1. e4 e5 2. Kxe8')`
- **THEN** the function returns `3`.
