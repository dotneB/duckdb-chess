## ADDED Requirements

### Requirement: Visitor tag decoding avoids unnecessary allocation
The `read_pgn` PGN visitor MUST avoid decoding or allocating tag values that do not contribute to the emitted row, including unknown tags and duplicate known tags that are ignored by existing semantics.

#### Scenario: Unknown tags do not affect output
- **WHEN** a game contains header tags outside the known output tag set
- **THEN** those tags do not affect any emitted output columns
- **AND** parsing continues normally

#### Scenario: Duplicate known tag uses first value
- **WHEN** a game contains a known header tag more than once (e.g., two `Event` tags)
- **THEN** the emitted column uses the first parsed value
- **AND** later duplicates are ignored

### Requirement: Movetext finalization avoids unnecessary cloning
The `read_pgn` visitor MUST avoid creating a new owned movetext string during record finalization when trimming would not change the assembled movetext, while preserving the exact trimming semantics of the current implementation.

#### Scenario: Movetext with no surrounding whitespace is preserved
- **WHEN** the visitor assembles movetext with no leading or trailing whitespace
- **THEN** the emitted movetext value is identical to the assembled movetext

#### Scenario: Movetext surrounding whitespace is trimmed
- **WHEN** the visitor assembles movetext that includes leading or trailing whitespace
- **THEN** the emitted movetext value matches the result of trimming surrounding whitespace
