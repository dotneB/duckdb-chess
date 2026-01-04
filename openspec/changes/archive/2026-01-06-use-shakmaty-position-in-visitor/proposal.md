# Change: Use Shakmaty Position in Visitor Instead of Movetext Buffer

## Why
The current visitor implementation in `visitor.rs` maintains a string buffer for movetext and manually counts moves. This approach has several limitations:

1. **Manual counting is error-prone**: The `move_count` field requires careful maintenance and can get out of sync
2. **No move validation**: Invalid moves are only detected later during processing, not during parsing
3. **Missing error context**: When a move is invalid, we can't report which move failed or why
4. **Result marker handling**: The current approach removes the result marker (1-0, 0-1, 1/2-1/2, *) from movetext during processing, making the exported movetext invalid PGN

By maintaining a `shakmaty::Chess` position object instead, we can:
- Validate moves as they're parsed and report errors immediately
- Eliminate manual move counting (shakmaty tracks position state)
- Generate properly formatted movetext on demand
- Preserve the complete game result in movetext output
- Provide detailed error messages when moves are illegal
- Also preserve comments in movetext (previously deferred)

## What Changes
- **MODIFIED** `src/chess/visitor.rs`:
  - Replace `movetext_buffer: String` field with `position: Chess` field
  - Remove `move_count: u32` field (position state provides this)
  - Add `moves: Vec<SanPlus>` to store validated moves
  - Add `comments: Vec<(usize, String)>` to store comments with their ply positions
  - Update `san()` method to validate moves against current position and store them
  - Update `comment()` method to store comments (instead of ignoring them)
  - Add `result_marker: Option<String>` to preserve game result
  - Update movetext generation to rebuild from validated moves, include comments, and include result marker
  - Update error handling to report specific move validation failures

- **MODIFIED** pgn-parsing spec: Update visitor pattern requirements to reflect position-based parsing
- **MODIFIED** data-schema spec: Clarify that movetext includes result markers and preserves comments

## Impact
- Affected specs: `pgn-parsing`, `data-schema`
- Affected code: `src/chess/visitor.rs`
- Breaking change: **NO** - Output schema and behavior remain compatible
- Performance impact: Minimal - shakmaty position tracking is efficient
- Benefits:
  - Better error messages for invalid moves
  - Proper PGN movetext output with result markers
  - Comment preservation (previously deferred, now included)
  - More maintainable code without manual move counting
  - Foundation for future move validation features
