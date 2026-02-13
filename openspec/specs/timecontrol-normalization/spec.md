# timecontrol-normalization Specification

## Purpose
To normalize raw PGN `TimeControl` tags into canonical, seconds-based values and expose structured parse metadata for SQL analysis.

## Requirements

### Requirement: Normalize TimeControl tag to canonical seconds
The system SHALL provide a function to normalize a raw PGN `TimeControl` tag value into a canonical, PGN spec-shaped, seconds-based string.

The canonical format SHALL use only these forms:

- unknown: `?`
- no time limit: `-`
- sandclock: `*<seconds>`
- stage: `<seconds>` or `<seconds>+<increment_seconds>`
- stage-by-moves: `<moves>/<seconds>` or `<moves>/<seconds>+<increment_seconds>`
- multi-stage: stages separated by `:`

#### Scenario: Normalize already-spec value
- **WHEN** the input `TimeControl` is already spec-shaped (for example `180+2`)
- **THEN** the normalized output equals the input exactly (`180+2`)

#### Scenario: Normalize no-time-limit and unknown
- **WHEN** the input is `-` or `?`
- **THEN** the normalized output equals the input

#### Scenario: Normalize sandclock
- **WHEN** the input is `*60`
- **THEN** the normalized output equals `*60`

#### Scenario: Normalize multi-stage
- **WHEN** the input is `40/5400+30:1800+30`
- **THEN** the normalized output equals `40/5400+30:1800+30`

### Requirement: Lenient parsing for common real-world shorthand
The system SHALL interpret a limited set of common non-spec `TimeControl` shorthands and normalize them into canonical seconds-based output.

Any such interpretation SHALL be marked as inferred in the structured parse output.

#### Scenario: Interpret minute shorthand N+I as minutes+seconds
- **WHEN** the input matches `N+I` where `I <= 60` and `N < 60` (for example `3+2`)
- **THEN** the normalized output equals `<N*60>+<I>` (for example `180+2`)

#### Scenario: Interpret minute shorthand for classical bases 75/90 with +30
- **WHEN** the input matches `N+30` where `N` is `75` or `90`
- **THEN** the normalized output equals `<N*60>+30` (for example `5400+30`)

#### Scenario: Do not reinterpret large spec-shaped values
- **WHEN** the input is `900+10`
- **THEN** the normalized output equals `900+10`

#### Scenario: Interpret bare minute shorthand
- **WHEN** the input is a bare integer `N` where `N < 60` (for example `25`)
- **THEN** the normalized output equals `<N*60>` (for example `1500`)

#### Scenario: Normalize punctuation variants
- **WHEN** the input is `75 | 30`
- **THEN** the normalized output equals `4500+30`

#### Scenario: Normalize quoted values
- **WHEN** the input is surrounded by quotes (for example `"180+2"`)
- **THEN** the normalized output equals `180+2`

#### Scenario: Normalize spaces around operators
- **WHEN** the input is `15 + 10`
- **THEN** the normalized output equals `900+10`

#### Scenario: Normalize apostrophe minute/second notation
- **WHEN** the input is `10'+5''`
- **THEN** the normalized output equals `600+5`

### Requirement: Structured TimeControl parsing output
The system SHALL provide a function that returns a structured representation of a `TimeControl` parse, including inference warnings.

The structured output SHALL include:

- `raw` (original input)
- `normalized` (canonical string or NULL)
- `mode` (`unknown`, `unlimited`, `sandclock`, `normal`)
- `periods` (array of stages)
- `warnings` (array of warning codes)
- `inferred` (boolean)

#### Scenario: Structured output for inferred shorthand
- **WHEN** the input is `3+2`
- **THEN** `normalized` equals `180+2`
- **AND** `inferred` is `true`
- **AND** `warnings` contains `interpreted_small_base_as_minutes`

#### Scenario: Structured output for strict value
- **WHEN** the input is `180+2`
- **THEN** `normalized` equals `180+2`
- **AND** `inferred` is `false`

### Requirement: Failure behavior
The system SHALL return NULL normalized output for values that cannot be parsed or normalized with high confidence.

#### Scenario: Unparseable free text returns NULL
- **WHEN** the input is `klassisch`
- **THEN** the normalized output is NULL
