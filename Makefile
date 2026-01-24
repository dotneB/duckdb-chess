# Simplified Makefile for DuckDB Chess Extension
# 
# This Makefile provides convenient aliases for cargo-based builds.
# All targets are optional - you can use cargo commands directly.
#
# Need to run `make install-tools` once to set up the build tools.

.PHONY: all build release test test-release check check-fix clean install-tools dev help

EXTENSION_NAME := duckdb_chess

# Default target
all: build

# Install build tools (run once)
install-tools:
	@echo "Installing cargo-duckdb-ext-tools..."
	cargo install cargo-duckdb-ext-tools --locked --git "https://github.com/dotneB/cargo-duckdb-ext-tools.git" --branch "fix/windows-build"
#	cargo install cargo-duckdb-ext-tools --path "../cargo-duckdb-ext-tools" --locked
	@echo "Installing duckdb-sqllogictest-rs..."
	cargo binstall duckdb-slt --locked

# Build debug extension
build:
	@echo "Building debug extension..."
	cargo duckdb-ext-build

# Build release extension
release:
	@echo "Building release extension..."
	cargo duckdb-ext-build -- --release

# Run Rust unit + integration tests
test: build
	@echo "Running cargo tests..."
	cargo test
	@echo "Running duckdb-slt integration tests..."
	duckdb-slt.exe -e ./target/debug/$(EXTENSION_NAME).duckdb_extension -u -w "$(CURDIR)" "$(CURDIR)/test/sql/*.test"

# Run Rust unit + integration tests (release)
test-release: release
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

# Clean build artifacts
clean:
	@echo "Cleaning build artifacts..."
	cargo clean

# Development helper: run a quick build and test cycle
dev: build check test

# Show available targets
help:
	@echo "Available targets:"
	@echo "  make install-tools  - Install cargo-duckdb-ext-tools (one-time setup)"
	@echo "  make dev            - Quick build + test cycle"
	@echo "  make build          - Build debug extension"
	@echo "  make release        - Build release extension"
	@echo "  make test           - Run unit + integration tests (debug)"
	@echo "  make test-release   - Run unit + integration tests (release)"
	@echo "  make check          - Run fmt/clippy checks"
	@echo "  make check-fix      - Auto-fix fmt/clippy issues"
	@echo "  make clean          - Clean build artifacts"
	@echo ""
	@echo "Direct cargo commands:"
	@echo "  cargo duckdb-ext-build                    - Build debug"
	@echo "  cargo duckdb-ext-build -- --release       - Build release"
	@echo "  cargo test                                - Run tests"
