# Simplified Makefile for DuckDB Chess Extension
# 
# This Makefile provides convenient aliases for cargo-based builds.
# All targets are optional - you can use cargo commands directly.
#
# Prerequisites:
#   - Rust toolchain (https://rustup.rs/)
#   - cargo-duckdb-ext-tools: cargo install cargo-duckdb-ext-tools
#
# Usage:
#   make build          - Build debug extension
#   make release        - Build release extension  
#   make test           - Run tests
#   make clean          - Clean build artifacts

.PHONY: all build release test clean install-tools

EXTENSION_NAME := duckdb_chess

# Default target
all: build

# Install build tools (run once)
install-tools:
	@echo "Installing cargo-duckdb-ext-tools..."
	cargo install cargo-duckdb-ext-tools --path "../cargo-duckdb-ext-tools"

# Build debug extension
build:
	@echo "Building debug extension..."
	cargo duckdb-ext-build

# Build release extension
release:
	@echo "Building release extension..."
	cargo duckdb-ext-build -- --release

# Run Rust unit tests
test: build
	@echo "Running cargo tests..."
	cargo test

# Clean build artifacts
clean:
	@echo "Cleaning build artifacts..."
	cargo clean

# Development helper: run a quick build and test cycle
dev: build test

# Show available targets
help:
	@echo "Available targets:"
	@echo "  make install-tools  - Install cargo-duckdb-ext-tools (one-time setup)"
	@echo "  make build          - Build debug extension"
	@echo "  make release        - Build release extension"
	@echo "  make test           - Run Rust unit tests"
	@echo "  make clean          - Clean build artifacts"
	@echo "  make dev            - Quick build + test cycle"
	@echo ""
	@echo "Direct cargo commands:"
	@echo "  cargo duckdb-ext-build                    - Build debug"
	@echo "  cargo duckdb-ext-build -- --release       - Build release"
	@echo "  cargo test                                - Run tests"
