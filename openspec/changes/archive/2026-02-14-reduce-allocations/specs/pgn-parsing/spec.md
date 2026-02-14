## ADDED Requirements

### Requirement: Low-Allocation Header Capture
The PGN visitor SHALL capture known header fields into dedicated structured storage during tag visitation, avoiding repeated full-header scans and unnecessary cloning for common header lookups.

#### Scenario: Known header values are captured directly
- **WHEN** the visitor processes standard tags such as `Event`, `Site`, `White`, and `Black`
- **THEN** values are stored in dedicated fields used for row construction without repeated linear scans over a tag vector

#### Scenario: Unsupported or extra tags remain non-blocking
- **WHEN** the visitor encounters tags outside the known Lichess-style output set
- **THEN** parsing continues without affecting required output columns or error handling semantics

### Requirement: Low-Allocation Movetext Assembly
The visitor SHALL build mainline movetext using append-oriented string operations that minimize temporary allocation while preserving existing formatting and comment behavior.

#### Scenario: Mainline formatting parity
- **WHEN** parsing a valid game with move numbers and SAN tokens
- **THEN** movetext output remains equivalent to existing mainline formatting behavior

#### Scenario: Comment formatting parity
- **WHEN** parsing comments within `{...}` blocks
- **THEN** comments remain represented in `{ ... }` form with existing whitespace-normalization semantics
