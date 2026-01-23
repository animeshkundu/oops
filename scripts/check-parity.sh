#!/bin/bash
# Check parity between oops and thefuck rules
# Usage: ./scripts/check-parity.sh [--days N] [--output json]

set -e

cd "$(dirname "$0")/.."

echo "🔧 Building parity checker..."
cargo build --bin check_parity --release --quiet

echo ""
./target/release/check_parity "$@"
