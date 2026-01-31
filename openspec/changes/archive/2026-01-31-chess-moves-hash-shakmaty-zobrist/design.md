## Context

The extension exposes movetext utilities via `chess_moves_*` scalar functions. Today `chess_moves_hash(movetext)` hashes a canonicalized string of SAN tokens using Rust's `DefaultHasher`, which is:

- not a chess-defined hash (hard to validate externally), and
- not stable across Rust versions/platforms by contract.

We already depend on `shakmaty` for SAN parsing and move application (e.g. `chess_moves_json` builds FENs by applying moves). `shakmaty` also provides stable Zobrist hashing for positions.

## Goals / Non-Goals

**Goals:**

- Define `chess_moves_hash` in terms of `shakmaty`'s stable Zobrist hash of the final position reached by the parsed mainline movetext.
- Implement hashing using a `pgn-reader` `Visitor` so we can stream-parse movetext, skip variations, ignore comments/NAGs, and avoid intermediate allocations when possible.
- Reuse/extend existing movetext parsing utilities (`NormalizeVisitor` / `ParsedMovetext`) where it reduces duplication.
- Update tests to assert shakmaty-derived expected values.

**Non-Goals:**

- Computing a hash that uniquely identifies the move *sequence* (transpositions may intentionally collide).
- Changing the `chess_moves_json`, `chess_moves_normalize`, `chess_moves_subset`, or `chess_ply_count` semantics.
- Adding a new SQL function or changing function signatures.

## Decisions

- Use `shakmaty::zobrist::Zobrist64` via `Position::zobrist_hash::<Zobrist64>(EnPassantMode::Legal)`.
  - Rationale: documented stable hash; `EnPassantMode::Legal` matches common engine practice and avoids encoding ephemeral ep squares that are not actually capturable.
  - Alternative: `EnPassantMode::Always` (rejected: more sensitive to representation choices).

- Define error/partial-parse behavior as "hash the last successfully reached position".
  - When movetext is empty: return NULL.
  - When movetext is NULL: return NULL (default DuckDB scalar null propagation).
  - When parsing encounters a SAN token that cannot be applied legally: stop processing immediately and hash the current position.
  - Alternative: return NULL or error on illegal move (rejected: other movetext utilities already prefer best-effort prefix processing).

- Implement a dedicated visitor that maintains state:
  - `pos: shakmaty::Chess`
  - `hash: shakmaty::zobrist::Zobrist64` (updated after each applied move)
  - `parse_error: bool` and/or `stopped: bool`
  - It will:
    - ignore `nag`, `comment`, and `partial_comment` events,
    - `Skip(true)` on `begin_variation` to keep mainline-only behavior,
    - on each `san` event, use the `SanPlus` value provided by `pgn-reader` (avoid stringify+reparse), convert to a move with `san_plus.san.to_move(&pos)`, apply with `pos.play_unchecked(m)`, and refresh `hash`.
    - on the first SAN token that cannot be applied, stop parsing by returning `ControlFlow::Break(())` (or equivalent), preserving the last valid position/hash.

- Reuse strategy:
  - Keep `parse_movetext_mainline()` as-is for string-based functions.
  - Add a new helper (e.g. `parse_movetext_final_position_hash(movetext) -> (Zobrist64, bool)`) implemented via the new visitor.
  - Optionally extend `ParsedMovetext` to carry `final_zobrist: Option<u64>` if that reduces duplicate parsing in future functions, but avoid forcing unrelated callers to compute hashes.

## Risks / Trade-offs

- [Behavior change] Hash values will change for all inputs -> Mitigation: update tests; document in the move-analysis delta spec.
- [Collisions by transposition] Different move sequences can lead to the same final position and thus the same hash -> Mitigation: explicitly define the new semantics in the spec; adjust tests to avoid expecting discrimination-by-sequence.
- [DuckDB type compatibility] Zobrist is an unsigned 64-bit value -> Mitigation: return `UBIGINT` from `chess_moves_hash` to avoid signed reinterpretation.
- [Parsing/legality differences] `pgn-reader` accepts more syntaxes than `shakmaty` can apply in a given position -> Mitigation: stop at first illegal/unapplicable SAN and hash the last valid position.

## Migration Plan

- Update implementation and tests together; no data migration is required.
- This is an output-value change only; deployments can roll back by reverting the extension build.

## Open Questions

- None.
