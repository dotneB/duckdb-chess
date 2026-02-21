## 1. Reader Runtime Resilience

- [x] 1.1 Replace runtime `lock().unwrap()` calls in `src/chess/reader.rs` with poison-safe handling (recover or propagate structured errors without panicking).
- [x] 1.2 Keep explicit single-file ingestion fail-hard behavior when open/read setup fails for a non-glob path.
- [x] 1.3 Preserve glob-mode continue-on-error behavior while ensuring poisoned-lock handling does not regress multi-file progress guarantees.

## 2. Glob Warning Observability

- [x] 2.1 Handle glob iterator `Err` entries explicitly in multi-file path collection and emit warnings with actionable context.
- [x] 2.2 Ensure unreadable files discovered during glob ingestion are skipped with warnings (not silent drops) and successful files still process.
- [x] 2.3 Keep warning messages stable enough for tests/log inspection while avoiding brittle exact-string coupling.

## 3. Tests and Verification

- [x] 3.1 Add or extend unit tests for poison-safe reader behavior to confirm runtime paths no longer panic on poisoned mutexes.
- [x] 3.2 Add or extend tests for glob entry iteration errors to verify warnings are observable and ingestion continues.
- [x] 3.3 Validate explicit-path fail-hard vs glob skip semantics with existing and updated tests.
- [x] 3.4 Run `just test` (and `just test-release` if needed) to confirm all existing `read_pgn` SQL tests and unit tests pass.
