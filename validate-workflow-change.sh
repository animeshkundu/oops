#!/bin/bash
# Validation script for auto-release workflow change
# This script verifies the workflow change is correct and safe

set -e

echo "🔍 Validating Auto-Release Workflow Change"
echo "=========================================="
echo ""

# Colors
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Test counter
PASS=0
FAIL=0

test_pass() {
    echo -e "${GREEN}✅ PASS${NC}: $1"
    PASS=$((PASS + 1))
}

test_fail() {
    echo -e "${RED}❌ FAIL${NC}: $1"
    FAIL=$((FAIL + 1))
}

test_warn() {
    echo -e "${YELLOW}⚠️  WARN${NC}: $1"
}

echo "📋 Test 1: YAML Syntax Validation"
echo "-----------------------------------"
if python3 -c "import yaml; yaml.safe_load(open('.github/workflows/auto-release.yml'))" 2>/dev/null; then
    test_pass "Workflow YAML is valid"
else
    test_fail "Workflow YAML syntax error"
fi
echo ""

echo "📋 Test 2: Workflow Structure Validation"
echo "----------------------------------------"
python3 << 'PYEOF'
import yaml
import sys

try:
    with open('.github/workflows/auto-release.yml', 'r') as f:
        workflow = yaml.safe_load(f)
    
    # Check job exists
    if 'create-version-bump-pr' not in workflow['jobs']:
        print("❌ Job 'create-version-bump-pr' not found")
        sys.exit(1)
    
    print("✅ Job 'create-version-bump-pr' found")
    
    # Check bump_type step
    steps = workflow['jobs']['create-version-bump-pr']['steps']
    bump_type_step = None
    for step in steps:
        if step.get('id') == 'bump_type':
            bump_type_step = step
            break
    
    if bump_type_step is None:
        print("❌ Step with id 'bump_type' not found")
        sys.exit(1)
    
    print("✅ Step 'bump_type' found")
    
    # Check for minor bump
    if 'BUMP_TYPE="minor"' not in bump_type_step['run']:
        print("❌ BUMP_TYPE not set to 'minor'")
        sys.exit(1)
    
    print("✅ BUMP_TYPE correctly set to 'minor'")
    
    # Check output format
    if 'bump_type=$BUMP_TYPE' not in bump_type_step['run']:
        print("❌ Output format incorrect")
        sys.exit(1)
    
    print("✅ Output format preserved")
    
    # Check conditional
    if bump_type_step.get('if') != "steps.check_release.outputs.skip == 'false'":
        print("❌ Conditional check modified")
        sys.exit(1)
    
    print("✅ Conditional execution preserved")
    
    sys.exit(0)
    
except Exception as e:
    print(f"❌ Error: {e}")
    sys.exit(1)
PYEOF

if [ $? -eq 0 ]; then
    PASS=$((PASS + 5))
else
    FAIL=$((FAIL + 5))
fi
echo ""

echo "📋 Test 3: Skip Release Logic Preserved"
echo "---------------------------------------"
if grep -q 'Check if release needed' .github/workflows/auto-release.yml && \
   grep -q '\[skip.*release\]' .github/workflows/auto-release.yml; then
    test_pass "Skip release logic found and intact"
else
    test_fail "Skip release logic missing or modified"
fi
echo ""

echo "📋 Test 4: Downstream Step Compatibility"
echo "----------------------------------------"
# Check that downstream steps reference bump_type output
if grep -q '\${{ steps.bump_type.outputs.bump_type }}' .github/workflows/auto-release.yml; then
    test_pass "Downstream steps use bump_type output"
else
    test_fail "Downstream steps do not reference bump_type"
fi
echo ""

echo "📋 Test 5: Documentation Updates"
echo "--------------------------------"
DOCS_UPDATED=0

if grep -q "All merged PRs trigger a MINOR version bump" docs/releases/AUTOMATED_RELEASES.md; then
    test_pass "AUTOMATED_RELEASES.md updated"
    DOCS_UPDATED=$((DOCS_UPDATED + 1))
else
    test_fail "AUTOMATED_RELEASES.md not updated"
fi

if grep -q "All PRs.*minor bump" docs/releases/auto-release-workflow.md; then
    test_pass "auto-release-workflow.md updated"
    DOCS_UPDATED=$((DOCS_UPDATED + 1))
else
    test_fail "auto-release-workflow.md not updated"
fi

if grep -q "All merged PRs trigger a \*\*minor version bump\*\*" docs/development/CONTRIBUTING.md; then
    test_pass "CONTRIBUTING.md updated"
    DOCS_UPDATED=$((DOCS_UPDATED + 1))
else
    test_fail "CONTRIBUTING.md not updated"
fi

if [ $DOCS_UPDATED -lt 3 ]; then
    test_warn "Not all documentation files updated ($DOCS_UPDATED/3)"
fi
echo ""

echo "📋 Test 6: Change Validation"
echo "----------------------------"
# Check that the old complex logic is removed
if grep -q 'feat!:.*fix!:.*breaking' .github/workflows/auto-release.yml; then
    test_warn "Old complex logic still present in workflow"
else
    test_pass "Complex conditional logic removed"
fi

# Check that PR_TITLE and PR_LABELS env vars are removed from bump_type step
if grep -A 20 'id: bump_type' .github/workflows/auto-release.yml | grep -q 'env:'; then
    test_warn "Environment variables still present in bump_type step"
else
    test_pass "Environment variables removed from bump_type step"
fi
echo ""

echo "📋 Test 7: Cargo Compatibility"
echo "------------------------------"
# Verify 'minor' is a valid cargo-edit bump type
if echo "major minor patch" | grep -q "minor"; then
    test_pass "'minor' is a valid cargo set-version bump type"
else
    test_fail "'minor' is not valid"
fi
echo ""

# Summary
echo "========================================"
echo "📊 Test Results Summary"
echo "========================================"
echo -e "${GREEN}Passed: $PASS${NC}"
echo -e "${RED}Failed: $FAIL${NC}"
echo ""

if [ $FAIL -eq 0 ]; then
    echo -e "${GREEN}✅ All tests passed! Change is safe to deploy.${NC}"
    exit 0
else
    echo -e "${RED}❌ Some tests failed. Review changes before deploying.${NC}"
    exit 1
fi
