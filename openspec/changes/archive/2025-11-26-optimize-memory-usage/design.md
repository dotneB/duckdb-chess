## Context
The current `read_pgn` implementation is serial and memory-heavy. To support large datasets efficiently, we need a streaming approach. To support high throughput on multi-core systems (a requested feature), we must allow concurrent processing of multiple files.

## Goals
- **Constant Memory Usage**: Fixed memory footprint regardless of dataset size.
- **Parallelism**: Utilize multiple CPU cores when reading multiple files.
- **Thread Safety**: Robust synchronization for the shared file queue.

## Design
### Architecture: The "Reader Pool"
Instead of a single serial cursor, we use a pool of active readers. DuckDB calls `func` concurrently from multiple worker threads.

1.  **Shared State (`ReadPgnInitData`)**:
    - `next_path_idx: AtomicUsize`: Index of the next file to be opened.
    - `readers: Mutex<Vec<PgnReaderState>>`: A pool of currently open but *idle* readers (waiting for a thread to pick them up).

2.  **Worker Logic (`func`)**:
    - **Acquire Work**:
        - Lock mutex.
        - **Option A**: Pop an existing reader from `readers`.
        - **Option B**: If no readers in pool, check `next_path_idx`. If `< paths.len()`, open new file, increment index, and return new reader.
        - **Option C**: If no readers and no new files, return `None` (Thread is done).
        - Unlock mutex.
    - **Process (Parallel Region)**:
        - If work acquired:
            - Parse chunk (2048 rows) using the private reader.
            - *Critical*: This happens outside the lock, allowing N threads to parse N files simultaneously.
    - **Return Work**:
        - Lock mutex.
        - If reader finished (EOF): Drop it.
        - If reader has more data: Push back to `readers` pool.
        - Unlock mutex.

### State Structures
```rust
struct ReadPgnInitData {
    // Guards access to the pool
    mutex: Mutex<SharedState> 
}

struct SharedState {
    next_path_idx: usize,
    // Readers not currently held by any thread
    available_readers: Vec<PgnReaderState> 
}

struct PgnReaderState {
    reader: BufReader<File>,
    path_idx: usize, // For error reporting
    // Reusable buffers
    game_buffer: String,
    record_buffer: GameRecord,
    line_buffer: Vec<u8>
}
```

## Trade-offs
- **Single File Bottleneck**: This design parallelizes *across files*. A single huge file will still be processed by one thread. This is acceptable as splitting variable-length PGN records without an index is complex and error-prone.
- **Ordering**: Output order is non-deterministic between files. This is standard for parallel SQL queries.
- **Complexity**: Slightly higher complexity to manage the pool vs simple serial loop.

## Open Questions
- **DuckDB Thread Scheduling**: DuckDB might retire a thread if it returns 0 rows. This logic (Option C) correctly handles that—once work is exhausted, threads retire.
