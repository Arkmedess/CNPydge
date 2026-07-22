#!/usr/bin/env bash
# Bootstrap script for CNPydge development environment.
# Requires: Python 3.14+, Rust (rustup)
set -euo pipefail

echo "=== CNPydge Bootstrap ==="

# Check prerequisites
command -v python3 >/dev/null 2>&1 || { echo "ERROR: python3 not found"; exit 1; }
command -v cargo >/dev/null 2>&1 || { echo "ERROR: cargo not found. Install Rust: https://rustup.rs"; exit 1; }

PYTHON_VERSION=$(python3 --version 2>&1)
RUST_VERSION=$(rustc --version 2>&1)
echo "Python: $PYTHON_VERSION"
echo "Rust:   $RUST_VERSION"

# Create virtualenv
echo ""
echo "--- Setting up Python virtualenv ---"
if [ ! -d ".venv" ]; then
    python3 -m venv .venv
fi
source .venv/bin/activate

# Install Python dependencies
echo "--- Installing Python dependencies ---"
pip install --upgrade pip
pip install -e ".[dev]"

# Build Rust extension
echo ""
echo "--- Building Rust extension (maturin) ---"
pip install maturin
cd rust/cnpj_core
maturin develop --release
cd ../..

# Run tests
echo ""
echo "--- Running Rust tests ---"
cd rust/cnpj_core
cargo test
cd ../..

echo ""
echo "=== Bootstrap complete ==="
echo "Activate the virtualenv: source .venv/bin/activate"
echo "Run the CLI: python -m python.entrypoints.cli --help"
