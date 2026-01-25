.PHONY: clean clean_all

PROJ_DIR := $(dir $(abspath $(lastword $(MAKEFILE_LIST))))

EXTENSION_NAME=duckdb_chess

# Set to 1 to enable Unstable API (binaries will only work on TARGET_DUCKDB_VERSION, forwards compatibility will be broken)
# Note: currently extension-template-rs requires this, as duckdb-rs relies on unstable C API functionality
USE_UNSTABLE_C_API=1

# Target DuckDB version
TARGET_DUCKDB_VERSION=v1.4.3

all: configure debug

# Include makefiles from DuckDB
include extension-ci-tools/makefiles/c_api_extensions/base.Makefile
include extension-ci-tools/makefiles/c_api_extensions/rust.Makefile

configure: venv platform extension_version

debug: build_extension_library_debug build_extension_with_metadata_debug
release: build_extension_library_release build_extension_with_metadata_release

test: test_debug
test_debug: test_extension_debug
test_release: test_extension_release

clean: clean_build clean_rust
clean_all: clean_configure clean

# --- Rust-specific targets ---

# Install build tools (run once)
install-tools:
	@echo "Installing cargo-duckdb-ext-tools..."
	cargo install cargo-duckdb-ext-tools --locked --git "https://github.com/dotneB/cargo-duckdb-ext-tools.git" --branch "fix/windows-build"
#	cargo install cargo-duckdb-ext-tools --path "../cargo-duckdb-ext-tools" --locked
	@echo "Installing duckdb-sqllogictest-rs..."
	cargo binstall duckdb-slt --locked

# Build debug extension
build-rs:
	@echo "Building debug extension..."
	cargo duckdb-ext-build

# Build release extension
release-rs:
	@echo "Building release extension..."
	cargo duckdb-ext-build -- --release

# Run Rust unit + integration tests
test-rs: build-rs
	@echo "Running cargo tests..."
	cargo test
	@echo "Running duckdb-slt integration tests..."
	duckdb-slt.exe -e ./target/debug/$(EXTENSION_NAME).duckdb_extension -u -w "$(CURDIR)" "$(CURDIR)/test/sql/*.test"

# Run Rust unit + integration tests (release)
test-release-rs: release-rs
	@echo "Running cargo tests..."
	cargo test
	@echo "Running duckdb-slt integration tests..."
	duckdb-slt.exe -e ./target/release/$(EXTENSION_NAME).duckdb_extension -u -w "$(CURDIR)" "$(CURDIR)/test/sql/*.test"

check:
	cargo fmt --check
	cargo clippy -- -D warnings

check-fix:
	cargo fmt
	cargo clippy --fix

# Development helper: run a quick build and test cycle
dev: check build-rs test-rs