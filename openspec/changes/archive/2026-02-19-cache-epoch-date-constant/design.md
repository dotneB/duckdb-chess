## Context

The `parse_date_field` method in `GameVisitor` computes the Unix epoch date (`1970-01-01`) on every invocation to calculate days-since-epoch for `duckdb_date` conversion. With millions of games in large PGN datasets, this repeated construction wastes CPU cycles on a constant value.

Current implementation (`visitor.rs:320-330`):
```rust
let epoch = match NaiveDate::from_ymd_opt(1970, 1, 1) {
    Some(v) => v,
    None => {
        Self::push_error(parse_error, "Conversion error: failed to create epoch".into());
        return None;
    }
};
```

The `match` error branch is dead code - `NaiveDate::from_ymd_opt(1970, 1, 1)` always succeeds.

## Goals / Non-Goals

**Goals:**
- Eliminate redundant epoch date construction on every date parse
- Remove unreachable error-handling code
- Maintain identical behavior (no API changes)

**Non-Goals:**
- Changes to date parsing logic or error handling
- Any modifications to public interfaces or SQL behavior

## Decisions

### Use `LazyLock<NaiveDate>` for epoch constant

**Rationale:**
- `LazyLock` is stable since Rust 1.80 (project MSRV is 1.89)
- Thread-safe, one-time initialization
- Zero runtime cost after initialization
- No external dependencies required

**Alternatives considered:**
- `OnceLock<NaiveDate>` - equally valid, but `LazyLock` is more idiomatic for computed values
- `const fn` - not possible since `NaiveDate::from_ymd_opt` is not const
- Inline computation - current approach, wasteful

## Risks / Trade-offs

**Static initialization overhead** → Negligible; one-time cost at extension load time, dwarfed by parsing work.

**Added import** → Minimal; `std::sync::LazyLock` already used elsewhere in the codebase (`timecontrol.rs`).