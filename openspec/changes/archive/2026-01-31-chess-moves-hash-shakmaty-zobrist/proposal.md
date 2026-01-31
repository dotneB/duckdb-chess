## Why

`chess_moves_hash` currently uses a custom hashing approach that diverges from standard chess tooling and produces values that are hard to validate against other libraries.
Switching to shakmaty's Zobrist hashing makes the hash deterministic, well-defined, and easier to test and reason about.

## What Changes

- Update `chess_moves_hash(movetext)` to compute the Zobrist hash of the final `shakmaty::Position` reached by applying the mainline moves in `movetext`.
- Parse movetext with a `pgn-reader` Visitor that incrementally updates both the `Position` and the running Zobrist hash as moves are processed.
- Reuse/extend the existing movetext parsing/visitor infrastructure (and `ParsedMovetext` if applicable) so hashing shares the same parsing rules as other `chess_moves_*` functions.
- Define NULL semantics: `chess_moves_hash(NULL)` and `chess_moves_hash('')` return `NULL`.
- Update tests to assert against shakmaty's Zobrist hash results (old expectations are intentionally discarded).

## Capabilities

### New Capabilities

<!-- None. -->

### Modified Capabilities

- `move-analysis`: Redefine `chess_moves_hash` as the Zobrist hash of the final position (transpositions may collide by design).

## Impact

- SQL API: `chess_moves_hash` return values will change for all inputs.
- SQL API: `chess_moves_hash` return type changes to `UBIGINT`.
- Implementation: movetext parsing code will gain a stateful Visitor that applies moves to a `shakmaty::Position`.
- Tests: unit and/or SQLLogicTests covering `chess_moves_hash` must be updated to use shakmaty-derived expected values.
