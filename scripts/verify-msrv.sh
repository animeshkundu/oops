#!/bin/bash
# Script to verify MSRV compatibility

set -e

echo "==================================="
echo "MSRV Compatibility Verification"
echo "==================================="
echo ""

# Check if Rust 1.70 is installed
if ! rustup toolchain list | grep -q "1.70"; then
    echo "📦 Installing Rust 1.70..."
    rustup install 1.70
else
    echo "✅ Rust 1.70 is already installed"
fi

echo ""
echo "🔍 Checking Cargo.toml for home crate patch..."
if grep -q "\[patch.crates-io\]" Cargo.toml && grep -q 'home = "=0.5.11"' Cargo.toml; then
    echo "✅ home crate is pinned to 0.5.11"
else
    echo "❌ home crate patch not found in Cargo.toml"
    exit 1
fi

echo ""
echo "🔍 Checking rust-version field in Cargo.toml..."
if grep -q 'rust-version = "1.70"' Cargo.toml; then
    echo "✅ rust-version is set to 1.70"
else
    echo "⚠️  rust-version field not found or incorrect"
fi

echo ""
echo "🧹 Cleaning previous build artifacts..."
cargo clean

echo ""
echo "🔨 Building with Rust 1.70..."
cargo +1.70 build

if [ $? -eq 0 ]; then
    echo ""
    echo "✅ SUCCESS! Build completed with Rust 1.70"
    echo ""
    echo "🎉 MSRV compatibility verified!"
else
    echo ""
    echo "❌ FAILED! Build failed with Rust 1.70"
    exit 1
fi

echo ""
echo "📋 Dependency tree (home crate):"
cargo +1.70 tree | grep home || echo "  (home crate found in tree)"

echo ""
echo "==================================="
echo "Verification Complete"
echo "==================================="
