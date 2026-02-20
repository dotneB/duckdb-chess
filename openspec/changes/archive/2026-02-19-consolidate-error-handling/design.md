## Context

Two locations implement identical error accumulation logic:

**visitor.rs:104-114** - `GameVisitor::push_error`:
```rust
fn push_error(parse_error: &mut Option<String>, msg: String) {
    match parse_error {
        Some(existing) => {
            existing.push_str("; ");
            existing.push_str(&msg);
        }
        None => {
            *parse_error = Some(msg);
        }
    }
}
```

**reader.rs:137-147** - `append_parse_error`:
```rust
fn append_parse_error(parse_error: &mut Option<String>, msg: &str) {
    match parse_error {
        Some(existing) => {
            existing.push_str("; ");
            existing.push_str(msg);
        }
        None => {
            *parse_error = Some(msg.to_string());
        }
    }
}
```

Both functions have the same structure and behavior but slightly different signatures (`String` vs `&str`).

## Goals / Non-Goals

**Goals:**
- Consolidate duplicate error accumulation logic into a single reusable type
- Maintain identical error formatting behavior (`"; "` separator)
- Make the API ergonomic for both `String` and `&str` inputs

**Non-Goals:**
- Changes to what errors are reported or when
- Changes to error message content
- Any modifications to public interfaces

## Decisions

### Create `ErrorAccumulator` struct with `Option<String>` storage

**Rationale:**
- Wrapping `Option<String>` provides a clear type for error accumulation
- Zero-cost abstraction - `Option<String>` is the same size as the wrapper
- Can implement `Default` for easy construction

**Alternatives considered:**
- **Trait extension on `Option<String>`**: Less discoverable, harder to import
- **Free function only**: Doesn't provide as clear an API boundary

### Accept `impl Into<String>` or `&str` for flexibility

**Rationale:**
- `&str` is the most common case (literal strings, borrowed strings)
- `impl Into<String>` allows owned strings without extra allocation
- Matches the existing usage patterns in both files

**API Design:**
```rust
pub struct ErrorAccumulator(Option<String>);

impl ErrorAccumulator {
    pub fn push(&mut self, msg: &str) { /* ... */ }
    pub fn take(&mut self) -> Option<String> { /* ... */ }
    pub fn is_empty(&self) -> bool { /* ... */ }
}
```

## Risks / Trade-offs

**New module/file** → Adds one small file; keeps error handling utilities centralized.

**Slight API change for callers** → Minimal; callers replace `push_error(&mut err, msg)` with `err.push(msg)`.