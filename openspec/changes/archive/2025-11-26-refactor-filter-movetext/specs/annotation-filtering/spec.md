# Annotation Filtering Capability

## MODIFIED Requirements

### Requirement: Movetext Annotation Removal
The system SHALL provide a `filter_movetext_annotations(movetext)` **scalar** function that removes curly brace annotations from chess movetext while preserving the move structure.

#### Scenario: Scalar usage in projection
- **WHEN** user calls `SELECT filter_movetext_annotations(movetext) FROM table`
- **THEN** the function returns the filtered movetext for each row

#### Scenario: Empty input handling
- **WHEN** the function receives an empty string
- **THEN** it returns an empty string

## REMOVED Requirements

### Requirement: Single Row Output
(This requirement defined the table function behavior of returning exactly one row. Scalar functions implicitly return one value per input, so this explicit table-function behavior is no longer needed).

#### Scenario: Table function result
#### Scenario: Empty input handling
