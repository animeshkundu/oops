#!/bin/bash
# Test script for release workflow fix
# This script helps verify that the release workflow triggers correctly

set -e

echo "🧪 Release Workflow Test Script"
echo "================================"
echo ""

# Check if we're in the right directory
if [ ! -f ".github/workflows/release.yml" ]; then
    echo "❌ Error: Must be run from repository root"
    exit 1
fi

echo "📋 Pre-flight Checks:"
echo "-------------------"

# Check if PAT is likely configured (we can't check the actual secret)
echo "ℹ️  Note: This script cannot verify if RELEASE_PAT secret is configured."
echo "   Repository maintainers should verify in Settings → Secrets → Actions"
echo ""

# Check workflow files exist
echo "✅ Checking workflow files..."
if [ -f ".github/workflows/auto-release.yml" ]; then
    echo "   ✓ auto-release.yml found"
else
    echo "   ✗ auto-release.yml NOT found"
    exit 1
fi

if [ -f ".github/workflows/release.yml" ]; then
    echo "   ✓ release.yml found"
else
    echo "   ✗ release.yml NOT found"
    exit 1
fi

# Check if the workflow uses RELEASE_PAT
echo ""
echo "✅ Checking PAT configuration in auto-release.yml..."
if grep -q "RELEASE_PAT" ".github/workflows/auto-release.yml"; then
    echo "   ✓ RELEASE_PAT reference found"
else
    echo "   ✗ RELEASE_PAT NOT found in workflow"
    exit 1
fi

# Check YAML syntax
echo ""
echo "✅ Validating YAML syntax..."
if command -v python3 &> /dev/null; then
    if python3 -c "import yaml; yaml.safe_load(open('.github/workflows/auto-release.yml'))" 2>/dev/null; then
        echo "   ✓ auto-release.yml is valid YAML"
    else
        echo "   ✗ auto-release.yml has YAML syntax errors"
        exit 1
    fi
    
    if python3 -c "import yaml; yaml.safe_load(open('.github/workflows/release.yml'))" 2>/dev/null; then
        echo "   ✓ release.yml is valid YAML"
    else
        echo "   ✗ release.yml has YAML syntax errors"
        exit 1
    fi
else
    echo "   ⚠️  Python3 not available, skipping YAML validation"
fi

echo ""
echo "📊 Test Options:"
echo "---------------"
echo ""
echo "Choose a test option:"
echo ""
echo "1) Push existing tag v0.1.2 (will trigger release workflow)"
echo "2) Push existing tag v0.1.1 (will trigger release workflow)"
echo "3) Create and push test tag v0.1.3-test"
echo "4) Just show existing tags (no push)"
echo "5) Exit"
echo ""

read -p "Enter option (1-5): " option

case $option in
    1)
        echo ""
        echo "🚀 Pushing tag v0.1.2..."
        git push origin v0.1.2
        echo ""
        echo "✅ Tag pushed! Check GitHub Actions:"
        echo "   https://github.com/$(git config --get remote.origin.url | sed 's/.*github.com[:/]\(.*\)\.git/\1/')/actions"
        ;;
    2)
        echo ""
        echo "🚀 Pushing tag v0.1.1..."
        git push origin v0.1.1
        echo ""
        echo "✅ Tag pushed! Check GitHub Actions:"
        echo "   https://github.com/$(git config --get remote.origin.url | sed 's/.*github.com[:/]\(.*\)\.git/\1/')/actions"
        ;;
    3)
        echo ""
        echo "🏷️  Creating test tag v0.1.3-test..."
        git tag v0.1.3-test
        echo "🚀 Pushing tag v0.1.3-test..."
        git push origin v0.1.3-test
        echo ""
        echo "✅ Test tag created and pushed! Check GitHub Actions:"
        echo "   https://github.com/$(git config --get remote.origin.url | sed 's/.*github.com[:/]\(.*\)\.git/\1/')/actions"
        ;;
    4)
        echo ""
        echo "📋 Existing tags:"
        git tag -l
        echo ""
        echo "💡 To manually test, run:"
        echo "   git push origin v0.1.2"
        ;;
    5)
        echo ""
        echo "👋 Exiting without changes"
        exit 0
        ;;
    *)
        echo ""
        echo "❌ Invalid option"
        exit 1
        ;;
esac

echo ""
echo "📝 What to check:"
echo "----------------"
echo "1. Go to Actions tab in GitHub"
echo "2. Look for 'Release' workflow run"
echo "3. Verify it's triggered by tag push"
echo "4. Wait for it to complete (~10-15 minutes)"
echo "5. Check Releases page for new release with binaries"
echo ""
echo "📚 More info: see docs/RELEASE_PAT_SETUP.md"
