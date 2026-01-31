## 1. Define Hash Semantics and Test Coverage

- [x] 1.1 Update unit tests for `chess_moves_hash` to assert Zobrist-based expectations (remove DefaultHasher-specific expected values)
- [x] 1.2 Add a test case proving transposition collision (two different SAN sequences reaching the same final position hash equal)
- [x] 1.3 Add a test case for empty input returning NULL (and NULL input returning NULL)

## 2. Implement Zobrist Hashing Visitor

- [x] 2.1 Add a new `pgn-reader` Visitor that maintains a `shakmaty::Chess` position and updates a `shakmaty::zobrist::Zobrist64` after each applied mainline move (use the `SanPlus` passed by `pgn-reader`, avoid stringify+reparse)
- [x] 2.2 Ensure the visitor skips variations (`begin_variation -> Skip(true)`) and ignores comments/NAGs/partial comments
- [x] 2.3 Define best-effort behavior: on the first SAN token that cannot be applied legally, stop parsing early (via `ControlFlow::Break`) and keep the last valid position/hash

## 3. Wire Visitor into `chess_moves_hash`

- [x] 3.1 Add a helper (e.g. `movetext_final_zobrist_hash(movetext) -> Option<u64>`) that drives `pgn_reader::Reader::read_game` using the new visitor (empty input -> None/NULL)
- [x] 3.2 Update `ChessMovesHashScalar` to use the helper and return the hash as DuckDB `UBIGINT`
- [x] 3.3 Verify `chess_moves_hash` ignores formatting/comments/NAGs/variations consistent with other movetext utilities

## 4. Regression and Compatibility Checks

- [x] 4.1 Run `make test-rs` and fix any failures
- [x] 4.2 Run `make test-release-rs` to ensure release build + SQLLogicTests pass
