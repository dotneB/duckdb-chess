# Change: Refactor PGN Reader to Use Direct Streaming

## Why

The current implementation in `src/chess/reader.rs` performs unnecessary double-buffering and manual game boundary detection:

1. **Double buffering**: Wraps `File` in `BufReader`, then the pgn-reader library adds its own internal buffering (as documented in its source: "Buffers the underlying reader with an appropriate strategy, so it's *not* recommended to add an additional layer of buffering")
2. **Manual game splitting**: Lines 166-592 manually read line-by-line to detect `[Event ` boundaries and buffer entire games into `game_buffer`, when pgn-reader's `Reader::read_game()` already handles game boundary detection through the Visitor pattern
3. **Unnecessary allocations**: `game_buffer` and `line_buffer` create extra string allocations and copies that could be avoided

This "duct-taped" approach reduces performance and maintainability. The pgn-reader library is designed to be used directly with `Reader::new(file).read_game()` for streaming PGN parsing.

## What Changes

- **Remove** manual line-by-line reading and game boundary detection (lines 166-592 in `src/chess/reader.rs`)
- **Remove** `BufReader` wrapper around `File` (pgn-reader handles buffering internally)
- **Remove** `game_buffer` and `line_buffer` fields from `PgnReaderState`
- **Add** `Reader<File>` field to `PgnReaderState` to hold the pgn-reader's streaming reader
- **Refactor** main parsing loop in `func()` to call `pgn_reader.read_game(&mut visitor)` directly
- **Simplify** EOF handling by relying on `read_game()` returning `Ok(None)`
- **Update** error handling to work with `read_game()`'s error types

This change preserves all existing functionality (error handling, chunking, parallelism) while eliminating unnecessary complexity.

## Impact

- **Affected specs**: `pgn-parsing` (modified requirements for implementation details)
- **Affected code**: 
  - `src/chess/reader.rs` (major simplification, ~400 lines removed)
  - `src/chess/visitor.rs` (`PgnReaderState` struct simplified)
- **Performance**: Expected improvement from reduced allocations and better buffering strategy
- **Compatibility**: No breaking changes to SQL API or output schema
- **Tests**: All existing tests should pass without modification (behavior unchanged)
