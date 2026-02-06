## MODIFIED Requirements

### Requirement: Rust-Only Build System
The project MUST provide a Rust-first local workflow for building, linting, and testing the extension.

The primary local workflow SHALL use Rust/cargo tooling (`cargo duckdb-ext-build`, `cargo test`, `cargo fmt`, `cargo clippy`) and repository wrappers in `Makefile`.

#### Scenario: Rust-first development workflow
- **WHEN** a contributor builds and tests locally
- **THEN** they can use `make check`, `make build-rs`, `make test-rs`, and `make dev` without Python-specific setup

#### Scenario: Template compatibility path
- **WHEN** template/community-extension Make targets are used
- **THEN** `extension-ci-tools` and Python/venv-based tooling may be required as a compatibility path
- **AND** this does not replace the Rust-first local workflow

### Requirement: Native Build Tools
The project MUST use `duckdb-ext-macros` and `cargo-duckdb-ext-tools` for extension build and packaging workflows.

#### Scenario: Build process
- **WHEN** building for development
- **THEN** the workflow uses `cargo duckdb-ext-build`
- **AND** release builds use `cargo duckdb-ext-build -- --release`

#### Scenario: Tool installation pin
- **WHEN** installing extension build tooling from repository wrappers
- **THEN** `cargo-duckdb-ext-tools` is installed from the pinned repository/branch used by this project configuration

### Requirement: Simplified CI/CD
The project MUST keep CI/CD and template wiring compatible with DuckDB community extension expectations while preserving straightforward Rust build commands.

#### Scenario: CI compatibility
- **WHEN** repository CI or template workflows build the extension
- **THEN** they remain compatible with `extension-ci-tools` wiring used by this repository
- **AND** Rust build commands remain the core extension build mechanism

### Requirement: DuckDB Version Compatibility
The project MUST pin DuckDB dependencies and build targets to the repository's current supported DuckDB version.

#### Scenario: Version specification
- **WHEN** checking dependency and build target configuration
- **THEN** `duckdb` and `libduckdb-sys` are pinned to `=1.4.4` in `Cargo.toml`
- **AND** Makefile target version is `v1.4.4`

### Requirement: Optional Makefile
The project MUST provide Make targets for local Rust workflows and MAY include additional template-compatibility targets.

#### Scenario: Makefile usage
- **WHEN** a Makefile is present
- **THEN** it provides wrappers for local Rust-first workflows (`check`, `build-rs`, `release-rs`, `test-rs`, `dev`)
- **AND** it may also include template-compatibility targets that require extra tooling
