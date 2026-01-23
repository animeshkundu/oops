#!/bin/bash
# Test script to simulate the build-and-release workflow

set -e

echo "=================================================="
echo "Testing Build and Release Workflow"
echo "=================================================="
echo ""

# Test 1: Verify workflow file exists and is valid YAML
echo "✅ Test 1: Workflow file exists and is valid YAML"
python3 -c "import yaml; yaml.safe_load(open('.github/workflows/build-and-release.yml'))"
echo "   PASSED: Workflow YAML is valid"
echo ""

# Test 2: Run formatting check
echo "✅ Test 2: Check code formatting"
cargo fmt --check
echo "   PASSED: Code formatting is correct"
echo ""

# Test 3: Run clippy
echo "✅ Test 3: Run Clippy lints"
cargo clippy -- -D warnings > /dev/null 2>&1
echo "   PASSED: No clippy warnings"
echo ""

# Test 4: Run tests
echo "✅ Test 4: Run test suite"
cargo test > /dev/null 2>&1
echo "   PASSED: All tests passed"
echo ""

# Test 5: Build release binary
echo "✅ Test 5: Build release binary"
cargo build --release --target x86_64-unknown-linux-gnu > /dev/null 2>&1
echo "   PASSED: Release build succeeded"
echo ""

# Test 6: Verify binary exists
echo "✅ Test 6: Verify binary exists"
if [ -f "target/x86_64-unknown-linux-gnu/release/oops" ]; then
    echo "   PASSED: Binary exists at expected location"
else
    echo "   FAILED: Binary not found"
    exit 1
fi
echo ""

# Test 7: Test binary execution
echo "✅ Test 7: Test binary execution"
./target/x86_64-unknown-linux-gnu/release/oops --version > /dev/null 2>&1
echo "   PASSED: Binary executes successfully"
echo ""

# Test 8: Test checksum generation
echo "✅ Test 8: Test checksum generation"
cp target/x86_64-unknown-linux-gnu/release/oops test-oops-binary
sha256sum test-oops-binary > test-oops-binary.sha256
sha256sum -c test-oops-binary.sha256 > /dev/null 2>&1
rm -f test-oops-binary test-oops-binary.sha256
echo "   PASSED: Checksum generation and verification works"
echo ""

echo "=================================================="
echo "✅ All workflow tests passed!"
echo "=================================================="
echo ""
echo "The workflow is ready to use. You can:"
echo "1. Trigger it manually via workflow_dispatch with a tag"
echo "2. It will run on push to any branch (builds only)"
echo "3. It will run on pull_request to any branch (builds only)"
echo ""
echo "To test the full workflow with release creation:"
echo "  gh workflow run build-and-release.yml -f tag=v0.1.1-test"
echo ""
