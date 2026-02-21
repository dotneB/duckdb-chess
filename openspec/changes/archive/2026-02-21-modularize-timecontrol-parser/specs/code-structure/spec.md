## ADDED Requirements

### Requirement: TimeControl parser internals are decomposed by responsibility
The project MUST decompose TimeControl parsing internals into dedicated modules for strict parsing, inference/normalization, and JSON rendering while preserving backward-compatible SQL behavior.

#### Scenario: Dedicated module boundaries exist
- **WHEN** maintainers review TimeControl implementation layout
- **THEN** strict parsing, inference logic, and JSON rendering are implemented in separate `src/chess/timecontrol/*` modules
- **AND** a thin `timecontrol` facade exposes the existing public entrypoints

#### Scenario: SQL API remains stable after refactor
- **WHEN** users call `chess_timecontrol_normalize`, `chess_timecontrol_json`, and `chess_timecontrol_category`
- **THEN** function names and return contracts remain unchanged
- **AND** NULL handling semantics remain unchanged

#### Scenario: Warning taxonomy and inference semantics are preserved
- **WHEN** existing inferred shorthand and strict inputs are parsed
- **THEN** warning codes and inference decisions match pre-refactor behavior
- **AND** no new warning categories are introduced by the modularization itself

#### Scenario: Existing fixture behavior remains equivalent
- **WHEN** existing TimeControl fixtures and SQL tests are executed after modularization
- **THEN** normalized outputs and JSON parse fields remain equivalent to prior behavior
