#!/bin/bash
#
# Release Workflow Testing Script
#
# This script helps verify both automated PR-merge and manual tag release paths.
# Run with: ./scripts/test-release-verification.sh [automated|manual|both]
#

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Repository info
REPO_OWNER="animeshkundu"
REPO_NAME="oops"
REPO="${REPO_OWNER}/${REPO_NAME}"

print_header() {
    echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo -e "${BLUE}$1${NC}"
    echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
}

print_success() {
    echo -e "${GREEN}✅ $1${NC}"
}

print_error() {
    echo -e "${RED}❌ $1${NC}"
}

print_warning() {
    echo -e "${YELLOW}⚠️  $1${NC}"
}

print_info() {
    echo -e "${BLUE}ℹ️  $1${NC}"
}

check_command() {
    if ! command -v $1 &> /dev/null; then
        print_error "$1 not found. Please install it first."
        exit 1
    fi
}

# Check prerequisites
check_prerequisites() {
    print_header "Checking Prerequisites"
    
    check_command "git"
    check_command "gh"
    check_command "jq"
    check_command "cargo"
    
    # Check if we're in the right directory
    if [ ! -f "Cargo.toml" ]; then
        print_error "Not in repository root. Please run from repository root."
        exit 1
    fi
    
    # Check if gh is authenticated
    if ! gh auth status &> /dev/null; then
        print_error "GitHub CLI not authenticated. Run: gh auth login"
        exit 1
    fi
    
    print_success "All prerequisites met"
}

# Get current version from Cargo.toml
get_current_version() {
    cargo metadata --no-deps --format-version 1 | jq -r '.packages[] | select(.name == "oops") | .version'
}

# Test automated release path
test_automated_release() {
    print_header "Testing Automated PR Merge Release Path"
    
    CURRENT_VERSION=$(get_current_version)
    print_info "Current version: $CURRENT_VERSION"
    
    # Calculate next versions
    IFS='.' read -ra VERSION_PARTS <<< "$CURRENT_VERSION"
    MAJOR=${VERSION_PARTS[0]}
    MINOR=${VERSION_PARTS[1]}
    PATCH=${VERSION_PARTS[2]}
    
    NEXT_PATCH="$MAJOR.$MINOR.$((PATCH + 1))"
    NEXT_MINOR="$MAJOR.$((MINOR + 1)).0"
    NEXT_MAJOR="$((MAJOR + 1)).0.0"
    
    echo ""
    print_info "Testing patch release (fix: prefix)"
    echo "Expected next version: $NEXT_PATCH"
    echo ""
    
    # Create test branch
    TEST_BRANCH="test-automated-release-$(date +%s)"
    print_info "Creating test branch: $TEST_BRANCH"
    
    git checkout -b "$TEST_BRANCH"
    
    # Make a small change
    echo "# Test automated release" >> README.md
    git add README.md
    git commit -m "fix: test automated release workflow"
    
    print_info "Pushing test branch..."
    git push origin "$TEST_BRANCH"
    
    # Create PR
    print_info "Creating pull request..."
    PR_URL=$(gh pr create \
        --title "fix: test automated release workflow" \
        --body "This is a test PR to verify the automated release workflow.

**Expected behavior:**
1. When this PR is merged, auto-release workflow should trigger
2. A version bump PR should be created for v$NEXT_PATCH
3. When version bump PR is merged, tag should be created
4. Release workflow should build binaries
5. GitHub release should be published

**Testing**: This is a test PR and can be closed without merging if needed." \
        --base master \
        --head "$TEST_BRANCH")
    
    print_success "Test PR created: $PR_URL"
    
    echo ""
    print_warning "Manual steps required:"
    echo "1. Review the PR: $PR_URL"
    echo "2. Merge the PR (or close if you don't want to test right now)"
    echo "3. Monitor workflows:"
    echo "   - Auto Release: https://github.com/$REPO/actions/workflows/auto-release.yml"
    echo "   - Create Release Tag: https://github.com/$REPO/actions/workflows/create-release-tag.yml"
    echo "   - Release: https://github.com/$REPO/actions/workflows/release.yml"
    echo "4. Check for version bump PR"
    echo "5. Merge version bump PR when CI passes"
    echo "6. Verify release published: https://github.com/$REPO/releases"
    echo ""
    echo "Expected release version: v$NEXT_PATCH"
}

# Test manual tag release
test_manual_release() {
    print_header "Testing Manual Tag Release Path"
    
    CURRENT_VERSION=$(get_current_version)
    print_info "Current version: $CURRENT_VERSION"
    
    # Calculate next version (patch bump for manual test)
    IFS='.' read -ra VERSION_PARTS <<< "$CURRENT_VERSION"
    MAJOR=${VERSION_PARTS[0]}
    MINOR=${VERSION_PARTS[1]}
    PATCH=${VERSION_PARTS[2]}
    
    NEXT_VERSION="$MAJOR.$MINOR.$((PATCH + 1))"
    TAG="v$NEXT_VERSION"
    
    echo ""
    print_warning "This will create a real release!"
    read -p "Do you want to proceed with manual tag release test? (yes/no): " CONFIRM
    
    if [ "$CONFIRM" != "yes" ]; then
        print_info "Manual tag release test cancelled"
        return
    fi
    
    # Update version in Cargo.toml
    print_info "Updating Cargo.toml to version $NEXT_VERSION..."
    
    # Use cargo-edit if available, otherwise manual edit
    if command -v cargo-set-version &> /dev/null; then
        cargo set-version "$NEXT_VERSION"
    else
        print_warning "cargo-edit not installed. Manual version update required."
        echo "Please update Cargo.toml version to $NEXT_VERSION and press Enter to continue..."
        read
    fi
    
    # Update Cargo.lock
    cargo update -p oops
    
    # Commit version bump
    git add Cargo.toml Cargo.lock
    git commit -m "chore: bump version to $NEXT_VERSION (manual release test)"
    
    print_info "Pushing version bump to master..."
    git push origin master
    
    # Create and push tag
    print_info "Creating tag $TAG..."
    git tag -a "$TAG" -m "Release $TAG (manual test)

This is a manual release test to verify the release workflow.

Built binaries:
- Linux x86_64 (glibc)
- Linux x86_64 (musl - static)
- Linux ARM64
- macOS x86_64 (Intel)
- macOS ARM64 (Apple Silicon)
- Windows x86_64"
    
    print_info "Pushing tag $TAG..."
    git push origin "$TAG"
    
    print_success "Tag $TAG created and pushed!"
    
    echo ""
    print_info "Monitor release workflow:"
    echo "   https://github.com/$REPO/actions/workflows/release.yml"
    echo ""
    echo "Expected actions:"
    echo "1. Release workflow should trigger automatically"
    echo "2. All 6 binaries should build"
    echo "3. GitHub release should be published"
    echo "4. Release link: https://github.com/$REPO/releases/tag/$TAG"
    echo ""
    echo "Timeline: ~10-15 minutes for complete release"
}

# Verify existing releases
verify_releases() {
    print_header "Verifying Existing Releases"
    
    print_info "Fetching releases from GitHub..."
    RELEASES=$(gh release list --limit 5)
    
    if [ -z "$RELEASES" ]; then
        print_warning "No releases found"
        return
    fi
    
    echo ""
    echo "$RELEASES"
    echo ""
    
    # Get latest release
    LATEST=$(gh release view --json tagName,assets | jq -r '.tagName')
    print_info "Latest release: $LATEST"
    
    # Check assets
    print_info "Checking release assets for $LATEST..."
    ASSETS=$(gh release view "$LATEST" --json assets | jq -r '.assets[].name')
    
    EXPECTED_ASSETS=(
        "oops-linux-x86_64"
        "oops-linux-x86_64.sha256"
        "oops-linux-x86_64-musl"
        "oops-linux-x86_64-musl.sha256"
        "oops-linux-aarch64"
        "oops-linux-aarch64.sha256"
        "oops-darwin-x86_64"
        "oops-darwin-x86_64.sha256"
        "oops-darwin-aarch64"
        "oops-darwin-aarch64.sha256"
        "oops-windows-x86_64.exe"
        "oops-windows-x86_64.exe.sha256"
    )
    
    MISSING_ASSETS=()
    for ASSET in "${EXPECTED_ASSETS[@]}"; do
        if echo "$ASSETS" | grep -q "$ASSET"; then
            print_success "Found: $ASSET"
        else
            print_error "Missing: $ASSET"
            MISSING_ASSETS+=("$ASSET")
        fi
    done
    
    if [ ${#MISSING_ASSETS[@]} -eq 0 ]; then
        print_success "All expected assets present!"
    else
        print_error "Missing ${#MISSING_ASSETS[@]} assets"
    fi
}

# Check workflow status
check_workflow_status() {
    print_header "Checking Workflow Status"
    
    print_info "Auto Release workflow runs:"
    gh run list --workflow=auto-release.yml --limit 5
    
    echo ""
    print_info "Create Release Tag workflow runs:"
    gh run list --workflow=create-release-tag.yml --limit 5
    
    echo ""
    print_info "Release workflow runs:"
    gh run list --workflow=release.yml --limit 5
}

# Main menu
show_menu() {
    echo ""
    print_header "Release Workflow Testing Menu"
    echo ""
    echo "1. Test automated PR merge release path"
    echo "2. Test manual tag release path"
    echo "3. Verify existing releases"
    echo "4. Check workflow status"
    echo "5. Run all verifications"
    echo "6. Exit"
    echo ""
}

# Main script
main() {
    print_header "Release Workflow Testing Script"
    print_info "Repository: $REPO"
    echo ""
    
    check_prerequisites
    
    if [ $# -eq 1 ]; then
        case $1 in
            automated)
                test_automated_release
                ;;
            manual)
                test_manual_release
                ;;
            verify)
                verify_releases
                ;;
            status)
                check_workflow_status
                ;;
            both)
                test_automated_release
                echo ""
                test_manual_release
                ;;
            *)
                echo "Usage: $0 [automated|manual|verify|status|both]"
                exit 1
                ;;
        esac
    else
        # Interactive mode
        while true; do
            show_menu
            read -p "Choose an option (1-6): " CHOICE
            
            case $CHOICE in
                1)
                    test_automated_release
                    ;;
                2)
                    test_manual_release
                    ;;
                3)
                    verify_releases
                    ;;
                4)
                    check_workflow_status
                    ;;
                5)
                    verify_releases
                    check_workflow_status
                    ;;
                6)
                    print_info "Exiting..."
                    exit 0
                    ;;
                *)
                    print_error "Invalid option"
                    ;;
            esac
            
            echo ""
            read -p "Press Enter to continue..."
        done
    fi
}

# Run main function
main "$@"
