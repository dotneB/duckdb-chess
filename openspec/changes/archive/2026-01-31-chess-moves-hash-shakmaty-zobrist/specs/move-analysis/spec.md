## MODIFIED Requirements

### Requirement: Moves Hashing
The system SHALL provide a scalar function `chess_moves_hash(movetext)` that returns a consistent 64-bit unsigned integer representing the shakmaty Zobrist hash of the final chess position reached by applying the parsed mainline moves in `movetext`.

The function SHALL ignore formatting differences, comments, NAGs, and variations.

The function SHALL return a DuckDB `UBIGINT` containing the Zobrist hash value.

#### Scenario: Hash consistency
- **WHEN** user calls `chess_moves_hash` on two movetext strings with identical mainline moves but different comments/formatting
- **THEN** the returned hash values are identical.

#### Scenario: Hash discrimination for different final positions
- **WHEN** user calls `chess_moves_hash` on two movetext strings whose mainline moves lead to different final positions
- **THEN** the returned hash values are different (with high probability).

#### Scenario: Transposition collision
- **WHEN** user calls `chess_moves_hash('1. Nf3 d5 2. g3')` and `chess_moves_hash('1. g3 d5 2. Nf3')`
- **THEN** the returned hash values are identical.

#### Scenario: Empty input
- **WHEN** user calls `chess_moves_hash('')`
- **THEN** the function returns `NULL`.

#### Scenario: Null input
- **WHEN** user calls `chess_moves_hash(NULL)`
- **THEN** the function returns `NULL`.
