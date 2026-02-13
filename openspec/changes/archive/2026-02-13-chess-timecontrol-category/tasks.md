## 1. TimeControl category core logic

- [x] 1.1 Add a pure helper in `src/chess/timecontrol.rs` that maps parsed TimeControl data to Lichess categories using `base_seconds + 40 * increment_seconds`.
- [x] 1.2 Implement NULL-safe handling in the helper for non-normal or non-categorizable inputs (`?`, `-`, `*N`, missing periods, parse failures).
- [x] 1.3 Add focused unit tests for threshold boundaries (`29`, `30`, `179`, `180`, `479`, `480`, `1499`, `1500`), increment-driven cases (for example `2+12`), and shorthand ambiguity (`29+0` vs `29''`).

## 2. SQL scalar integration

- [x] 2.1 Implement `ChessTimecontrolCategoryScalar` in `src/chess/timecontrol.rs` following existing scalar patterns.
- [x] 2.2 Register the new scalar in `src/chess/mod.rs` as `chess_timecontrol_category`.
- [x] 2.3 Ensure scalar behavior matches existing null semantics (NULL in -> NULL out; unparseable -> NULL category).

## 3. SQL-visible behavior tests

- [x] 3.1 Add SQLLogicTest coverage in `test/sql/` for representative inputs and expected categories.
- [x] 3.2 Add SQLLogicTest cases for unsupported modes and invalid values returning NULL.

## 4. Documentation and validation

- [x] 4.1 Update `README.md` with the new `chess_timecontrol_category` function and threshold definition.
- [x] 4.2 Run `just dev` and fix any formatting, clippy, unit, or SQLLogicTest issues.
