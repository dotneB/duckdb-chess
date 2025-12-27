# build-system Specification

## Purpose
To establish a simplified Rust-native build system using modern DuckDB extension tooling, eliminating Python and Make dependencies while maintaining cross-platform compatibility.

## Requirements
### Requirement: Rust-Only Build System
The project MUST use only Rust toolchain dependencies for building of extension.

#### Scenario: Build dependencies
When building extension
It must require only Rust toolchain and `cargo-duckdb-ext-tools`
And it must not require Python, make, or git submodules

### Requirement: Modern Extension Macros
The project MUST use `duckdb-ext-macros` instead of `duckdb-loadable-macros`.

#### Scenario: Macro dependency
When building extension
It must depend on `duckdb-ext-macros` version 0.1.0+
And it must not depend on `duckdb-loadable-macros`

### Requirement: Native Build Tools
The project MUST use `cargo-duckdb-ext-tools` for extension packaging.

#### Scenario: Build process
When building for development
It must use `cargo duckdb-ext-build`
When building for release
It must use `cargo duckdb-ext-build -- --release`
When packaging manually
It must use `cargo duckdb-ext-pack`

### Requirement: Rust 2024 Edition
The project MUST target Rust 2024 Edition.

#### Scenario: Edition configuration
The `Cargo.toml` must specify `edition = "2024"`
And code must be compatible with Rust 2024 Edition requirements

### Requirement: Simplified CI/CD
The project MUST use direct cargo commands in CI/CD without complex workflows.

#### Scenario: GitHub Actions
When building in CI
It must directly call `cargo duckdb-ext-build`
And it must not use reusable workflows from extension-ci-tools

### Requirement: DuckDB Version Compatibility
The project MUST specify compatible DuckDB versions in dependencies.

#### Scenario: Version specification
The `Cargo.toml` must specify `duckdb` and `libduckdb-sys` versions
And it must target DuckDB 1.4.3 for user compatibility

### Requirement: Optional Makefile
The project MUST provide a minimal Makefile for backward compatibility or have documentation explaining cargo-only workflow.

#### Scenario: Makefile usage
If a Makefile is provided
It must wrap cargo commands with simple aliases
And it must be documented as optional

### Requirement: Cross-Platform Support
The build system MUST support all DuckDB platforms.

#### Scenario: Target platforms
The build must support:
- Linux amd64/arm64
- macOS Intel/ARM64  
- Windows x64
And cargo must handle cross-compilation appropriately