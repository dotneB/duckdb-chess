# Change: Migrate to Modern DuckDB Rust Extension Template

## Why

The current extension uses DuckDB's official `extension-template-rs` which has several limitations:
- Only supports Rust 2021 Edition, limiting access to modern Rust features
- Requires multiple external dependencies (Python, Python3-venv, make, git) beyond the Rust toolchain
- Uses Python scripts (`append_extension_metadata.py`) for packaging instead of native Rust tooling
- Includes heavy `extension-ci-tools` infrastructure as a git submodule
- Build process is complex with mixed Rust/Python/Make workflows

A community member ([@redraiment](https://github.com/redraiment)) has created a modern alternative that:
- Supports Rust 2024 Edition and newer Rust versions
- Only requires the Rust toolchain (no Python, make, git dependencies)
- Uses native Rust cargo plugins (`cargo-duckdb-ext-tools`) for packaging
- Provides simplified, pure-Rust development workflow
- Uses modern procedural macros (`duckdb-ext-macros`) compatible with Rust 2024 Edition

## What Changes

- **Update dependencies**: Replace `duckdb-loadable-macros` with `duckdb-ext-macros`
- **Update Cargo.toml**: Bump to Rust 2024 Edition, update dependency versions
- **Simplify build system**: Replace Make + Python build scripts with cargo-only workflow
- **Update entry point macro**: Change from `#[duckdb_entrypoint_c_api]` to `#[duckdb_extension]`
- **Remove extension-ci-tools submodule**: Eliminate complex CI infrastructure in favor of simpler approach
- **Update CI/CD pipeline**: Simplify GitHub Actions workflow to use cargo-based builds
- **Update documentation**: Reflect new simpler build process in README

## Impact

**Affected specs:**
- `build-system` (new capability)
- `code-structure` (modified for entry point changes)

**Affected code:**
- `Cargo.toml` - Edition and dependency updates
- `src/chess/mod.rs` - Entry point macro change
- `Makefile` - Simplified or removed
- `.github/workflows/MainDistributionPipeline.yml` - Updated CI/CD
- `extension-ci-tools/` - Removed submodule
- `.cargo/config.toml` - Potentially simplified
- `README.md` - Updated build instructions

**Benefits:**
- Simpler developer onboarding (just install Rust)
- Faster builds (pure Rust, no Python overhead)
- Access to Rust 2024 Edition features
- More maintainable build system
- Reduced repository size (no submodule)
- Native Rust development workflow
- **Improved version compatibility**: Modern template may offer better version flexibility vs strict version matching requirement of old template with `USE_UNSTABLE_C_API=1`

**Risks:**
- May need to verify cross-platform build compatibility
- CI/CD pipeline for multi-platform builds needs validation
- Need to ensure extension metadata format remains compatible
- **Version compatibility**: Need to validate whether modern template still requires strict DuckDB version matching or provides better forward/backward compatibility
