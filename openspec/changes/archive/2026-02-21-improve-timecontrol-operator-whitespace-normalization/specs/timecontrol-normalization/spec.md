## MODIFIED Requirements

### Requirement: Lenient parsing for common real-world shorthand
The system SHALL interpret a limited set of common non-spec `TimeControl` shorthands and normalize them into canonical seconds-based output.

Any such interpretation SHALL be marked as inferred in the structured parse output.

The system SHALL treat optional whitespace around structural operators (`+`, `/`, `:`) as semantically insignificant when the surrounding tokens otherwise form a valid control.

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

#### Scenario: Normalize spaces around plus operator
- **WHEN** the input is `15 + 10`
- **THEN** the normalized output equals `900+10`

#### Scenario: Normalize spaces around slash and colon operators in staged control
- **WHEN** the input is `40 / 5400 + 30 : 1800 + 30`
- **THEN** the normalized output equals `40/5400+30:1800+30`

#### Scenario: Normalize mixed operator whitespace in stage-by-moves shorthand
- **WHEN** the input is `90 + 30 / 30 + 30`
- **THEN** the normalized output equals `5400+30:1800+30`

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
