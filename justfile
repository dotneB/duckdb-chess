install-tools:
  echo "Installing cargo-duckdb-ext-tools..."
  cargo install cargo-duckdb-ext-tools --locked --git "https://github.com/dotneB/cargo-duckdb-ext-tools.git" --branch "fix/windows-build"
  echo "Installing duckdb-sqllogictest-rs..."
  cargo binstall duckdb-slt --locked

debug:
  cargo duckdb-ext-build

release:
  cargo duckdb-ext-build -- --release

test: debug
  echo "Running cargo tests..."
  cargo test
  echo "Running duckdb-slt integration tests..."
  duckdb-slt -e ./target/debug/chess.duckdb_extension -u -w . test/sql/*.test

test-release: release
  echo "Running cargo tests..."
  cargo test
  echo "Running duckdb-slt integration tests..."
  duckdb-slt -e ./target/release/chess.duckdb_extension -u -w . test/sql/*.test

check:
  cargo fmt --check
  cargo clippy -- -D warnings

check-fix:
  cargo fmt
  cargo clippy --fix

dev: check test
  echo "Development workflow completed."

full: check test test-release
  echo "Full workflow completed."

bump-duckdb version:
  sed -i 's/duckdb = { version = "=[0-9.][0-9.]*"/duckdb = { version = "={{version}}"/' Cargo.toml
  sed -i 's/libduckdb-sys = { version = "=[0-9.][0-9.]*"/libduckdb-sys = { version = "={{version}}"/' Cargo.toml
  sed -i 's/\*\*DuckDB\*\* `[0-9.][0-9.]*`/**DuckDB** `{{version}}`/' README.md
  sed -i 's/DUCKDB_VERSION: "[0-9.][0-9.]*"/DUCKDB_VERSION: "{{version}}"/' .github/workflows/ci.yml
  sed -i 's/duckdb_version: v[0-9.][0-9.]*/duckdb_version: v{{version}}/' .github/workflows/MainDistributionPipeline.yml
  sed -i 's/DUCKDB_VERSION: "[0-9.][0-9.]*"/DUCKDB_VERSION: "{{version}}"/' .github/workflows/release.yml
  sed -i 's/- DuckDB target: [0-9.][0-9.]*/- DuckDB target: {{version}}/' openspec/config.yaml
  sed -i 's/TARGET_DUCKDB_VERSION=v[0-9.][0-9.]*/TARGET_DUCKDB_VERSION=v{{version}}/' Makefile

bump-msrv version:
  sed -i 's/rust-version = "[0-9.][0-9.]*"/rust-version = "{{version}}"/' Cargo.toml
  sed -i 's/(MSRV) is `[0-9.][0-9.]*`/(MSRV) is `{{version}}`/' README.md
  sed -i 's/RUST_MSRV: "[0-9.][0-9.]*"/RUST_MSRV: "{{version}}"/' .github/workflows/ci.yml
  sed -i 's/RUST_MSRV: "[0-9.][0-9.]*"/RUST_MSRV: "{{version}}"/' .github/workflows/release.yml
  sed -i 's/Rust MSRV: [0-9.][0-9.]*/Rust MSRV: {{version}}/' openspec/config.yaml

bump-toolchain version:
  sed -i 's/channel = "[0-9.][0-9.]*"/channel = "{{version}}"/' rust-toolchain.toml
  sed -i 's/repo toolchain is `[0-9.][0-9.]*`/repo toolchain is `{{version}}`/' README.md
  sed -i 's/RUST_TOOLCHAIN: "[0-9.][0-9.]*"/RUST_TOOLCHAIN: "{{version}}"/' .github/workflows/ci.yml
  sed -i 's/RUST_TOOLCHAIN: "[0-9.][0-9.]*"/RUST_TOOLCHAIN: "{{version}}"/' .github/workflows/release.yml
  sed -i 's/Toolchain: [0-9.][0-9.]*/Toolchain: {{version}}/' openspec/config.yaml
