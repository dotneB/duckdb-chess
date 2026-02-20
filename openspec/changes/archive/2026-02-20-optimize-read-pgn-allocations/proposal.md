## Why

`read_pgn` is a high-throughput hot path, and the current visitor pays avoidable allocation costs: it decodes/allocates header tag values even for unknown tags and duplicates, and it clones movetext during final trimming even when no trimming is needed. These costs compound when scanning large PGN corpora.

## What Changes

- In `src/chess/visitor.rs`, only decode/allocate header tag values for known output tags, and only when the destination field is unset (skip unknown tags and duplicates without allocating).
- In `build_game_record()`, avoid an unconditional `trim()`-driven clone of movetext; only allocate a trimmed copy when trimming would change the string.
- Preserve all SQL-visible behavior and output formatting (schema, tag selection semantics, movetext formatting, error handling).
- Validate behavior parity with `just test`.

## Capabilities

### New Capabilities
- None.

### Modified Capabilities
- `allocation-efficiency`: Add explicit non-functional requirements for low-allocation tag processing and movetext finalization in the `read_pgn` visitor while preserving existing semantics.

## Impact

- Affected code: `src/chess/visitor.rs` (tag visitation + record building).
- User-facing SQL contract: unchanged.
- Performance: fewer transient allocations in header parsing and record finalization.
