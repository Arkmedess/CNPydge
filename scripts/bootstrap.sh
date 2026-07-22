#!/usr/bin/env bash
# Bootstrap script for CNPydge development environment.
# Requires: uv (https://docs.astral.sh/uv/), Rust (rustup)
set -euo pipefail

echo "=== CNPydge Bootstrap ==="

# Check prerequisites
command -v uv >/dev/null 2>&1 || { echo "ERROR: uv not found. Install: curl -LsSf https://astral.sh/uv/install.sh | sh"; exit 1; }
command -v cargo >/dev/null 2>&1 || { echo "ERROR: cargo not found. Install Rust: https://rustup.rs"; exit 1; }

UV_VERSION=$(uv --version 2>&1)
RUST_VERSION=$(rustc --version 2>&1)
echo "uv:     $UV_VERSION"
echo "Rust:   $RUST_VERSION"

# Sync Python dependencies (creates .venv if needed, installs from lockfile)
echo ""
echo "--- Syncing Python dependencies ---"
uv sync

# Build Rust extension
echo ""
echo "--- Building Rust extension (maturin) ---"
uv run maturin develop --release

# Run Rust tests
echo ""
echo "--- Running Rust tests ---"
cd rust/cnpj_core
cargo test
cd ../..

echo ""
echo "=== Bootstrap complete ==="
echo "Run the CLI: uv run python -m python.entrypoints.cli --help"
echo "Run tests:   uv run pytest"
