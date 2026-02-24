## MODIFIED Requirements

### Requirement: Chunked Output
The system SHALL output parsed games in chunks to manage memory efficiently for large datasets, and the maximum rows per output chunk SHALL be derived from the runtime DuckDB output vector capacity.

#### Scenario: Large dataset processing
- **WHEN** parsing results in more games than the current DuckDB output vector capacity
- **THEN** the function outputs games in chunks of up to the runtime vector capacity per call

#### Scenario: Small dataset processing
- **WHEN** parsing results in fewer games than the current DuckDB output vector capacity
- **THEN** the function outputs all games in a single chunk

#### Scenario: Non-default DuckDB vector size
- **WHEN** DuckDB is configured with a vector size different from 2048
- **THEN** `read_pgn` uses that configured runtime vector capacity as the chunk upper bound
