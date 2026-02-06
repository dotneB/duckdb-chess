## MODIFIED Requirements

### Requirement: Subsumption Detection
The system SHALL provide a scalar function `chess_moves_subset(short_movetext, long_movetext)` that returns `TRUE` if the moves in `short_movetext` are an exact prefix of the moves in `long_movetext`.

The function SHALL parse both inputs as mainline movetext and compare normalized move tokens.

The function SHALL treat empty-string inputs as empty move sequences.

The function SHALL return `FALSE` if either non-NULL input is non-empty and cannot be parsed as movetext.

The function SHALL preserve DuckDB NULL propagation semantics.

#### Scenario: Exact subset
- **WHEN** user calls `chess_moves_subset('1. e4', '1. e4 e5')`
- **THEN** the function returns `TRUE`

#### Scenario: Not a subset
- **WHEN** user calls `chess_moves_subset('1. d4', '1. e4 e5')`
- **THEN** the function returns `FALSE`

#### Scenario: Same game
- **WHEN** user calls `chess_moves_subset('1. e4', '1. e4')`
- **THEN** the function returns `TRUE`

#### Scenario: Short is longer than long
- **WHEN** user calls `chess_moves_subset('1. e4 e5', '1. e4')`
- **THEN** the function returns `FALSE`

#### Scenario: Empty short input
- **WHEN** user calls `chess_moves_subset('', '1. e4')`
- **THEN** the function returns `TRUE`

#### Scenario: Empty long input with non-empty short input
- **WHEN** user calls `chess_moves_subset('1. e4', '')`
- **THEN** the function returns `FALSE`

#### Scenario: Both inputs empty
- **WHEN** user calls `chess_moves_subset('', '')`
- **THEN** the function returns `TRUE`

#### Scenario: Invalid non-empty short input
- **WHEN** user calls `chess_moves_subset('not movetext', '1. e4')`
- **THEN** the function returns `FALSE`

#### Scenario: Invalid non-empty long input
- **WHEN** user calls `chess_moves_subset('1. e4', 'not movetext')`
- **THEN** the function returns `FALSE`

#### Scenario: Both inputs invalid and non-empty
- **WHEN** user calls `chess_moves_subset('not movetext', 'still not movetext')`
- **THEN** the function returns `FALSE`

#### Scenario: Null input propagation
- **WHEN** user calls `chess_moves_subset(NULL, '1. e4')` or `chess_moves_subset('1. e4', NULL)`
- **THEN** the function returns `NULL`
