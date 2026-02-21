## ADDED Requirements

### Requirement: TimeControl quote preprocessing SHALL preserve structural semantics
The system SHALL preprocess quote characters in raw `TimeControl` values using context-aware rules so that non-semantic wrappers are removed while structural numeric/operator content and apostrophe unit markers remain intact.

#### Scenario: Normalize mixed wrapper quotes around canonical value
- **WHEN** the input is `'"180+2"'`
- **THEN** the normalized output equals `180+2`

#### Scenario: Preserve apostrophe unit notation during quote cleanup
- **WHEN** the input is `3' + 2''`
- **THEN** the normalized output equals `180+2`

#### Scenario: Normalize repeated outer quote noise around minute shorthand
- **WHEN** the input is `''"15 + 10"''`
- **THEN** the normalized output equals `900+10`

### Requirement: Ambiguous quoted residue SHALL degrade safely
If quote preprocessing cannot produce an unambiguous structural token stream, the system MUST fail normalization safely instead of inferring stitched numeric values.

#### Scenario: Unbalanced quoted fragment returns NULL
- **WHEN** the input is `"90 + "30`
- **THEN** the normalized output is NULL

#### Scenario: Category output follows safe failure on ambiguous quotes
- **WHEN** the input is `"90 + "30`
- **THEN** `chess_timecontrol_category(...)` returns NULL

#### Scenario: JSON output exposes failed normalization on ambiguous quotes
- **WHEN** the input is `"90 + "30`
- **THEN** `chess_timecontrol_json(...)` includes `"normalized": null`
