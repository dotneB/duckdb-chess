## 1. Create Macro

- [x] 1.1 Create `pgn_visitor_skip_variations!` macro in `src/chess/visitor.rs`
- [x] 1.2 Export macro for use in other modules (`pub use`)

## 2. Apply to Visitors in moves.rs

- [x] 2.1 Replace boilerplate in `MovesJsonVisitor` with macro invocation
- [x] 2.2 Replace boilerplate in `PlyCountVisitor` with macro invocation
- [x] 2.3 Replace boilerplate in `ZobristHashVisitor` with macro invocation

## 3. Apply to Visitors in filter.rs

- [x] 3.1 Replace boilerplate in `NormalizeSerializeVisitor` with macro invocation
- [x] 3.2 Replace boilerplate in `NormalizeVisitor` with macro invocation

## 4. Apply to GameVisitor

- [x] 4.1 ~~Replace boilerplate in `GameVisitor` with macro invocation~~ **SKIPPED**: GameVisitor has a custom `comment` method that preserves comments in movetext. Applying the macro would override this functionality. Only the `begin_variation` method matches the boilerplate pattern.

## 5. Verification

- [x] 5.1 Run `just dev` to verify lint, build, and tests pass
- [x] 5.2 Run `just full` to verify release build and tests pass