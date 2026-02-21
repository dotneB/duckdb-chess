## 1. FFI Boundary Isolation

- [x] 1.1 Audit the pinned `duckdb-rs` version for public `BindInfo` accessors and record whether adapter code can avoid raw layout casting.
- [x] 1.2 Create a dedicated BindInfo adapter module that owns all `BindInfo` <-> `duckdb_bind_info` interop with explicit `SAFETY` invariants.
- [x] 1.3 Refactor `src/chess/reader.rs` bind/named-parameter helpers to use the adapter API and remove direct pointer-layout casts from reader code.

## 2. Behavior Parity Safeguards

- [x] 2.1 Add or update tests to lock in `compression` named-parameter behavior for omitted, SQL `NULL`, `zstd`, and unsupported values.
- [x] 2.2 Verify behavior parity in bind-time error and success paths so no SQL-visible regressions occur for named-parameter handling.

## 3. Upgrade Guidance and Validation

- [x] 3.1 Add maintainer-facing `duckdb-rs` upgrade guidance/checklist covering adapter invariants, accessor adoption checks, and validation steps.
- [x] 3.2 Run `just full` and confirm all checks/tests pass after the refactor.

## 4. DuckDB Helper Module Grouping

- [x] 4.1 Regroup DuckDB-specific helper modules under `src/chess/duckdb/` while preserving helper APIs and behavior.
- [x] 4.2 Update caller imports to use the dedicated DuckDB helper namespace module and keep the extension entrypoint compiling.
- [x] 4.3 Update change specs/design references to reflect helper module locations after regrouping.
