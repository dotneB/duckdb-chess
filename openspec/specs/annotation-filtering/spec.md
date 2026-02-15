# Annotation Filtering Capability

## Purpose
To provide mechanisms for cleaning and normalizing chess game text, specifically removing annotations (comments) to facilitate analysis of raw move sequences.
## Requirements
### Requirement: Parser-backed annotation filtering
The system SHALL implement annotation filtering through PGN parser/visitor processing rather than ad-hoc text scanning algorithms.

#### Scenario: Comments are ignored by parser-backed normalization
- **WHEN** movetext contains PGN comments such as `'1. e4 {Best by test} e5'`
- **THEN** normalized output excludes comment content and preserves mainline SAN moves

#### Scenario: Variations and NAGs are ignored consistently
- **WHEN** movetext contains variations and NAGs such as `'1. e4 (1. d4) e5?! $1'`
- **THEN** normalized output includes only the canonical mainline move sequence

### Requirement: Whitespace Normalization
The system SHALL normalize whitespace in the filtered output.

#### Scenario: Remove extra spaces
- **WHEN** annotation removal creates multiple consecutive spaces
- **THEN** the function collapses them into single spaces

#### Scenario: Trim leading and trailing whitespace
- **WHEN** the filtered result has leading or trailing spaces
- **THEN** the function trims them from the final output

#### Scenario: Lichess-style bracket payloads inside comments are removed
- **WHEN** comments include payloads like `[%eval ...]` and `[%clk ...]`
- **THEN** normalization removes the comment blocks while preserving surrounding move sequence semantics

### Requirement: Move Structure Preservation
The system SHALL preserve mainline move semantics while emitting normalized movetext in canonical PGN-style formatting.

#### Scenario: Preserve move numbers
- **WHEN** movetext includes move numbers like `'1. e4'` and `'2. Nf3'`
- **THEN** normalized output includes appropriate move numbers for the preserved mainline sequence

#### Scenario: Preserve move notation
- **WHEN** movetext uses Standard Algebraic Notation (SAN)
- **THEN** SAN tokens in the preserved mainline remain semantically equivalent after normalization
