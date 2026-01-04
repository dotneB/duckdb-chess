# Implementation Tasks

## 1. Update Visitor Trait Associated Types
- [x] 1.1 Change `type Movetext = String` to `type Movetext = Chess` in `GameVisitor`
- [x] 1.2 Change `type Tags = ()` to `type Tags = Option<Chess>` to support FEN tags
- [x] 1.3 Change `type Output = ()` to `type Output = bool` to track validation success/failure
- [x] 1.4 Verify `GameVisitor` struct fields:
  - [x] 1.4.1 Keep `headers: Vec<(String, String)>` for header tag collection
  - [x] 1.4.2 Add `moves: Vec<SanPlus>` to store validated moves (for movetext reconstruction)
  - [x] 1.4.3 Add `comments: Vec<(usize, String)>` to store comments with their ply positions
  - [x] 1.4.4 Add `result_marker: Option<String>` to store game result (1-0, 0-1, etc.)
  - [x] 1.4.5 Add `error_message: Option<String>` to capture validation errors
  - [x] 1.4.6 Add `validation_failed: bool` flag to track if any move validation failed
  - [x] 1.4.7 Add `game_number: usize` to track which game is being parsed
- [x] 1.5 Remove `movetext_buffer: String` field (no longer needed)
- [x] 1.6 Remove `move_count: u32` field (position tracks this)
- [x] 1.7 Update `GameVisitor::new()` to initialize new fields

## 2. Update `begin_tags()` Method
- [x] 2.1 Reset `game_number` counter
- [x] 2.2 Clear `headers` vector
- [x] 2.3 Clear `moves` vector
- [x] 2.4 Clear `comments` vector
- [x] 2.5 Clear `result_marker`
- [x] 2.6 Clear `error_message`
- [x] 2.7 Set `validation_failed` to false
- [x] 2.8 Return `ControlFlow::Continue(None)` to signal no FEN tag yet

## 3. Update `tag()` Method (Support FEN Tags)
- [x] 3.1 Add FEN tag handling following shakmaty example pattern:
  - [x] 3.1.1 Check if tag name is "FEN"
  - [x] 3.1.2 Parse FEN string using `Fen::from_ascii()`
  - [x] 3.1.3 Convert FEN to Chess position using `fen.into_position(CastlingMode::Chess960)`
  - [x] 3.1.4 Store position in Tags parameter (`*tags = Some(pos)`)
  - [x] 3.1.5 Handle errors: capture in error_message and return error if FEN is invalid
- [x] 3.2 Continue storing all other tags in headers vector

## 4. Update `begin_movetext()` Method
- [x] 4.1 Return `ControlFlow::Continue(tags.unwrap_or_default())`
- [x] 4.2 If FEN tag was present, use that position; otherwise use `Chess::default()`
- [x] 4.3 Clear `moves` vector for new game

## 5. Update `san()` Method (Move Validation)
- [x] 5.1 Validate move against position using `san_plus.san.to_move(movetext)`
- [x] 5.2 On success:
  - [x] 5.2.1 Call `movetext.play_unchecked(m)` to update position
  - [x] 5.2.2 Add `san_plus` to `moves` vector (for later movetext reconstruction)
  - [x] 5.2.3 Return `ControlFlow::Continue(())`
- [x] 5.3 On failure (illegal move):
  - [x] 5.3.1 Calculate ply number: `ply = moves.len() + 1`
  - [x] 5.3.2 Generate FEN from current position using `Fen::from_position(movetext, EnPassantMode::Legal)`
  - [x] 5.3.3 Store error message: `format!("Illegal move '{}' at ply {} from position {}: {}", san_plus, ply, fen, err)`
  - [x] 5.3.4 Set `validation_failed` flag to `true`
  - [x] 5.3.5 Return `ControlFlow::Continue(())` (don't break, continue parsing to capture partial game)

## 6. Update `comment()` Method
- [x] 6.1 Get current ply position: `ply = moves.len()`
- [x] 6.2 Extract comment text from `RawComment` using `String::from_utf8_lossy(comment.as_bytes())`
- [x] 6.3 Format comment with braces and whitespace: `format!(" {{ {} }}", comment_text.trim())`
- [x] 6.4 Store comment with ply position: `comments.push((ply, formatted_comment))`
- [x] 6.5 Return `ControlFlow::Continue(())`
- [x] 6.6 Note: Comments are associated with the ply they appear after (ply 0 = before first move, ply 1 = after first move, etc.)

## 7. Update `begin_variation()` Method
- [x] 7.1 Keep existing implementation: return `ControlFlow::Continue(Skip(true))`
- [x] 7.2 Variations continue to be skipped to maintain main line

## 8. Update `end_game()` Method
- [x] 8.1 Generate movetext from `moves` vector using `generate_movetext()`
- [x] 8.2 Check `validation_failed` flag
- [x] 8.3 If `validation_failed` is true:
  - [x] 8.3.1 Call `finalize_game_with_error(error_message.take().unwrap_or_default())`
  - [x] 8.3.2 Return `false` (validation failed)
- [x] 8.4 Otherwise:
  - [x] 8.4.1 Call `finalize_game()`
  - [x] 8.4.2 Return `true` (success)

## 9. Implement Movetext Generation
- [x] 9.1 Add `generate_movetext()` helper method that:
  - [x] 9.1.1 Create output string buffer
  - [x] 9.1.2 Check for comments at ply 0 (before first move) and append if present
  - [x] 9.1.3 Iterate through `moves` vector with index
  - [x] 9.1.4 For each move at index `i`:
    - [x] 9.1.4.1 Add space if not first move and no trailing comment
    - [x] 9.1.4.2 If white's move (even index): add move number "1. ", "2. ", etc.
    - [x] 9.1.4.3 Add move text using `san_plus.to_string()`
    - [x] 9.1.4.4 Check `comments` vector for any comments at ply `i + 1` and append them
  - [x] 9.1.5 After all moves, append `result_marker` if present (e.g., " 1-0")
  - [x] 9.1.6 Return complete movetext string
- [x] 9.2 Handle edge case: empty games (no moves) with or without result marker
- [x] 9.3 Handle edge case: comments at ply 0 (before any moves)
- [x] 9.4 Ensure spacing matches original PGN format (comments already include leading space)

## 10. Update `finalize_game()` Method
- [x] 10.1 Keep all header extraction logic unchanged
- [x] 10.2 Change `movetext` field assignment to use `generate_movetext()` result
- [x] 10.3 Ensure `parse_error` is `None` for successful games

## 11. Update `finalize_game_with_error()` Method
- [x] 11.1 Keep all header extraction logic unchanged
- [x] 11.2 Generate partial movetext using `generate_movetext()` (includes moves up to error)
- [x] 11.3 Store the detailed error message in `parse_error` field
- [x] 11.4 Ensure error messages include game number, move text, ply, and position FEN

## 12. Implement `outcome()` Method
- [x] 12.1 Add implementation for `outcome(&mut self, _movetext: &mut Self::Movetext, outcome: Outcome)`
- [x] 12.2 Convert Outcome to string representation using `outcome.to_string()` or `outcome.as_str()`
- [x] 12.3 Store the string in `result_marker` field
  - [x] 12.3.1 For `Known(KnownOutcome)` variants: "1-0", "0-1", or "1/2-1/2"
  - [x] 12.3.2 For `Unknown` variant: "*"
- [x] 12.4 Return `ControlFlow::Continue(())` to continue parsing
- [x] 12.5 Note: This method is called by pgn-reader when game termination markers are encountered

## 13. Fallback for Result Header
- [x] 13.1 In `end_game()`, check if `result_marker` is still `None`
- [x] 13.2 If no outcome was provided, check the `Result` header from tags
- [x] 13.3 Use the `Result` header value as `result_marker` if available
- [x] 13.4 This handles PGN files that only have the Result header without a termination marker

## 14. Error Handling Improvements
- [x] 14.1 Ensure FEN tag errors include game number and detailed context
- [x] 14.2 Ensure move validation errors include move text, ply number, position FEN
- [x] 14.3 Add helper to format errors consistently across the visitor
- [x] 14.4 Test that errors are captured but don't stop file parsing

## 15. Testing
- [x] 15.1 Update existing unit tests in `visitor.rs` to work with new approach
- [x] 15.2 Add test for legal move sequence validation
- [x] 15.3 Add test for illegal move detection and error reporting
- [x] 15.4 Add test for result marker preservation in movetext output
- [x] 15.5 Add test for `outcome()` method with different termination markers
- [x] 15.6 Add test for FEN tag support (non-standard starting positions)
- [x] 15.7 Add test for movetext generation from moves vector
- [x] 15.8 Add test for comment preservation (verify "1. e4 { best by test } e5" stays exactly the same)
- [x] 15.9 Add test for multiple comments in a game
- [x] 15.10 Add test for comments before first move (ply 0)
- [x] 15.11 Add test for Lichess-style comments with [%eval] and [%clk] annotations
- [x] 15.12 Verify all existing SQL integration tests still pass (especially test_visitor_with_comments)
- [x] 15.13 Add integration test for invalid movetext with proper error messages
- [x] 15.15 Verify that movetext output exactly matches input PGN for valid games

## 16. Documentation
- [x] 16.1 Update code comments in `visitor.rs` to reference shakmaty example pattern
- [x] 16.2 Add doc comments explaining Chess as Movetext associated type
- [x] 16.3 Add doc comments explaining move validation and error handling
- [x] 16.4 Add doc comments explaining `outcome()` method implementation
- [x] 16.5 Add doc comments explaining comment preservation strategy (ply-based tracking)
- [x] 16.6 Update spec references in code comments
- [x] 16.7 Add reference to official shakmaty validation example URL
- [x] 16.8 Document comment formatting rules (preserving braces and whitespace)