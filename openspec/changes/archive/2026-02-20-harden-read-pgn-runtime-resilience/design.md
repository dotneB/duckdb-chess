## Context

`read_pgn` is a runtime-facing table function that currently coordinates file ingestion across workers with shared mutex-protected state. The extension contract requires non-panicking runtime behavior, but poison-induced `lock().unwrap()` calls can panic and abort execution. In multi-file glob mode, the system should continue on recoverable failures, yet entry-level glob iterator errors are not consistently surfaced as warnings, reducing diagnosability.

## Goals / Non-Goals

**Goals:**
- Eliminate panic-on-poison behavior in runtime reader paths.
- Preserve ingest semantics: explicit single-file paths fail hard; glob mode skips unreadable files and continues.
- Ensure skipped glob entries (including iterator errors) produce observable warnings.
- Add focused tests for poison-safe behavior and glob error observability.

**Non-Goals:**
- No changes to `read_pgn` output schema or chunking behavior.
- No redesign of parser/visitor logic or movetext semantics.
- No change to compression feature set.

## Decisions

- Replace runtime `lock().unwrap()` with poison-safe handling (`match` on `lock()` and recovery via `into_inner()` where safe) so poisoned state does not trigger panics in table-function runtime paths.
  - Alternative considered: propagating poison as fatal errors everywhere. Rejected because it would violate the extension's continue-on-recoverable-error posture for multi-file ingestion and increase user-visible hard failures.
- Keep explicit-path failure handling strict: open/read failures for a non-glob single path remain immediate errors.
  - Alternative considered: downgrading explicit-path failures to warnings. Rejected to preserve current contract and avoid silently ignoring user-typed paths.
- Treat glob iterator entry errors like unreadable files: log warning with path/error context and continue processing remaining entries.
  - Alternative considered: ignore iterator `Err` entries silently. Rejected because it hides data-loss conditions.
- Add tests near reader/runtime coverage to assert non-panicking poison handling and warning-triggering behavior for glob iteration failures where test seams allow deterministic assertions.

## Risks / Trade-offs

- [Recovered poisoned lock may expose partially updated state] -> Mitigation: limit recovery to paths where state is either immutable metadata or can safely advance to next file/entry; fail with structured error if invariant cannot be guaranteed.
- [Warning assertions can be brittle if tied to exact log text] -> Mitigation: assert stable substrings or behavior outcomes (continue vs fail-hard), keeping message wording flexible.
- [Additional branching in hot path] -> Mitigation: keep poison and iterator-error branches cold and minimal; preserve existing streaming/chunked flow.

## Migration Plan

- No user migration required.
- Implement in `reader` internals behind unchanged SQL API.
- Validate with existing `just test`/`just test-release` plus targeted new tests.
- Rollback is straightforward by reverting the change set if regressions are found.

## Open Questions

- Whether current test harness can directly capture warning output for glob iterator errors, or whether behavior-based assertions should be preferred in unit tests.
