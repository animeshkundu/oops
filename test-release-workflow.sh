#!/bin/bash
set -e

echo "🧪 Testing Release Workflow Logic"
echo "=================================="

# Test 1: Check cargo-edit is available (required for version bumping)
echo ""
echo "Test 1: Checking if cargo-edit tools are available..."
if ! command -v cargo-set-version &> /dev/null; then
    echo "⚠️  cargo-edit not installed. Installing..."
    cargo install cargo-edit --version 0.12.2
fi
echo "✅ cargo-edit is available"

# Test 2: Verify we can read current version
echo ""
echo "Test 2: Reading current version from Cargo.toml..."
CURRENT_VERSION=$(cargo metadata --no-deps --format-version 1 | jq -r '.packages[0].version')
echo "📦 Current version: $CURRENT_VERSION"
if [ -z "$CURRENT_VERSION" ]; then
    echo "❌ Failed to read version"
    exit 1
fi
echo "✅ Version read successfully"

# Test 3: Test version bump (in dry-run mode)
echo ""
echo "Test 3: Testing version bump logic..."
echo "Original version: $CURRENT_VERSION"

# Save original Cargo.toml and Cargo.lock
cp Cargo.toml Cargo.toml.backup
cp Cargo.lock Cargo.lock.backup

# Test patch bump
echo "Testing patch bump..."
cargo set-version --bump patch
NEW_VERSION=$(cargo metadata --no-deps --format-version 1 | jq -r '.packages[0].version')
echo "After patch bump: $NEW_VERSION"

if [ "$CURRENT_VERSION" = "$NEW_VERSION" ]; then
    echo "❌ Version did not change after patch bump"
    exit 1
fi

# Restore original files
mv Cargo.toml.backup Cargo.toml
mv Cargo.lock.backup Cargo.lock

echo "✅ Version bump logic works"

# Test 4: Verify git operations
echo ""
echo "Test 4: Testing git operations..."
git config --local user.email "test@example.com"
git config --local user.name "Test Bot"
echo "✅ Git configuration works"

# Test 5: Check if jq is available (required for parsing metadata)
echo ""
echo "Test 5: Checking if jq is available..."
if ! command -v jq &> /dev/null; then
    echo "❌ jq is not installed (required for parsing JSON)"
    exit 1
fi
echo "✅ jq is available"

# Test 6: Verify workflow files are valid YAML
echo ""
echo "Test 6: Validating workflow YAML syntax..."
python3 -c "import yaml; yaml.safe_load(open('.github/workflows/auto-release.yml'))"
echo "✅ auto-release.yml is valid"
python3 -c "import yaml; yaml.safe_load(open('.github/workflows/release.yml'))"
echo "✅ release.yml is valid"

# Test 7: Check if we can build the project
echo ""
echo "Test 7: Testing if project builds..."
if cargo build --quiet 2>&1; then
    echo "✅ Project builds successfully"
else
    echo "❌ Project build failed"
    exit 1
fi

echo ""
echo "=================================="
echo "🎉 All tests passed!"
echo ""
echo "Summary:"
echo "- cargo-edit tools: ✅"
echo "- Version reading: ✅"
echo "- Version bumping: ✅"
echo "- Git operations: ✅"
echo "- jq availability: ✅"
echo "- YAML validation: ✅"
echo "- Project build: ✅"
echo ""
echo "The workflow should work correctly in GitHub Actions."
