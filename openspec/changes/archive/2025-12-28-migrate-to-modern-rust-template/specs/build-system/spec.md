# Build System Capability

## ADDED Requirements

### Requirement: Rust 2024 Edition Support
The extension build system SHALL use Rust 2024 Edition to enable modern language features and improved compiler diagnostics.

#### Scenario: Cargo.toml specifies edition
- **WHEN** `Cargo.toml` is read
- **THEN** it SHALL contain `edition = "2024"`

#### Scenario: Modern Rust features are available
- **WHEN** developers write code using Rust 2024 features
- **THEN** the code SHALL compile successfully

### Requirement: Native Rust Toolchain Only
The extension build system SHALL only require the Rust toolchain, eliminating Python, Make, and git submodule dependencies.

#### Scenario: Build with cargo only
- **WHEN** developer runs `cargo duckdb-ext-build`
- **THEN** the extension SHALL build successfully without requiring Python or Make

#### Scenario: Development prerequisites minimal
- **WHEN** new developer sets up the project
- **THEN** they SHALL only need to install Rust and `cargo-duckdb-ext-tools`

### Requirement: Modern Extension Macros
The extension SHALL use `duckdb-ext-macros` instead of the legacy `duckdb-loadable-macros` for defining the extension entry point.

#### Scenario: Entry point uses modern macro
- **WHEN** `src/chess/mod.rs` is examined
- **THEN** it SHALL use `#[duckdb_extension]` attribute macro
- **AND** it SHALL NOT use `#[duckdb_entrypoint_c_api]`

#### Scenario: Macro attributes are correct
- **WHEN** the extension entry point is defined
- **THEN** it SHALL use `name` attribute for extension name
- **AND** it SHALL use `api_version` attribute for DuckDB version requirement

### Requirement: Cargo Plugin Build Tools
The build system SHALL use `cargo-duckdb-ext-tools` for building and packaging extensions with DuckDB metadata.

#### Scenario: High-level build command available
- **WHEN** developer runs `cargo duckdb-ext-build`
- **THEN** the command SHALL compile the extension and append DuckDB metadata

#### Scenario: Release build supported
- **WHEN** developer runs `cargo duckdb-ext-build -- --release`
- **THEN** the command SHALL create an optimized release build with metadata

#### Scenario: Low-level packaging available
- **WHEN** developer needs manual control over metadata
- **THEN** `cargo duckdb-ext-pack` command SHALL be available for explicit parameter specification

### Requirement: Cross-Platform Build Support
The build system SHALL support building extensions for all major DuckDB platforms using Rust target triples.

#### Scenario: Linux x64 build
- **WHEN** building for `x86_64-unknown-linux-gnu` target
- **THEN** extension SHALL be built with platform identifier `linux_amd64`

#### Scenario: Linux ARM64 build
- **WHEN** building for `aarch64-unknown-linux-gnu` target
- **THEN** extension SHALL be built with platform identifier `linux_arm64`

#### Scenario: macOS Intel build
- **WHEN** building for `x86_64-apple-darwin` target
- **THEN** extension SHALL be built with platform identifier `osx_amd64`

#### Scenario: macOS Apple Silicon build
- **WHEN** building for `aarch64-apple-darwin` target
- **THEN** extension SHALL be built with platform identifier `osx_arm64`

#### Scenario: Windows x64 build
- **WHEN** building for `x86_64-pc-windows-msvc` target
- **THEN** extension SHALL be built with platform identifier `windows_amd64`

### Requirement: Extension Metadata Format
The build system SHALL generate DuckDB-compatible extension metadata footer matching the official format specification.

#### Scenario: Metadata footer present
- **WHEN** extension binary is built
- **THEN** it SHALL have a 534-byte metadata footer appended

#### Scenario: Metadata fields correct
- **WHEN** metadata is examined
- **THEN** it SHALL contain extension name, version, platform, DuckDB version, and ABI type

#### Scenario: Extension loads in DuckDB
- **WHEN** extension is loaded with `LOAD 'extension_path'`
- **THEN** DuckDB SHALL load it without metadata errors

### Requirement: Simplified CI/CD Pipeline
The CI/CD system SHALL use GitHub Actions with direct cargo commands instead of complex reusable workflows and git submodules.

#### Scenario: No Python in CI
- **WHEN** GitHub Actions workflow runs
- **THEN** it SHALL NOT install or use Python

#### Scenario: No git submodule
- **WHEN** repository is cloned
- **THEN** it SHALL NOT require git submodule initialization

#### Scenario: Multi-platform matrix builds
- **WHEN** CI workflow runs
- **THEN** it SHALL build for Linux x64, Linux ARM64, macOS Intel, macOS ARM64, and Windows x64

#### Scenario: Artifacts uploaded
- **WHEN** CI build completes successfully
- **THEN** extension binaries SHALL be uploaded as artifacts for each platform

### Requirement: Optional Make Wrapper
The project MAY provide a minimal Makefile for convenience, but it SHALL NOT be required for building.

#### Scenario: Makefile wraps cargo commands
- **WHEN** `Makefile` exists
- **THEN** it SHALL call `cargo duckdb-ext-build` commands

#### Scenario: Build without Make succeeds
- **WHEN** developer builds using only cargo commands
- **THEN** the build SHALL succeed identically to Make-based builds

### Requirement: Developer Documentation
The project SHALL document the simplified Rust-only build process clearly for developers.

#### Scenario: README has build instructions
- **WHEN** developer reads `README.md`
- **THEN** it SHALL document installation of `cargo-duckdb-ext-tools`
- **AND** it SHALL document basic build commands

#### Scenario: Prerequisites documented
- **WHEN** developer checks prerequisites
- **THEN** documentation SHALL specify only Rust toolchain is required
- **AND** Python and Make SHALL NOT be listed as required dependencies

#### Scenario: Version compatibility documented
- **WHEN** developer checks extension compatibility
- **THEN** documentation SHALL specify which DuckDB versions the extension works with
- **AND** it SHALL document whether strict version matching is required or if minor version flexibility exists
