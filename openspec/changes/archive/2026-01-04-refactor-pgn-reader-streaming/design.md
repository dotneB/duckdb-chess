# Design: Refactor PGN Reader Streaming

## Context

The current PGN reader implementation (introduced in the initial extension) manually performs game boundary detection and buffering:

- **Double buffering**: `File` → `BufReader` → pgn-reader's internal `Buffer`
- **Manual splitting**: Reads line-by-line, detects `[Event ` headers, buffers entire game text, then passes to `Reader::new(game_bytes)`
- **Extra allocations**: `game_buffer: String` and `line_buffer: Vec<u8>` create unnecessary copies

This was likely done to maintain control over chunking (DuckDB's 2048-row limit), but the pgn-reader library already supports streaming with `read_game()` that:
- Has its own optimized internal buffer (16KB default, expandable)
- Correctly handles game boundaries through the Visitor pattern
- Works with any `Read` impl (including `File` directly)
- Returns `Ok(None)` on EOF for clean stream termination

**Stakeholders**: Performance-sensitive users processing large Lichess datasets (multi-GB PGN files).

**Constraints**:
- Must preserve DuckDB's chunking behavior (2048 rows per `func()` call)
- Must maintain parallel execution via reader pool
- Must preserve error handling (partial games with `parse_error` column)
- Cannot change SQL API or output schema

## Goals / Non-Goals

**Goals:**
- Eliminate double buffering by using `Reader<File>` directly
- Remove manual game boundary detection (trust pgn-reader's `read_game()`)
- Reduce memory allocations by removing intermediate string buffers
- Improve code maintainability by reducing ~400 lines of manual parsing logic
- Maintain or improve performance (fewer allocations, better buffering)

**Non-Goals:**
- Changing SQL API or schema (still 17 columns, still `read_pgn(path)`)
- Adding parallelism beyond DuckDB's built-in thread pool
- Supporting additional PGN formats or chess variants
- Modifying Visitor implementation (only how it's called)

## Decisions

### Decision 1: Use Reader<File> directly
**Choice**: Store `Reader<File>` in `PgnReaderState`, pass raw `File` to `Reader::new()`

**Rationale**:
- pgn-reader documentation explicitly states: "Buffers the underlying reader with an appropriate strategy, so it's *not* recommended to add an additional layer of buffering like `BufReader`"
- `Reader::new()` accepts any `R: Read` and handles buffering internally
- Simplifies ownership (no nested generics)

**Alternatives considered**:
- ❌ Keep `BufReader`: Wastes memory with redundant buffering
- ❌ Use `Reader<Box<dyn Read>>`: Unnecessary dynamic dispatch overhead
- ✅ Use `Reader<File>`: Monomorphized, zero overhead, follows library design

### Decision 2: Direct read_game() loop
**Choice**: Replace lines 166-592 with simple loop calling `read_game()`

**Before** (duct-taped):
```rust
loop {
    reader.line_buffer.clear();
    match reader.reader.read_until(b'\n', &mut reader.line_buffer) {
        Ok(0) => break, // EOF
        Ok(_) => {
            let line = String::from_utf8_lossy(&reader.line_buffer).into_owned();
            if line.starts_with("[Event ") {
                // Parse previous game_buffer...
                reader.game_buffer.clear();
            }
            reader.game_buffer.push_str(&line);
        }
    }
}
```

**After** (clean):
```rust
loop {
    match reader.pgn_reader.read_game(&mut reader.visitor) {
        Ok(Some(_)) => {
            if let Some(game) = reader.visitor.current_game.take() {
                reader.record_buffer = game;
                // Write to DuckDB output
                count += 1;
            }
        }
        Ok(None) => break, // EOF
        Err(e) => {
            // Error handling with finalize_game_with_error
        }
    }
    if count >= 2048 { break; } // Chunk limit
}
```

**Rationale**:
- pgn-reader's `read_game()` handles all boundary detection internally
- Visitor pattern already collects headers and movetext
- Simpler control flow, fewer error cases

### Decision 3: Preserve chunking at application level
**Choice**: Keep the `while count < 2048` loop in `func()`, return reader to pool at chunk boundary

**Rationale**:
- DuckDB table functions must yield control periodically
- pgn-reader's streaming design is compatible with chunking
- Reader state is preserved when returned to pool

**Alternatives considered**:
- ❌ Read entire file at once: Would break chunking, cause OOM on large files
- ❌ Use crossbeam channels: Adds complexity, fights DuckDB's parallelism model
- ✅ Simple loop with chunk limit: Maintains existing behavior, clean integration

### Decision 4: Error handling strategy
**Choice**: Catch `Err(e)` from `read_game()`, call `visitor.finalize_game_with_error()`, output partial game

**Rationale**:
- Preserves existing error handling contract (partial games with `parse_error` column)
- pgn-reader's errors include context (e.g., "unterminated tag")
- Maintains backward compatibility with existing queries

## Risks / Trade-offs

### Risk: Breaking change in error messages
**Mitigation**: 
- Test with `test/pgn_files/parse_errors.pgn` to verify error messages remain descriptive
- Accept minor wording changes in error messages (not a breaking API change)

### Risk: Performance regression
**Mitigation**:
- Benchmark with 100MB+ PGN file before/after
- Expected improvement, but if regression occurs, investigate buffer sizes
- pgn-reader's default 16KB buffer may need tuning (use `ReaderBuilder`)

### Risk: Subtle behavior change in game boundaries
**Mitigation**:
- Run full SQLLogicTest suite
- Verify game counts match on multi-game files
- Test edge cases (empty lines, malformed headers, EOF without newline)

### Trade-off: Less control over buffering
**Accept**: pgn-reader's buffer strategy is well-tested and optimized for PGN format

**Benefit**: Simpler code, fewer bugs from manual parsing

## Migration Plan

### Pre-deployment:
1. Create feature branch `refactor/pgn-reader-streaming`
2. Implement changes per `tasks.md`
3. Run full test suite (`cargo test` + SQLLogicTest)
4. Benchmark with Lichess dataset sample (compare throughput)

### Deployment:
1. Merge to main after approval
2. CI builds extension for all platforms
3. Local testing with `duckdb -unsigned` and real datasets

### Rollback:
- If critical regression found: revert commit (simple revert, no data migration needed)
- No database schema changes, so rollback is safe

### Validation:
- Query `SELECT COUNT(*) FROM read_pgn('test/pgn_files/*.pgn')` - verify count matches
- Query games with `parse_error IS NOT NULL` - verify error messages are descriptive
- Query large glob pattern - verify memory usage remains constant

## Performance Results

**Benchmarked on real-world Lichess datasets (LumbrasGigaBase):**

### Single File (1.2M games)
- Before: 15.47s - 21.25s (variation due to disk cache)
- After: 11.43s
- **Improvement: 26-46% faster**

### Multi-File Glob Pattern (4.5M games across multiple files)
- Before: 69.05s - 80.71s
- After: 43.58s
- **Improvement: 37-46% faster**

### Why the Improvement
1. Eliminated double buffering (no `BufReader` wrapper)
2. Removed intermediate string allocations (`game_buffer`, `line_buffer`)
3. pgn-reader's optimized 16KB buffer vs. manual line-by-line reading
4. No UTF-8 conversion on every line
5. No pattern matching (`[Event ` check) on every line

**Conclusion:** Performance risk was unfounded - refactoring provides significant speedup on large datasets.

## Open Questions

None - design is straightforward refactoring with clear alternatives and no ambiguity.
