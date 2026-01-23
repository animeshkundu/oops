#!/bin/bash
#
# Local testing script for version bump logic
# Tests the same logic used in .github/workflows/auto-release.yml
#
# Usage: ./test-version-bump.sh "PR Title" ["label1,label2"]
#

set -e

# Colors for output
GREEN='\033[0;32m'
RED='\033[0;31m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Border constants
BORDER_DOUBLE="═══════════════════════════════════════════════════════════"
BORDER_BOX_TOP="╔${BORDER_DOUBLE}╗"
BORDER_BOX_BOTTOM="╚${BORDER_DOUBLE}╝"

# Function to print test header
print_test_header() {
    local description="$1"
    local pr_title="$2"
    local pr_labels="$3"
    
    echo ""
    echo -e "${BLUE}${BORDER_DOUBLE}${NC}"
    echo -e "${BLUE}Test: $description${NC}"
    echo -e "${BLUE}${BORDER_DOUBLE}${NC}"
    echo "PR Title: $pr_title"
    echo "PR Labels: $pr_labels"
    echo ""
}

# Function to test a PR scenario
test_scenario() {
    local pr_title="$1"
    local pr_labels="$2"
    local expected="$3"
    local description="$4"
    
    print_test_header "$description" "$pr_title" "$pr_labels"
    
    # Run the logic (same as workflow)
    PR_TITLE="$pr_title"
    PR_LABELS="$pr_labels"
    
    # Initialize bump type
    BUMP_TYPE="patch"
    REASON="default (no specific indicators found)"
    
    # Normalize title for case-insensitive matching
    TITLE_LOWER=$(echo "$PR_TITLE" | tr '[:upper:]' '[:lower:]')
    
    # Check for BREAKING CHANGE (highest priority)
    if echo "$TITLE_LOWER" | grep -qE '(^|[^a-z])(feat|fix|chore|docs|style|refactor|perf|test)!\s*(\(|:)'; then
        BUMP_TYPE="major"
        REASON="conventional commit with '!' (breaking change marker)"
        echo "✓ Detected: $REASON"
    elif echo "$TITLE_LOWER" | grep -qE '\bbreaking\s+change\b'; then
        BUMP_TYPE="major"
        REASON="'BREAKING CHANGE' found in title"
        echo "✓ Detected: $REASON"
    elif echo "$TITLE_LOWER" | grep -qE '\[breaking\]'; then
        BUMP_TYPE="major"
        REASON="[breaking] tag in title"
        echo "✓ Detected: $REASON"
    # Check labels for breaking change
    elif echo "$PR_LABELS" | grep -qiE '(breaking|breaking-change)'; then
        BUMP_TYPE="major"
        REASON="'breaking' or 'breaking-change' label"
        echo "✓ Detected: $REASON"
    
    # Check for FEATURE (minor version bump)
    elif echo "$TITLE_LOWER" | grep -qE '(^|[^a-z])feat\s*(\(|:)'; then
        BUMP_TYPE="minor"
        REASON="conventional commit 'feat:' or 'feat(...)'"
        echo "✓ Detected: $REASON"
    elif echo "$TITLE_LOWER" | grep -qE '\[feat\]'; then
        BUMP_TYPE="minor"
        REASON="[feat] tag in title"
        echo "✓ Detected: $REASON"
    elif echo "$PR_LABELS" | grep -qiE '(feature|enhancement)'; then
        BUMP_TYPE="minor"
        REASON="'feature' or 'enhancement' label"
        echo "✓ Detected: $REASON"
    
    # Default to PATCH
    else
        echo "ℹ️  No major or minor indicators found - defaulting to patch bump"
    fi
    
    echo ""
    echo "Result: $BUMP_TYPE"
    echo "Reason: $REASON"
    
    # Validate result
    if [ "$BUMP_TYPE" = "$expected" ]; then
        echo -e "${GREEN}✅ PASS${NC} - Got expected bump type: $expected"
        return 0
    else
        echo -e "${RED}❌ FAIL${NC} - Expected $expected, got $BUMP_TYPE"
        return 1
    fi
}

# Run test suite
echo -e "${YELLOW}${BORDER_BOX_TOP}${NC}"
echo -e "${YELLOW}║         Version Bump Logic Test Suite                    ║${NC}"
echo -e "${YELLOW}${BORDER_BOX_TOP}${NC}"

PASS_COUNT=0
FAIL_COUNT=0

# Test MAJOR bumps
if test_scenario "feat!: add new API" "" "major" "Breaking change with feat!"; then
    PASS_COUNT=$((PASS_COUNT + 1))
else
    FAIL_COUNT=$((FAIL_COUNT + 1))
fi

if test_scenario "fix!: change return type" "" "major" "Breaking change with fix!"; then
    PASS_COUNT=$((PASS_COUNT + 1))
else
    FAIL_COUNT=$((FAIL_COUNT + 1))
fi

if test_scenario "feat: BREAKING CHANGE: new architecture" "" "major" "BREAKING CHANGE in title"; then
    PASS_COUNT=$((PASS_COUNT + 1))
else
    FAIL_COUNT=$((FAIL_COUNT + 1))
fi

if test_scenario "[breaking] Refactor core module" "" "major" "[breaking] tag"; then
    PASS_COUNT=$((PASS_COUNT + 1))
else
    FAIL_COUNT=$((FAIL_COUNT + 1))
fi

if test_scenario "fix: update dependencies" "breaking" "major" "breaking label"; then
    PASS_COUNT=$((PASS_COUNT + 1))
else
    FAIL_COUNT=$((FAIL_COUNT + 1))
fi

if test_scenario "chore: update deps" "breaking-change" "major" "breaking-change label"; then
    PASS_COUNT=$((PASS_COUNT + 1))
else
    FAIL_COUNT=$((FAIL_COUNT + 1))
fi

if test_scenario "FEAT!: Add support for new format" "" "major" "Case insensitive breaking"; then
    PASS_COUNT=$((PASS_COUNT + 1))
else
    FAIL_COUNT=$((FAIL_COUNT + 1))
fi

# Test MINOR bumps
if test_scenario "feat: add new command" "" "minor" "Feature with feat:"; then
    PASS_COUNT=$((PASS_COUNT + 1))
else
    FAIL_COUNT=$((FAIL_COUNT + 1))
fi

if test_scenario "feat(cli): improve output" "" "minor" "Feature with feat(scope)"; then
    PASS_COUNT=$((PASS_COUNT + 1))
else
    FAIL_COUNT=$((FAIL_COUNT + 1))
fi

if test_scenario "[feat] Add new option" "" "minor" "[feat] tag"; then
    PASS_COUNT=$((PASS_COUNT + 1))
else
    FAIL_COUNT=$((FAIL_COUNT + 1))
fi

if test_scenario "Add new feature for aliases" "feature" "minor" "feature label"; then
    PASS_COUNT=$((PASS_COUNT + 1))
else
    FAIL_COUNT=$((FAIL_COUNT + 1))
fi

if test_scenario "Improve user experience" "enhancement" "minor" "enhancement label"; then
    PASS_COUNT=$((PASS_COUNT + 1))
else
    FAIL_COUNT=$((FAIL_COUNT + 1))
fi

if test_scenario "FEAT: Add new functionality" "" "minor" "Case insensitive feat"; then
    PASS_COUNT=$((PASS_COUNT + 1))
else
    FAIL_COUNT=$((FAIL_COUNT + 1))
fi

# Test PATCH bumps (default)
if test_scenario "fix: resolve bug" "" "patch" "Bug fix"; then
    PASS_COUNT=$((PASS_COUNT + 1))
else
    FAIL_COUNT=$((FAIL_COUNT + 1))
fi

if test_scenario "docs: update README" "" "patch" "Documentation"; then
    PASS_COUNT=$((PASS_COUNT + 1))
else
    FAIL_COUNT=$((FAIL_COUNT + 1))
fi

if test_scenario "chore: update dependencies" "" "patch" "Chore"; then
    PASS_COUNT=$((PASS_COUNT + 1))
else
    FAIL_COUNT=$((FAIL_COUNT + 1))
fi

if test_scenario "style: format code" "" "patch" "Style changes"; then
    PASS_COUNT=$((PASS_COUNT + 1))
else
    FAIL_COUNT=$((FAIL_COUNT + 1))
fi

if test_scenario "refactor: clean up code" "" "patch" "Refactor"; then
    PASS_COUNT=$((PASS_COUNT + 1))
else
    FAIL_COUNT=$((FAIL_COUNT + 1))
fi

if test_scenario "test: add more tests" "" "patch" "Tests"; then
    PASS_COUNT=$((PASS_COUNT + 1))
else
    FAIL_COUNT=$((FAIL_COUNT + 1))
fi

if test_scenario "perf: optimize performance" "" "patch" "Performance"; then
    PASS_COUNT=$((PASS_COUNT + 1))
else
    FAIL_COUNT=$((FAIL_COUNT + 1))
fi

if test_scenario "build: update build config" "" "patch" "Build"; then
    PASS_COUNT=$((PASS_COUNT + 1))
else
    FAIL_COUNT=$((FAIL_COUNT + 1))
fi

if test_scenario "ci: update CI config" "" "patch" "CI"; then
    PASS_COUNT=$((PASS_COUNT + 1))
else
    FAIL_COUNT=$((FAIL_COUNT + 1))
fi

if test_scenario "Random PR title" "" "patch" "No conventional format"; then
    PASS_COUNT=$((PASS_COUNT + 1))
else
    FAIL_COUNT=$((FAIL_COUNT + 1))
fi

# Edge cases
if test_scenario "feat:no space after colon" "" "minor" "No space after colon"; then
    PASS_COUNT=$((PASS_COUNT + 1))
else
    FAIL_COUNT=$((FAIL_COUNT + 1))
fi

if test_scenario "  feat: leading spaces  " "" "minor" "Leading/trailing spaces"; then
    PASS_COUNT=$((PASS_COUNT + 1))
else
    FAIL_COUNT=$((FAIL_COUNT + 1))
fi

if test_scenario "prefixfeat: confusing" "" "patch" "feat: not at start (should be patch)"; then
    PASS_COUNT=$((PASS_COUNT + 1))
else
    FAIL_COUNT=$((FAIL_COUNT + 1))
fi

if test_scenario "defeat: contains feat but not feature" "" "patch" "Contains 'feat' but not a feature"; then
    PASS_COUNT=$((PASS_COUNT + 1))
else
    FAIL_COUNT=$((FAIL_COUNT + 1))
fi

# Priority tests (breaking > feature > default)
if test_scenario "feat!: new feature but breaking" "" "major" "Breaking takes priority over feature"; then
    PASS_COUNT=$((PASS_COUNT + 1))
else
    FAIL_COUNT=$((FAIL_COUNT + 1))
fi

if test_scenario "feat: new feature" "breaking" "major" "Breaking label overrides feat:"; then
    PASS_COUNT=$((PASS_COUNT + 1))
else
    FAIL_COUNT=$((FAIL_COUNT + 1))
fi

# Summary
echo ""
echo -e "${YELLOW}${BORDER_BOX_TOP}${NC}"
echo -e "${YELLOW}║                    Test Results                           ║${NC}"
echo -e "${YELLOW}${BORDER_BOX_BOTTOM}${NC}"
echo ""
echo -e "Total Tests: $((PASS_COUNT + FAIL_COUNT))"
echo -e "${GREEN}Passed: $PASS_COUNT${NC}"
echo -e "${RED}Failed: $FAIL_COUNT${NC}"
echo ""

if [ $FAIL_COUNT -eq 0 ]; then
    echo -e "${GREEN}✅ All tests passed!${NC}"
    exit 0
else
    echo -e "${RED}❌ Some tests failed!${NC}"
    exit 1
fi
