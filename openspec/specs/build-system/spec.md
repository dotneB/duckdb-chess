# build-system Specification

## Purpose
To define a Rust-first build and validation workflow for this DuckDB extension while preserving compatibility with DuckDB community extension template wiring.
## Requirements
### Requirement: Rust-Only Build System
The project MUST provide a Rust-first local workflow for building, linting, and testing the extension.

The primary local workflow SHALL use Rust/cargo tooling (`cargo duckdb-ext-build`, `cargo test`, `cargo fmt`, `cargo clippy`) and repository wrappers in `justfile`.

#### Scenario: Rust-first development workflow
- **WHEN** a contributor builds and tests locally
- **THEN** they can use `just install-tools`, `just check`, `just debug`, `just test`, and `just dev` without Python-specific setup

#### Scenario: Template compatibility path
- **WHEN** template/community-extension Make targets are used
- **THEN** `extension-ci-tools` and Python/venv-based tooling may be required as a compatibility path
- **AND** this does not replace the Rust-first local workflow

### Requirement: Modern Extension Macros
The project MUST use `duckdb-ext-macros` for extension entrypoint registration.

#### Scenario: Macro dependency
- **WHEN** checking extension macro dependencies
- **THEN** the project depends on `duckdb-ext-macros`
- **AND** it does not depend on `duckdb-loadable-macros`

### Requirement: Native Build Tools
The project MUST use `duckdb-ext-macros` and `cargo-duckdb-ext-tools` for extension build and packaging workflows.

#### Scenario: Build process
- **WHEN** building for development
- **THEN** the workflow uses `cargo duckdb-ext-build`
- **AND** release builds use `cargo duckdb-ext-build -- --release`

#### Scenario: Tool installation pin
- **WHEN** installing extension build tooling from repository wrappers
- **THEN** `cargo-duckdb-ext-tools` is installed from the pinned repository/branch used by this project configuration

### Requirement: Rust 2024 Edition
The project MUST target Rust 2024 Edition.

#### Scenario: Edition configuration
- **WHEN** checking crate metadata
- **THEN** `Cargo.toml` specifies `edition = "2024"`

### Requirement: Simplified CI/CD
The project MUST keep CI/CD and template wiring compatible with DuckDB community extension expectations while preserving straightforward Rust build commands.

#### Scenario: CI compatibility
- **WHEN** repository CI or template workflows build the extension
- **THEN** they remain compatible with `extension-ci-tools` wiring used by this repository
- **AND** Rust build commands remain the core extension build mechanism

### Requirement: DuckDB Version Compatibility
The project MUST pin DuckDB dependencies and build targets to the repository's current supported DuckDB version.

### Requirement: Justfile Local Wrappers
The project MUST provide `just` recipes for local Rust-first workflows.

#### Scenario: justfile usage
- **WHEN** a `justfile` is present
- **THEN** it provides wrappers for local Rust-first workflows (`install-tools`, `check`, `debug`, `release`, `test`, `test-release`, `dev`)

### Requirement: Optional Makefile
The project MAY provide Make targets for template/community-extension compatibility paths.

#### Scenario: Makefile usage
- **WHEN** a Makefile is present
- **THEN** it is used for `extension-ci-tools` compatibility wiring
- **AND** local Rust-first workflows are documented via `just` recipes

### Requirement: Cross-Platform Support
The build system MUST support DuckDB target platforms.

#### Scenario: Target platforms
- **WHEN** building extension artifacts for release
- **THEN** workflows can target Linux, macOS, and Windows DuckDB extension platforms
