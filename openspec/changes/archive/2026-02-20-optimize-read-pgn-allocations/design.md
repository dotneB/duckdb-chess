## Context

`read_pgn` is implemented as a streaming PGN reader whose visitor (`src/chess/visitor.rs`) accumulates header tags and movetext into a per-game record. In the current hot path, the visitor performs avoidable work:

- Header tag values are decoded/allocated even when the tag is not used by the output schema.
- Duplicate tags may still trigger decoding/allocation even though only one value is semantically used.
- `build_game_record()` trims movetext by creating a new owned string unconditionally, even when the assembled movetext already has no leading/trailing whitespace.

This change is an internal optimization: output (columns, formatting, error semantics) must remain identical.

## Goals / Non-Goals

**Goals:**
- Reduce per-game heap allocations in the visitor hot path (tags + record finalization).
- Preserve all SQL-visible behavior, including duplicate-tag semantics, movetext formatting, and `parse_error` behavior.
- Keep the visitor code straightforward to extend when new output tags are added.

**Non-Goals:**
- No user-facing schema or behavior changes.
- No changes to globbing, file I/O, chunking, or malformed-game continuation.
- No new dependencies or architectural refactors outside `src/chess/visitor.rs`.

## Decisions

- **Known-tag fast path:** Implement a tight match over the known output tag names, and only process values for those tags.
- **Allocate only when needed:** For each known tag, only decode/allocate the tag value when the destination field that drives output is currently unset. This avoids allocations for duplicates that do not affect output.
- **Skip unknown tags entirely:** Unknown/unsupported tags are ignored without copying or storing.
- **Movetext trim without unconditional clone:** In `build_game_record()`, compute `trimmed = movetext.trim()`. If trimming would not change the string, move the original `String` into the record; otherwise allocate `trimmed.to_string()` to preserve exact trimming semantics.
- **Behavior lock-in via tests:** Rely on existing unit + SQLLogicTests for broad parity, and add small targeted tests for duplicate-tag handling and movetext whitespace finalization to prevent accidental semantic drift.

## Risks / Trade-offs

- **Duplicate-tag semantics mismatch:** If the current behavior is not "first value wins", gating on "destination field unset" could change output. Mitigation: verify current behavior before switching, and implement allocation avoidance in a way that preserves the existing winner policy.
- **Maintainability vs micro-optimizations:** A hand-written match table is slightly more code than generic tag storage, but it makes performance intent explicit and keeps output mapping localized.
