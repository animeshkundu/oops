#!/bin/bash
# Quick fix script for CI pipeline issues
# Run this after applying the code changes

set -e

echo "🔧 Applying CI Pipeline Fixes..."
echo ""

# Step 1: Generate Cargo.lock if missing
if [ ! -f "Cargo.lock" ]; then
    echo "📦 Generating Cargo.lock..."
    cargo generate-lockfile
    echo "✅ Cargo.lock generated"
else
    echo "✅ Cargo.lock already exists"
fi

echo ""

# Step 2: Verify the fixes compile
echo "🔨 Building project..."
cargo build --all-features
echo "✅ Build successful"

echo ""

# Step 3: Run clippy on all targets
echo "📎 Running clippy checks..."
cargo clippy --all-targets --all-features -- -D warnings
echo "✅ Clippy passed"

echo ""

# Step 4: Run tests
echo "🧪 Running tests..."
cargo test --all-features
echo "✅ Tests passed"

echo ""

# Step 5: Test audit workflow
echo "🔒 Testing security audit..."
if command -v cargo-audit &> /dev/null; then
    cargo audit
    echo "✅ Audit passed"
else
    echo "⚠️  cargo-audit not installed, installing..."
    cargo install cargo-audit --locked
    cargo audit
    echo "✅ Audit passed"
fi

echo ""
echo "🎉 All fixes validated successfully!"
echo ""
echo "Next steps:"
echo "  1. Commit Cargo.lock: git add Cargo.lock && git commit -m 'chore: add Cargo.lock'"
echo "  2. Push changes: git push origin copilot/fix-ci-cd-pipeline-issues"
echo "  3. Monitor CI runs in GitHub Actions"
