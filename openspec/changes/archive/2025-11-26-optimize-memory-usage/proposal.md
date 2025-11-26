# Change: Optimize Memory Usage & Throughput

## Why
The current implementation of `read_pgn` reads all PGN files entirely into memory (into a `Vec<GameRecord>`) during the first call, causing OOM on large datasets and blocking parallel execution. The user explicitly requested strategies to minimize memory usage and leverage multiple cores when processing multiple files.

## What Changes
- Refactor `read_pgn` to use a **Parallel Streaming** architecture.
- Implement a **Reader Pool** in the shared state:
    - Threads dynamically claim files (or in-progress readers) from a shared queue.
    - Parsing happens locally on the thread (holding the reader exclusively).
    - Readers are returned to the pool after filling a chunk.
- Introduce a **reusable `GameRecord` buffer** to minimize heap allocations.
- Eliminate `Vec<GameRecord>` storage.

## Impact
- **Affected Specs**: `pgn-parsing`
- **Affected Code**: `src/lib.rs` (State management and `func` logic).
- **Performance**: 
    - **Memory**: Constant `O(1)` per thread (Chunk Size + File Buffer).
    - **Throughput**: Scales linearly with cores for multi-file datasets (N files >= M cores).
- **Behavior**: Order of results is non-deterministic across files (standard for parallel scans) but deterministic within files.
