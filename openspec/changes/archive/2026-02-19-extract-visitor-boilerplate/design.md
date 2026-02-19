## Context

Six visitor implementations share identical boilerplate for methods that:
1. Ignore NAGs (`nag`)
2. Ignore comments (`comment`, `partial_comment`)
3. Skip variations (`begin_variation` returns `Skip(true)`)

Affected visitors:
- `moves.rs`: `MovesJsonVisitor`, `PlyCountVisitor`, `ZobristHashVisitor`
- `filter.rs`: `NormalizeSerializeVisitor`, `NormalizeVisitor`
- `visitor.rs`: `GameVisitor`

Each visitor repeats ~12 lines of identical method implementations.

## Goals / Non-Goals

**Goals:**
- Eliminate ~50 lines of repetitive code across 6 visitors
- Make adding new visitors simpler (single macro invocation)
- Centralize the "skip variations, ignore annotations" behavior

**Non-Goals:**
- Changes to visitor behavior or parsing logic
- Changes to the `pgn-reader` crate or its `Visitor` trait
- Any modifications to public interfaces

## Decisions

### Use declarative macro (`macro_rules!`)

**Rationale:**
- No external dependencies required
- Zero runtime cost (expands at compile time)
- Simple to understand and maintain
- Works with the trait implementation pattern

**Alternatives considered:**
- **Trait with default methods**: Rust traits don't support default implementations that call other trait methods with the same signature pattern needed here
- **Base struct with delegation**: More complex, requires storing the visitor state separately
- **Proc macro**: Overkill for this use case, adds build complexity

### Macro design: `impl_pgn_visitor_boilerplate!($type, $output, $movetext)`

The macro generates the four boilerplate methods with the correct associated types:

```rust
macro_rules! impl_pgn_visitor_boilerplate {
    ($ty:ty, $out:ty, $mov:ty) => {
        impl pgn_reader::Visitor for $ty {
            // ... generated methods
        }
    };
}
```

Actually, since each visitor already has a custom `Visitor` impl with different associated types and custom `san`, `begin_tags`, etc., a better approach is a macro that generates just the boilerplate methods:

```rust
macro_rules! pgn_visitor_skip_variations {
    () => {
        fn nag(&mut self, _: &mut Self::Movetext, _: Nag) -> ControlFlow<Self::Output> {
            ControlFlow::Continue(())
        }
        fn comment(&mut self, _: &mut Self::Movetext, _: RawComment<'_>) -> ControlFlow<Self::Output> {
            ControlFlow::Continue(())
        }
        fn partial_comment(&mut self, _: &mut Self::Movetext, _: RawComment<'_>) -> ControlFlow<Self::Output> {
            ControlFlow::Continue(())
        }
        fn begin_variation(&mut self, _: &mut Self::Movetext) -> ControlFlow<Self::Output, Skip> {
            ControlFlow::Continue(Skip(true))
        }
    };
}
```

## Risks / Trade-offs

**Macro hygiene/debugging** → Macro is simple and well-documented; any errors will be obvious at compile time.

**IDE support** → rust-analyzer handles simple macros well; no significant impact on developer experience.