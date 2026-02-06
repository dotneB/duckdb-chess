## ADDED Requirements

### Requirement: Subset Query Performance Guidance
The project SHALL document practical query patterns for subset checks on large datasets, distinguishing semantic-correctness-first workflows from optimization-first workflows.

#### Scenario: Raw or noisy movetext guidance
- **WHEN** users work with movetext that may include comments, variations, NAGs, or inconsistent formatting
- **THEN** documentation recommends `chess_moves_subset(short, long)` as the default semantic-safe approach

#### Scenario: Canonicalized movetext guidance
- **WHEN** users materialize canonical movetext with `chess_moves_normalize` (or equivalent trusted preprocessing)
- **THEN** documentation includes a prefix-filter pattern using `starts_with(canonical_long, canonical_short)` for repeated large-scale filtering

#### Scenario: Contains warning
- **WHEN** users review subset examples
- **THEN** documentation explicitly warns that `contains` is not equivalent to subset-prefix semantics

#### Scenario: Result marker and normalization caveats
- **WHEN** users compare subset patterns in documentation
- **THEN** examples include caveats about result markers and normalization prerequisites so query semantics remain correct
