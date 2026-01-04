# Design: Position-Based Visitor Using Shakmaty

## Overview
This design replaces the string buffer-based movetext accumulation with shakmaty::Chess position tracking for move validation and movetext generation. As part of this change, comments are also preserved during movetext reconstruction (previously deferred).

## Data Structures

### GameVisitor Fields
```rust
pub struct GameVisitor {
    headers: Vec<(String, String)>,           // Existing
    position: Chess,                           // NEW: replaces movetext_buffer
    moves: Vec<SanPlus>,                       // NEW: validated moves
    comments: Vec<(usize, String)>,            // NEW: (ply, formatted_comment)
    result_marker: Option<String>,             // NEW: "1-0", "0-1", "1/2-1/2", "*"
    error_message: Option<String>,             // NEW: validation errors
    validation_failed: bool,                   // NEW: error flag
    game_number: usize,                        // NEW: for error reporting
    pub current_game: Option<GameRecord>,      // Existing
}
```

### Comment Storage Strategy

Comments are stored as `(ply, formatted_comment)` tuples where:
- **ply**: Position in the move sequence (0 = before first move, 1 = after first move, etc.)
- **formatted_comment**: Complete comment string with braces and whitespace: `" { comment text }"`

This allows comments to be:
1. Captured during parsing at any position
2. Stored independently from move validation
3. Reconstructed at their exact original positions during movetext generation

## Implementation Flow

### 1. Comment Capture (during parsing)

```rust
fn comment(&mut self, _movetext: &mut Self::Movetext, comment: RawComment) -> ControlFlow<Self::Output> {
    let ply = self.moves.len();  // Current position in move sequence
    let comment_text = String::from_utf8_lossy(comment.as_bytes());
    let formatted = format!(" {{ {} }}", comment_text.trim());
    self.comments.push((ply, formatted));
    ControlFlow::Continue(())
}
```

**Key Points:**
- Ply is determined by current `moves.len()` (number of moves processed so far)
- Comment text is extracted and reformatted with braces
- Leading space ensures proper spacing in final output

### 2. Move Validation (existing shakmaty pattern)

```rust
fn san(&mut self, movetext: &mut Self::Movetext, san_plus: SanPlus) -> ControlFlow<Self::Output> {
    match san_plus.san.to_move(movetext) {
        Ok(m) => {
            movetext.play_unchecked(&m);
            self.moves.push(san_plus);
            ControlFlow::Continue(())
        }
        Err(err) => {
            // Store error with context (move, ply, FEN)
            self.validation_failed = true;
            // Continue parsing to capture partial game
            ControlFlow::Continue(())
        }
    }
}
```

### 3. Movetext Reconstruction

```rust
fn generate_movetext(&self) -> String {
    let mut output = String::new();
    
    // Check for comments before first move (ply 0)
    for (ply, comment) in &self.comments {
        if *ply == 0 {
            output.push_str(comment);
        }
    }
    
    // Generate moves with interleaved comments
    for (i, san_plus) in self.moves.iter().enumerate() {
        // Add spacing
        if i > 0 || !output.is_empty() {
            output.push(' ');
        }
        
        // Add move number for white's moves
        if i % 2 == 0 {
            output.push_str(&format!("{}. ", (i / 2) + 1));
        }
        
        // Add move
        output.push_str(&san_plus.to_string());
        
        // Add any comments at this ply
        let ply = i + 1;
        for (comment_ply, comment) in &self.comments {
            if *comment_ply == ply {
                output.push_str(comment);
            }
        }
    }
    
    // Add result marker
    if let Some(ref marker) = self.result_marker {
        if !output.is_empty() {
            output.push(' ');
        }
        output.push_str(marker);
    }
    
    output
}
```

## Comment Position Mapping

| PGN Input | Ply | Comment Storage |
|-----------|-----|-----------------|
| `{ opening comment } 1. e4` | 0 | (0, " { opening comment }") |
| `1. e4 { best move }` | 1 | (1, " { best move }") |
| `1. e4 e5 { response }` | 2 | (2, " { response }") |
| `1. e4 e5 2. Nf3 { developing }` | 3 | (3, " { developing }") |

## Examples

### Example 1: Simple Comment
**Input PGN:**
```
1. e4 { best by test } e5 1-0
```

**Processing:**
1. `san()` called for "e4" → `moves = [e4]`, position updated
2. `comment()` called → `comments = [(1, " { best by test }")]`
3. `san()` called for "e5" → `moves = [e4, e5]`, position updated
4. `outcome()` called → `result_marker = "1-0"`
5. `generate_movetext()` → `"1. e4 { best by test } e5 1-0"`

**Output:** Identical to input ✓

### Example 2: Lichess Annotations
**Input PGN:**
```
1. d4 { [%eval 0.25] [%clk 1:30:43] } Nf6 { [%eval 0.22] [%clk 1:30:42] }
```

**Processing:**
1. `moves = [d4]`, `comments = [(1, " { [%eval 0.25] [%clk 1:30:43] }")]`
2. `moves = [d4, Nf6]`, `comments = [..., (2, " { [%eval 0.22] [%clk 1:30:42] }")]`
3. Output: Exact match to input ✓

### Example 3: Comment Before First Move
**Input PGN:**
```
{ opening comment } 1. e4 e5
```

**Processing:**
1. `comment()` called before any moves → `comments = [(0, " { opening comment }")]`
2. Moves processed normally
3. `generate_movetext()` prepends ply-0 comment
4. Output: `"{ opening comment } 1. e4 e5"` ✓

## Error Handling

Comments are preserved even when move validation fails:

**Input PGN with illegal move:**
```
1. e4 { good } e5 2. Nf7 { illegal! } Nc6
```

**Processing:**
1. Moves: e4, e5 validated ✓
2. Comments: (1, " { good }") stored
3. Nf7 validation fails → error captured, parsing continues
4. Comments: (2, " { illegal! }") still stored
5. Output movetext includes partial moves + all comments
6. `parse_error` field contains validation error with context

## Benefits

1. **Exact PGN Preservation**: Output matches input character-for-character for valid games
2. **Independent Storage**: Comments don't interfere with move validation
3. **Error Resilience**: Comments preserved even when moves are invalid
4. **Annotation Support**: All comment types ([%eval], [%clk], text) preserved
5. **Position Flexibility**: Comments can appear anywhere in move sequence
6. **Backward Compatible**: No breaking changes to output schema

## Testing Strategy

1. **Unit Tests**:
   - Comment after single move
   - Multiple comments in sequence
   - Comments before first move
   - Lichess annotations
   - Empty games with comments

2. **Integration Tests**:
   - Verify `sample.pgn` output matches input
   - Test with real Lichess game data
   - Verify existing tests still pass

3. **Edge Cases**:
   - Comments with illegal moves
   - Comments in games with FEN tags
   - Games with only comments (no moves)
   - Unicode in comments
   - Nested braces in comments (if supported by pgn-reader)

## Performance Impact

- **Memory**: Negligible - comments are small strings
- **CPU**: Minimal - simple vector operations
- **Complexity**: O(n) for movetext generation where n = moves + comments

## Migration Path

No breaking changes:
- Existing output schema unchanged
- Movetext format remains valid PGN
- Only difference: comments now preserved (previously lost)
