## Context

The `HeaderFields` struct stores parsed PGN headers as `Option<String>` fields. In `build_game_record`, each field is cloned when transferring to `GameRecord`:

```rust
event: self.headers.event.clone(),
site: self.headers.site.clone(),
source: self.headers.source.clone(),
// ... 15+ more clones
```

This pattern causes ~15 heap allocations per game parsed, even though the source strings will never be used again (the visitor clears headers before each new game).

## Goals / Non-Goals

**Goals:**
- Eliminate heap allocations when transferring string fields to `GameRecord`
- Maintain identical external behavior (no API changes)
- Preserve empty-vs-present semantics

**Non-Goals:**
- Changes to `GameRecord` struct (must remain `Option<String>` for downstream compatibility)
- Any modifications to parsing logic or error handling

## Decisions

### Replace `Option<String>` with `String` in `HeaderFields`

**Rationale:**
- Empty string can represent "not set" - no semantic loss
- Enables zero-copy transfer via `mem::take()`
- Simplifies `clear()` - just assign `Self::default()`

**Alternatives considered:**
- Keep `Option<String>` but use `mem::take()` - requires mapping `None` to empty string anyway for `GameRecord`
- Use `Cow<'static, str>` - overengineered for this use case

### Use helper method for Option conversion

Create `HeaderFields::opt_take(&mut self, field: &mut String) -> Option<String>`:
- Returns `None` if empty, `Some(taken_value)` otherwise
- Replaces the field with empty string
- Centralizes the empty-string-means-None logic

## Risks / Trade-offs

**Empty string vs None confusion** → Helper method encapsulates the convention; tests verify behavior.

**Slightly more complex `set_known_tag`** → Must check `is_empty()` instead of `is_none()`; negligible impact.