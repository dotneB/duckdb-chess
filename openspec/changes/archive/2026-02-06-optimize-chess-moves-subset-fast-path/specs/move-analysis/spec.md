## MODIFIED Requirements

### Requirement: Subsumption Detection
The system SHALL provide a scalar function `chess_moves_subset(short_movetext, long_movetext)` that returns `TRUE` if the moves in `short_movetext` are an exact prefix of the moves in `long_movetext`.

The implementation MAY use a conservative clean-input fast path for canonical mainline movetext.

When the fast path is used, the returned boolean SHALL be equivalent to parser-based mainline comparison.

When input is uncertain (including comments, recursive variations, NAGs, or ambiguous tokens), the function SHALL fall back to parser-based comparison.

The function SHALL ignore move numbers and trailing result markers when determining subset prefix semantics.

The function SHALL preserve DuckDB NULL propagation behavior.

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

#### Scenario: Fast path equivalence on clean canonical input
- **WHEN** user calls `chess_moves_subset('1. e4 e5', '1. e4 e5 2. Nf3 Nc6')`
- **THEN** the function returns the same boolean as parser-based subset comparison.

#### Scenario: Trailing result markers do not change subset decision
- **WHEN** user calls `chess_moves_subset('1. e4 e5 1-0', '1. e4 e5')`
- **THEN** the function returns `TRUE`.

#### Scenario: Fallback for annotated or uncertain input
- **WHEN** user calls `chess_moves_subset('1. e4 {comment} e5', '1. e4 e5 2. Nf3')`
- **THEN** the function uses parser-based behavior and returns `TRUE`.

#### Scenario: Null input propagation
- **WHEN** user calls `chess_moves_subset(NULL, '1. e4')` or `chess_moves_subset('1. e4', NULL)`
- **THEN** the function returns `NULL`.
