## Context

Move lists in `filter.rs` (`ParsedMovetext::sans`) and `moves.rs` visitors use `Vec<String>`, causing heap allocations for every game. Typical chess games have 40-80 moves, meaning nearly every game triggers at least one heap allocation when the vector grows beyond its initial capacity.

## Goals / Non-Goals

**Goals:**
- Eliminate heap allocations for typical games (≤64 moves)
- Maintain identical API and behavior
- Use a well-maintained, zero-dependency crate

**Non-Goals:**
- Changes to parsing logic or output format
- Modifications to public interfaces
- Optimizing other data structures (only move lists)

## Decisions

### Use `smallvec` crate with inline capacity of 128

**Rationale:**
- 128 moves covers ~95% of chess games based on real data (17.6M game dataset, avg 83 plies)
- `smallvec` is widely used (part of Servo project), zero dependencies, well-maintained
- API is nearly identical to `Vec` - minimal code changes
- Falls back to heap automatically for longer games

**Data-driven capacity selection:**
- Analyzed 17,645,043 games with average 83 plies
- Peak distribution at 55-80 plies (mode ~69)
- Capacity 128 covers games up to ~ply 128 without heap allocation

**Alternatives considered:**
- **`arrayvec`**: Requires fixed capacity, panics on overflow - less flexible
- **`tinyvec`**: Requires `Default` for elements - `String` doesn't implement it
- **Pre-allocated `Vec::with_capacity(64)`**: Still heap allocates, just reduces reallocations

### Type alias for move list

```rust
type MoveList = SmallVec<[String; 128]>;
```

**Rationale:**
- Centralizes the capacity choice
- Makes intent clear
- Easy to adjust capacity later if needed

## Risks / Trade-offs

**Increased stack size** → Each visitor struct grows by ~1KB (128 empty String slots). Acceptable trade-off for heap elimination on 95% of games.

**Games > 128 moves** → Fall back to heap allocation seamlessly. No behavioral change. (~5% of games based on data)

**New dependency** → `smallvec` is stable, well-maintained, and has no transitive dependencies.