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

#### Scenario: Normalize apostrophe increment with per-move suffix
- **WHEN** the input is `3' + 2''/mv from move 1`
- **THEN** the normalized output equals `180+2`

#### Scenario: Normalize apostrophe increment with per-move suffix for larger base
- **WHEN** the input is `15' + 10''/mv from move 1`
- **THEN** the normalized output equals `900+10`

#### Scenario: Normalize compact minute-second text abbreviation
- **WHEN** the input is `3 mins + 2 seconds increment`
- **THEN** the normalized output equals `180+2`

#### Scenario: Normalize compact classical minute-second text abbreviation
- **WHEN** the input is `90 mins + 30 Secs`
- **THEN** the normalized output equals `5400+30`

#### Scenario: Normalize compact minute-second text with prefix labels
- **WHEN** the input is `Standard: 90mins + 30sec increment`
- **THEN** the normalized output equals `5400+30`

#### Scenario: Normalize compact classical plus trailing qualifier text
- **WHEN** the input is `90 + 30 OFICIAL`
- **THEN** the normalized output equals `5400+30`

#### Scenario: Do not strip suffixes that include structural numeric tokens
- **WHEN** the input is `90 + 30 round2`
- **THEN** the normalized output is NULL

#### Scenario: Normalize clock-style base with increment
- **WHEN** the input is `1:30.00 + 30 seconds increment from move 1`
- **THEN** the normalized output equals `5400+30`

#### Scenario: Normalize compact FIDE two-stage apostrophe shorthand with game token
- **WHEN** the input is `90'/40+30'/G+30''`
- **THEN** the normalized output equals `40/5400+30:1800+30`

#### Scenario: Normalize compact FIDE two-stage apostrophe shorthand with end token
- **WHEN** the input is `90'/40m + 30'/end + 30'/move`
- **THEN** the normalized output equals `40/5400+30:1800+30`

#### Scenario: Normalize compact FIDE two-stage bonus wording
- **WHEN** the input is `90'/40 moves + 30' + 30'' bonus increment`
- **THEN** the normalized output equals `40/5400+30:1800+30`

#### Scenario: Normalize compact FIDE two-stage additional wording
- **WHEN** the input is `90mins+30second additional +30mins after move 40`
- **THEN** the normalized output equals `40/5400+30:1800+30`

#### Scenario: Normalize compact staged triple-plus shorthand
- **WHEN** the input is `90 + 30 + 30s per move`
- **THEN** the normalized output equals `40/5400+30:1800+30`

#### Scenario: Normalize staged shorthand missing move qualifier without inventing moves
- **WHEN** the input is `90+30/30+30`
- **THEN** the normalized output equals `5400+30:1800+30`

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

### Requirement: TimeControl pipeline materialization guidance
The project SHALL document practical SQL patterns for materializing analytics-ready TimeControl columns from `chess_timecontrol_normalize`, `chess_timecontrol_json`, and `chess_timecontrol_category`.

#### Scenario: Materialized normalized and category columns
- **WHEN** users build persistent analytic tables from `read_pgn`
- **THEN** documentation includes a pattern that materializes normalized TimeControl and category columns for repeated filtering/aggregation

#### Scenario: JSON parse details extraction
- **WHEN** users need typed fields such as base seconds, increment seconds, or inference flags
- **THEN** documentation includes a JSON extraction pattern over `chess_timecontrol_json(...)` to derive those fields in SQL

#### Scenario: NULL-safe materialization guidance
- **WHEN** users materialize TimeControl-derived columns at scale
- **THEN** examples include NULL handling for missing/unparseable values so analytic semantics stay deterministic
