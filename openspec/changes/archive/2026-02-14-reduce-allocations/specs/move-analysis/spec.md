## ADDED Requirements

### Requirement: Low-Allocation Move Analysis Execution
The system SHALL implement `chess_moves_json`, `chess_ply_count`, and `chess_moves_normalize` with streaming or equivalently low-allocation internal processing that avoids unnecessary intermediate move-vector materialization.

#### Scenario: JSON extraction can stop at max ply without full materialization
- **WHEN** `chess_moves_json(movetext, max_ply)` is called with a positive `max_ply`
- **THEN** processing may stop after `max_ply` plies while returning output equivalent to existing contract semantics

#### Scenario: Ply counting avoids move-sequence buffering
- **WHEN** `chess_ply_count(movetext)` is called on long movetext input
- **THEN** ply counting is computed without allocating a full `Vec<String>` of SAN tokens

### Requirement: Allocation Refactors Preserve Move-Analysis Contracts
Allocation-focused refactors in move-analysis internals MUST preserve all existing requirement-level behavior for output values, truncation semantics, and degradation on malformed input.

#### Scenario: Null and empty handling remain unchanged
- **WHEN** users call `chess_moves_json`, `chess_moves_normalize`, or `chess_ply_count` with NULL or empty inputs
- **THEN** each function returns results matching previously specified behavior for that function

#### Scenario: Partial-validity semantics remain unchanged
- **WHEN** move analysis encounters illegal or malformed SAN during processing
- **THEN** each function preserves existing partial-output or fallback behavior as currently specified
