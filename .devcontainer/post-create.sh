#!/usr/bin/env bash
set -euo pipefail

echo "Initializing submodules..."
git submodule update --init --recursive

if ! command -v just >/dev/null 2>&1; then
  echo "Installing just..."
  cargo install just --locked
fi

if ! command -v cargo-binstall >/dev/null 2>&1; then
  echo "Installing cargo-binstall..."
  cargo install cargo-binstall --locked
fi

if ! command -v cargo-duckdb-ext-build >/dev/null 2>&1 || ! command -v duckdb-slt >/dev/null 2>&1; then
  echo "Installing project tools..."
  just install-tools
fi

echo "Dev container setup complete."
