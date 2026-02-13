## ADDED Requirements

### Requirement: Categorize TimeControl using Lichess speed definitions
The system SHALL provide a scalar SQL function `chess_timecontrol_category(time_control)` that derives a Lichess speed category from a parsed `TimeControl` value.

The category SHALL be computed from estimated duration:

- `estimated_seconds = base_seconds + 40 * increment_seconds`
- `increment_seconds` defaults to `0` when omitted

The category mapping SHALL be:

- `<= 29` -> `ultra-bullet`
- `<= 179` -> `bullet`
- `<= 479` -> `blitz`
- `<= 1499` -> `rapid`
- `>= 1500` -> `classical`

#### Scenario: Categorize ultra-bullet from explicit-seconds notation
- **WHEN** the input is `29''`
- **THEN** the output is `ultra-bullet`

#### Scenario: Categorize bullet lower band
- **WHEN** the input is `1+0`
- **THEN** the output is `bullet`

#### Scenario: Ambiguous small-base shorthand is treated as minutes
- **WHEN** the input is `29+0`
- **THEN** the output is `classical`

#### Scenario: Categorize blitz lower boundary
- **WHEN** the input is `3+0`
- **THEN** the output is `blitz`

#### Scenario: Categorize rapid from increment-heavy control
- **WHEN** the input is `2+12`
- **THEN** the output is `rapid`

#### Scenario: Categorize classical threshold
- **WHEN** the input is `25+0`
- **THEN** the output is `classical`

### Requirement: Category derivation SHALL reuse normalization parsing behavior
The system SHALL derive categories from existing TimeControl parsing rules so that canonical and inferred forms are categorized consistently.

#### Scenario: Inferred shorthand is categorized
- **WHEN** the input is `3+2`
- **THEN** the output is `blitz`

#### Scenario: Canonical equivalent is categorized identically
- **WHEN** the input is `180+2`
- **THEN** the output is `blitz`

#### Scenario: Unknown mode returns NULL
- **WHEN** the input is `?`
- **THEN** the output is NULL

#### Scenario: Unlimited mode returns NULL
- **WHEN** the input is `-`
- **THEN** the output is NULL

#### Scenario: Sandclock mode returns NULL
- **WHEN** the input is `*60`
- **THEN** the output is NULL

#### Scenario: Unparseable value returns NULL
- **WHEN** the input is `klassisch`
- **THEN** the output is NULL
