# Tasks

- [x] Create reproduction PGN file with invalid UTF-8 bytes <!-- id: 1 -->
- [x] Implement lossy UTF-8 decoding in `read_pgn` loop using `read_until` and `String::from_utf8_lossy` <!-- id: 2 -->
- [x] Verify compilation and fix any mutable borrow issues <!-- id: 3 -->
- [x] Add regression test case (`test/sql/read_pgn_bad_utf8_real.test`) <!-- id: 4 -->
- [x] Verify tests pass with `cargo test` or `make test` <!-- id: 5 -->
