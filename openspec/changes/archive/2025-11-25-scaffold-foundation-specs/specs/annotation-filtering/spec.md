# Annotation Filtering Capability

## ADDED Requirements

### Requirement: Movetext Annotation Removal
The system SHALL provide a `filter_movetext_annotations(movetext)` table function that removes curly brace annotations from chess movetext while preserving the move structure.

#### Scenario: Remove single annotation
- **WHEN** user calls `filter_movetext_annotations('1. e4 { comment } e5')`
- **THEN** the function returns `'1. e4 e5'` with the annotation removed

#### Scenario: Remove multiple annotations
- **WHEN** movetext contains multiple annotations like `'1. e4 { first } e5 { second } 2. Nf3 { third }'`
- **THEN** the function removes all annotations and returns `'1. e4 e5 2. Nf3'`

#### Scenario: Preserve movetext without annotations
- **WHEN** movetext contains no annotations like `'1. e4 e5 2. Nf3 Nc6'`
- **THEN** the function returns the movetext unchanged

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

### Requirement: Single Row Output
The system SHALL return exactly one row containing the filtered movetext string.

#### Scenario: Table function result
- **WHEN** the function is called with any valid movetext
- **THEN** it returns a single-row, single-column result with the filtered text

#### Scenario: Empty input handling
- **WHEN** the function receives an empty string
- **THEN** it returns a single row with an empty filtered result
