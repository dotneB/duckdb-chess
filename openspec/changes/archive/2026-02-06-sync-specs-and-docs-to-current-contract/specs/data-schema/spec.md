## MODIFIED Requirements

### Requirement: Movetext Column
The system SHALL provide a column containing the parsed mainline movetext in a PGN-compatible format.

#### Scenario: Movetext with annotations
- **WHEN** a game includes moves with comments and annotations
- **THEN** the movetext column contains the moves in SAN format
- **AND** comments are included in `{ ... }` form
- **AND** leading/trailing whitespace inside comments is normalized

#### Scenario: Movetext format
- **WHEN** movetext is stored
- **THEN** it uses Standard Algebraic Notation with move numbers (e.g., `1. e4 e5 2. Nf3 Nc6`)
- **AND** variations are not included (mainline only)

#### Scenario: Movetext always present
- **WHEN** a game is parsed successfully
- **THEN** the movetext column always contains a non-NULL value (empty string if no mainline moves)

#### Scenario: Result marker is not embedded in movetext
- **WHEN** a game has a result (from movetext outcome marker or `Result` header)
- **THEN** the movetext column does NOT append a terminal result marker token (`1-0`, `0-1`, `1/2-1/2`, or `*`)
- **AND** game result metadata is exposed through the `Result` column
