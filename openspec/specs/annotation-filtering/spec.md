# Annotation Filtering Capability

## Purpose
To provide mechanisms for cleaning and normalizing chess game text, specifically removing annotations (comments) to facilitate analysis of raw move sequences.
## Requirements
### Requirement: Nested Annotation Handling
The system SHALL correctly handle nested curly braces within annotations.

#### Scenario: Nested braces
- **WHEN** an annotation contains nested braces like `'1. e4 { outer { inner } text } e5'`
- **THEN** the function tracks brace depth and removes the entire nested structure

#### Scenario: Brace depth tracking
- **WHEN** processing annotations with multiple nesting levels
- **THEN** the function maintains a brace depth counter to determine when annotation ends

### Requirement: Whitespace Normalization
The system SHALL normalize whitespace in the filtered output.

#### Scenario: Remove extra spaces
- **WHEN** annotation removal creates multiple consecutive spaces
- **THEN** the function collapses them into single spaces

#### Scenario: Trim leading and trailing whitespace
- **WHEN** the filtered result has leading or trailing spaces
- **THEN** the function trims them from the final output

### Requirement: Move Structure Preservation
The system SHALL preserve chess move numbering and notation exactly as provided in the input.

#### Scenario: Preserve move numbers
- **WHEN** movetext includes move numbers like `'1. e4'` and `'2. Nf3'`
- **THEN** the function preserves all move numbers in the output

#### Scenario: Preserve move notation
- **WHEN** movetext uses Standard Algebraic Notation (SAN)
- **THEN** all move symbols (pieces, squares, check/checkmate indicators) remain unchanged

