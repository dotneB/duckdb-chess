## ADDED Requirements

### Requirement: Inference arithmetic MUST be overflow-safe
The system SHALL use checked arithmetic for all inference-path minute/second math, including minute-to-second conversion and base-plus-increment composition.

#### Scenario: Overflow during inferred minute conversion
- **WHEN** input matches an inferred minute-style form whose conversion exceeds `u32` bounds (for example `71582789 mins + 1 seconds`)
- **THEN** normalization returns NULL (`normalized = NULL`)
- **AND** warnings include `inference_arithmetic_overflow`
- **AND** no wrapped numeric value is emitted in normalized output

#### Scenario: Boundary value remains valid
- **WHEN** input matches an inferred minute-style form at the exact `u32` boundary (for example `71582788 mins + 15 seconds`)
- **THEN** normalization returns `4294967295`
- **AND** warnings do not include `inference_arithmetic_overflow`

### Requirement: Overflow handling SHALL be consistent across scalar outputs
The system SHALL propagate inference overflow outcomes consistently for normalize, category, and JSON functions.

#### Scenario: Category output on overflow
- **WHEN** input triggers inference arithmetic overflow
- **THEN** `chess_timecontrol_category(...)` returns NULL

#### Scenario: JSON output on overflow
- **WHEN** input triggers inference arithmetic overflow
- **THEN** `chess_timecontrol_json(...)` includes `"normalized": null`
- **AND** `warnings` contains `inference_arithmetic_overflow`

### Requirement: Existing non-overflow behavior remains stable
The system SHALL preserve current normalization/category semantics for inputs that do not overflow.

#### Scenario: Existing inferred shorthand remains unchanged
- **WHEN** the input is `3+2`
- **THEN** normalized output equals `180+2`
- **AND** category output remains `blitz`

#### Scenario: Existing canonical value remains unchanged
- **WHEN** the input is `180+2`
- **THEN** normalized output equals `180+2`
- **AND** category output remains `blitz`
