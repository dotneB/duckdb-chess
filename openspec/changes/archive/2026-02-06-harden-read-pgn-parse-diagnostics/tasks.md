## 1. Deterministic Parser-Error Coverage

- [x] 1.1 Add malformed PGN fixture(s) in `test/pgn_files/` that deterministically trigger `read_game()` parser-stage `Err`
- [x] 1.2 Add SQLLogicTests in `test/sql/read_pgn_parse_errors.test` (or new `read_pgn_parser_errors.test`) asserting `parse_error IS NOT NULL` for parser-stage failure rows
- [x] 1.3 Add SQLLogicTest assertions that ingestion continues to subsequent games/files where current behavior requires continuation

## 2. Diagnostic Context Hardening

- [x] 2.1 Update parser-stage error message construction in `src/chess/reader.rs` to include stage and source file context
- [x] 2.2 Include game index context in parser-stage messages when available
- [x] 2.3 Ensure parser-stage diagnostics and conversion diagnostics are combined in `parse_error` for mixed-error rows
- [x] 2.4 Keep backward-compatible stderr warning emission for parser-stage failures

## 3. Unit and Integration Verification

- [x] 3.1 Add/adjust Rust unit tests around parser-error finalization paths in `src/chess/reader.rs` and/or `src/chess/visitor.rs`
- [x] 3.2 Run `make check` and fix any formatting/clippy issues
- [x] 3.3 Run `make test-rs` and ensure all unit + SQLLogicTests pass
