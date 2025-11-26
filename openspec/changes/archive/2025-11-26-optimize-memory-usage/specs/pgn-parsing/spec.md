## ADDED Requirements

### Requirement: Streaming & Parallel Execution
The PGN reader MUST use a streaming architecture that supports parallel processing across multiple files to maximize throughput on multi-core systems while maintaining constant memory usage.

#### Scenario: Multi-file parallel processing
- **GIVEN** a glob pattern matching multiple PGN files (e.g., `data/*.pgn`)
- **AND** the system has multiple CPU cores available
- **WHEN** `read_pgn` is executed
- **THEN** multiple files are processed concurrently
- **AND** the memory usage does not scale linearly with the dataset size

#### Scenario: Large dataset processing
- **GIVEN** a glob pattern matching files larger than available RAM
- **WHEN** `read_pgn` is executed
- **THEN** the query completes successfully without Out-Of-Memory errors
- **AND** the system reads files sequentially or concurrently in chunks
