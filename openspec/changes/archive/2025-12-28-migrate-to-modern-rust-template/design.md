# Design: Modern Rust Extension Template Migration

## Context

The chess-duckdb-extension currently uses DuckDB's official `extension-template-rs` which was designed for the older DuckDB ecosystem. The project requires:
- Python 3.12+ and virtual environment tooling
- Make build system
- Git submodule (`extension-ci-tools`) with Docker configs, Python scripts, and CI workflows
- Mixed build process: Cargo → Make → Python script → extension binary
- **Uses `USE_UNSTABLE_C_API=1`** which requires strict DuckDB version matching (e.g., extension built for DuckDB 1.4.1 won't load in DuckDB 1.4.3)

A modern alternative exists from the community:
- **duckdb-ext-macros**: Procedural macros supporting Rust 2024 Edition
- **cargo-duckdb-ext-tools**: Native Rust cargo plugins replacing Python scripts
- **duckdb-ext-rs-template**: Simplified template demonstrating the new approach

**Stakeholders:**
- Extension developers (want simpler workflow)
- CI/CD pipeline (wants faster builds)
- Extension users (want extensions that work across minor DuckDB versions without recompilation)

**Constraints:**
- Must maintain DuckDB extension compatibility (C API, metadata format)
- Must support cross-platform builds (Linux, macOS, Windows; x64, ARM64)
- Must preserve existing functionality (read_pgn, chess_moves_* functions)
- CI/CD should remain automated and multi-platform
- **Important**: Current user has DuckDB 1.4.3 installed; extension built for 1.4.1 requires exact match with old template

## Goals / Non-Goals

**Goals:**
- Simplify development workflow to Rust-only toolchain
- Support Rust 2024 Edition for modern language features
- Remove Python and Make dependencies
- Reduce repository complexity (remove git submodule)
- Maintain or improve build performance
- Preserve all existing extension functionality
- Keep multi-platform CI/CD builds working

**Non-Goals:**
- Changing extension functionality or API
- Adding new features to the chess extension
- Rewriting the CI/CD system entirely (incremental update only)
- Supporting platforms not currently supported
- Migration of test framework (continue using SQLLogicTest)

## Decisions

### Decision 1: Use `duckdb-ext-macros` instead of `duckdb-loadable-macros`

**Rationale:**
- Official `duckdb-loadable-macros` locked to Rust 2021 Edition
- `duckdb-ext-macros` is API-compatible drop-in replacement
- Enables Rust 2024 Edition features (async improvements, let-else, etc.)
- Maintained by active community member with responsive support

**Alternatives considered:**
1. **Stay with official macros**: Rejected due to edition lock and lack of updates
2. **Fork official macros**: Rejected due to maintenance burden
3. **Write custom macros**: Rejected due to complexity and duplication

**Trade-off:** Depends on community-maintained crate vs official DuckDB crate, but the API compatibility and active maintenance mitigate this risk.

### Decision 2: Use `cargo-duckdb-ext-tools` for packaging

**Rationale:**
- Native Rust implementation eliminates Python dependency
- Integrates naturally with cargo workflows (`cargo duckdb-ext-build`)
- Better performance than Python script invocation
- Provides both high-level (`build`) and low-level (`pack`) commands

**Alternatives considered:**
1. **Keep Python scripts**: Rejected due to dependency complexity
2. **Write custom build.rs**: Rejected due to cargo build.rs limitations (can't easily control post-build steps)
3. **Use cargo-make or other task runner**: Rejected as overly complex for this use case

**Trade-off:** Introduces dependency on `cargo-duckdb-ext-tools` binary, but this is a one-time `cargo install` vs managing Python environments.

### Decision 3: Remove `extension-ci-tools` submodule

**Rationale:**
- Submodule contains Docker configs, Python scripts, and CI templates not needed with new approach
- 80%+ of submodule content is for non-Rust extensions (C/C++ tooling, vcpkg ports)
- GitHub Actions can directly call cargo commands without complex wrapper makefiles
- Simplifies repository maintenance (no submodule updates)

**Alternatives considered:**
1. **Keep submodule, update workflows**: Rejected as submodule still adds complexity
2. **Cherry-pick needed files**: Rejected as minimal value vs direct cargo usage
3. **Create custom lightweight CI templates**: Rejected as cargo-duckdb-ext-tools handles this

**Migration path:** Extract GitHub Actions workflow patterns, rewrite to use cargo directly.

### Decision 4: Simplify or remove Makefile

**Rationale:**
- Current Makefile primarily wraps cargo + Python scripts
- With cargo plugins, Make adds little value
- Developers familiar with Rust expect `cargo` commands
- Keep minimal Makefile for backwards compatibility (optional)

**Alternatives considered:**
1. **Remove Makefile entirely**: Considered, but keep for familiarity
2. **Keep complex Makefile**: Rejected as defeats simplification purpose
3. **Use just/task runners**: Rejected as adding new dependency

**Decision:** Create minimal Makefile that wraps cargo commands with helpful aliases:
```makefile
.PHONY: build test release

build:
	cargo duckdb-ext-build

release:
	cargo duckdb-ext-build -- --release

test: build
	cargo test
	# Run SQLLogicTest tests
```

### Decision 5: Update Rust Edition to 2024

**Rationale:**
- Unlocks modern language features
- Better error messages and diagnostics
- Improved async/await ergonomics (not currently used, but future-proofing)
- Aligns with Rust ecosystem best practices

**Alternatives considered:**
1. **Stay on 2021**: Rejected as blocks language improvements
2. **Wait for 2027**: Rejected as 2024 is stable and ready

**Trade-off:** Minimal risk as codebase is small and standard patterns are used.

## Technical Architecture

### Current Architecture
```
Developer
  ↓
cargo build → libduckdb_chess.{so,dylib,dll}
  ↓
make release (wraps cargo + Python)
  ↓
extension-ci-tools/scripts/append_extension_metadata.py
  ↓
duckdb_chess.duckdb_extension (binary + 534-byte metadata footer)
```

### New Architecture
```
Developer
  ↓
cargo duckdb-ext-build (--release optional)
  ↓
  [Internally: cargo build + metadata append]
  ↓
duckdb_chess.duckdb_extension (binary + 534-byte metadata footer)
```

### Macro Migration
```rust
// OLD (duckdb-loadable-macros)
#[duckdb_entrypoint_c_api(ext_name = "duckdb_chess", min_duckdb_version = "v1.0.0")]
pub unsafe fn extension_entrypoint(con: Connection) -> Result<(), Box<dyn Error>>

// NEW (duckdb-ext-macros) 
#[duckdb_extension(name = "duckdb_chess", api_version = "v1.0.0")]
pub unsafe fn extension_entrypoint(con: Connection) -> Result<(), Box<dyn Error>>
```

**Changes:** Attribute names only (`ext_name` → `name`, `min_duckdb_version` → `api_version`). Function signature identical.

### Extension Metadata Format

Both approaches generate identical DuckDB extension metadata:
- Extension name
- DuckDB platform (e.g., `osx_arm64`, `linux_amd64`)
- Extension version
- DuckDB version compatibility
- ABI type (default: `C_STRUCT_UNSTABLE`)

The `cargo-duckdb-ext-tools` implementation is based on reverse-engineering the official Python script, ensuring byte-for-byte compatibility.

### CI/CD Changes

**Current workflow:**
```yaml
jobs:
  duckdb-stable-build:
    uses: duckdb/extension-ci-tools/.github/workflows/_extension_distribution.yml@v1.4.0
    with:
      extension_name: rusty_quack
      extra_toolchains: rust;python3
```

**New workflow approach:**
```yaml
jobs:
  build:
    strategy:
      matrix:
        include:
          - os: ubuntu-latest
            target: x86_64-unknown-linux-gnu
          - os: ubuntu-latest
            target: aarch64-unknown-linux-gnu
          - os: macos-latest
            target: x86_64-apple-darwin
          - os: macos-latest
            target: aarch64-apple-darwin
          - os: windows-latest
            target: x86_64-pc-windows-msvc
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          targets: ${{ matrix.target }}
      - run: cargo install cargo-duckdb-ext-tools
      - run: cargo duckdb-ext-build -- --release --target ${{ matrix.target }}
      - uses: actions/upload-artifact@v4
        with:
          name: extension-${{ matrix.target }}
          path: target/${{ matrix.target }}/release/*.duckdb_extension
```

**Key changes:**
- Direct matrix strategy instead of reusable workflow
- No Python installation needed
- Simplified artifact upload
- Explicit target specification for cross-compilation

## Risks / Trade-offs

### Risk 1: Community dependency vs official tooling
**Impact:** Medium  
**Mitigation:**
- `duckdb-ext-macros` is API-compatible, can fall back to official if needed
- Code change is minimal (attribute names only)
- Active maintainer with quick responses
- Template has been tested by other community members

### Risk 2: Cross-platform build compatibility
**Impact:** Medium  
**Mitigation:**
- `cargo-duckdb-ext-tools` explicitly supports all DuckDB platforms
- Test builds on all target platforms before merging
- Maintain old build system in separate branch during transition
- CI validation before deployment

### Risk 3: Extension metadata format changes
**Impact:** Low  
**Mitigation:**
- Tool based on DuckDB's official metadata format
- Validation step in CI to verify extension loads
- Can use `cargo duckdb-ext-pack` with explicit parameters if needed

### Risk 4: Loss of CI/CD features from extension-ci-tools
**Impact:** Low  
**Mitigation:**
- Current usage is minimal (basic build + test)
- Advanced features (signing, distribution) not currently used
- Can replicate needed functionality in GitHub Actions directly

### Risk 5: DuckDB Version Compatibility
**Impact:** High (critical user pain point)  
**Context:**
- Old template uses `USE_UNSTABLE_C_API=1` requiring exact DuckDB version matching
- User has DuckDB 1.4.3 installed; extension built for 1.4.1 won't load
- This creates friction for users across minor version updates

**Mitigation:**
- Investigate whether modern template uses stable C API or provides better version compatibility
- Test extension built with v1.4.2/1.4.3 dependencies loads in DuckDB 1.4.3
- Document version compatibility strategy in README
- If strict matching still required, ensure build versions align with common DuckDB installations
- Consider building multiple versions (1.4.1, 1.4.2, 1.4.3) in CI for broader compatibility

## Migration Plan

### Phase 1: Preparation (Day 1)
1. Create feature branch `migrate-modern-template`
2. Install `cargo-duckdb-ext-tools` locally
3. Test build locally with new tooling

### Phase 2: Code Migration (Day 1-2)
1. Update `Cargo.toml` (edition, dependencies)
2. Update `src/chess/mod.rs` (macro usage)
3. Create simplified `Makefile`
4. Test local builds (debug and release)
5. Run test suite locally

### Phase 3: CI/CD Migration (Day 2-3)
1. Update GitHub Actions workflow
2. Remove `extension-ci-tools` submodule
3. Test CI builds on feature branch
4. Validate artifacts from CI

### Phase 4: Validation (Day 3-4)
1. Run full test suite on all platforms
2. Manual load testing with DuckDB
3. Performance comparison (build time, binary size)
4. Documentation review

### Phase 5: Deployment (Day 4-5)
1. Update documentation
2. Create PR with migration
3. Review and merge
4. Tag release with new build system

### Rollback Plan
If critical issues found:
1. Keep old build system in `legacy-build` branch
2. Revert main branch to previous commit
3. Can maintain both approaches temporarily
4. Investigate issues in separate feature branch

### Validation Checklist
- [ ] Extension builds on Linux x64
- [ ] Extension builds on Linux ARM64
- [ ] Extension builds on macOS Intel
- [ ] Extension builds on macOS ARM64
- [ ] Extension builds on Windows x64
- [ ] Extension loads in DuckDB without errors
- [ ] **Extension built for 1.4.2 loads in DuckDB 1.4.3 (version compatibility test)**
- [ ] **Extension built for 1.4.3 loads in DuckDB 1.4.3 (exact version test)**
- [ ] All SQL tests pass
- [ ] `cargo test` passes
- [ ] Build time comparable or better
- [ ] Binary size comparable or smaller
- [ ] Metadata format validated

## Open Questions

1. **Should we maintain Make compatibility?**
   - Proposal: Keep minimal Makefile for developer familiarity
   - Decision: Yes, but make it optional and document cargo-only workflow

2. **How to handle version pinning for cargo-duckdb-ext-tools?**
   - Proposal: Document recommended version in README
   - Decision: Use `>=0.1.0` requirement, test with latest

3. **Should we update DuckDB version to 1.4.3 to match user's installation?**
   - Context: User currently has DuckDB 1.4.3 installed, current extension built for 1.4.1 won't load due to strict version matching
   - Proposal: Update dependencies to 1.4.3 OR investigate if modern template provides better version compatibility
   - Decision: Test with 1.4.3, verify whether strict matching is still required with modern macros
   - Alternative: Build multiple versions if strict matching persists

4. **How to handle the extension name mismatch (rusty_quack vs duckdb_chess)?**
   - Current state: CI workflow uses `rusty_quack`, code uses `duckdb_chess`
   - Proposal: Standardize on `duckdb_chess` throughout
   - Decision: Align extension name in all locations
