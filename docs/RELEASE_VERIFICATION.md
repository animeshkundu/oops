# Release Workflow Verification Guide

This document provides comprehensive verification for both automated PR-merge releases and manual tag releases.

## Table of Contents

1. [Release Flow Overview](#release-flow-overview)
2. [Automated PR Merge Release (Primary)](#automated-pr-merge-release-primary)
3. [Manual Tag Release (Backup/Override)](#manual-tag-release-backupoverride)
4. [Version Bump Logic](#version-bump-logic)
5. [Testing Both Paths](#testing-both-paths)
6. [Troubleshooting](#troubleshooting)

---

## Release Flow Overview

### Path 1: Automated PR Merge Release (Recommended)

```
PR Merged
    ↓
auto-release.yml triggers
    ↓
Determines version bump (patch/minor/major)
    ↓
Creates version bump PR
    ↓
Version bump PR merged
    ↓
create-release-tag.yml triggers
    ↓
Creates git tag (e.g., v0.1.2)
    ↓
release.yml triggers
    ↓
Builds 6 platform binaries
    ↓
Creates GitHub Release with executables
```

**Timeline**: ~20-25 minutes from PR merge to published release

### Path 2: Manual Tag Release (Backup)

```
Manual tag creation
    ↓
git tag v0.1.3
git push origin v0.1.3
    ↓
release.yml triggers
    ↓
Builds 6 platform binaries
    ↓
Creates GitHub Release with executables
```

**Timeline**: ~10-15 minutes from tag push to published release

---

## Automated PR Merge Release (Primary)

### How It Works

1. **Trigger**: PR is merged to `main` or `master` branch
2. **Workflow**: `.github/workflows/auto-release.yml`
3. **Version Determination**: Based on PR title/labels (see [Version Bump Logic](#version-bump-logic))
4. **Result**: Creates version bump PR automatically

### Requirements

✅ **For Basic Operation** (Works with `GITHUB_TOKEN`):
- PR merged to `main` or `master`
- PR title doesn't contain `[skip release]` or `[no release]`

✅ **For Fully Automated Operation** (Requires `RELEASE_PAT`):
- All basic requirements above
- `RELEASE_PAT` secret configured (see [PAT Setup](#pat-setup))

### Workflow Steps

#### Step 1: Auto-Release Workflow

**Trigger**: PR closed + merged

**Actions**:
1. ✅ Runs tests on all platforms (Linux, macOS, Windows)
2. ✅ Determines version bump type from PR title/labels
3. ✅ Bumps version in `Cargo.toml` and `Cargo.lock`
4. ✅ Creates version bump branch (e.g., `release/v0.1.2`)
5. ✅ Creates version bump PR with "release" and "automated" labels
6. ✅ (Optional) Enables auto-merge if `RELEASE_PAT` is configured

**Output**: Version bump PR ready for review/merge

#### Step 2: Version Bump PR Merged

**Actions**:
- CI runs on version bump PR
- If auto-merge enabled: Merges automatically when CI passes
- If manual: Maintainer reviews and merges

#### Step 3: Create Release Tag Workflow

**Trigger**: PR with "release" label + title starting with "chore: release" + merged

**Actions**:
1. ✅ Extracts version from `Cargo.toml`
2. ✅ Verifies tag doesn't already exist
3. ✅ Creates annotated git tag (e.g., `v0.1.2`)
4. ✅ Pushes tag to GitHub
5. ✅ Triggers release workflow

**Output**: Git tag created, release workflow triggered

#### Step 4: Release Workflow

**Trigger**: Tag push (e.g., `v0.1.2`)

**Actions**:
1. ✅ Runs tests
2. ✅ Builds binaries for 6 targets:
   - Linux x86_64 (glibc)
   - Linux x86_64 (musl - static)
   - Linux ARM64
   - macOS x86_64 (Intel)
   - macOS ARM64 (Apple Silicon)
   - Windows x86_64
3. ✅ Generates SHA256 checksums for each binary
4. ✅ Verifies checksums
5. ✅ Creates GitHub Release with all artifacts
6. ✅ Auto-generates release notes from PRs

**Output**: GitHub Release published with 6 binaries + checksums ready to download

### What Users See

After the full process completes, users can:

1. Go to https://github.com/animeshkundu/oops/releases
2. See the latest release (e.g., v0.1.2)
3. Download binaries for their platform:
   - `oops-linux-x86_64` (glibc)
   - `oops-linux-x86_64-musl` (static)
   - `oops-linux-aarch64` (ARM64)
   - `oops-darwin-x86_64` (Intel Mac)
   - `oops-darwin-aarch64` (Apple Silicon)
   - `oops-windows-x86_64.exe`
4. Download corresponding `.sha256` files for verification
5. See auto-generated release notes from merged PRs

---

## Manual Tag Release (Backup/Override)

### When to Use Manual Release

- **Hotfix**: Need to release immediately without waiting for auto-process
- **Override**: Auto-release failed or was skipped
- **Testing**: Want to test release workflow independently
- **Backport**: Creating release from an older commit/branch

### How to Create Manual Release

#### Method 1: Via Git Command Line

```bash
# 1. Ensure your Cargo.toml has the correct version
# Edit Cargo.toml if needed
vim Cargo.toml  # Change version = "0.1.3"

# 2. Update Cargo.lock
cargo update -p oops

# 3. Commit version changes
git add Cargo.toml Cargo.lock
git commit -m "chore: bump version to 0.1.3"
git push origin master

# 4. Create and push tag
git tag -a v0.1.3 -m "Release v0.1.3

Manual release for [reason].

Built binaries:
- Linux x86_64 (glibc)
- Linux x86_64 (musl)
- Linux ARM64
- macOS x86_64
- macOS ARM64
- Windows x86_64"

# 5. Push the tag
git push origin v0.1.3
```

#### Method 2: Via GitHub UI (Manual Workflow Dispatch)

**Note**: This requires `workflow_dispatch` to be added to `release.yml` (see [Enhancement](#enhancements-needed))

```
1. Go to Actions → Release workflow
2. Click "Run workflow"
3. Select branch/tag
4. Enter tag name (e.g., v0.1.3)
5. Click "Run workflow"
```

### Manual Release Verification

After pushing the tag, verify:

1. **Tag exists**:
   ```bash
   git ls-remote --tags origin | grep v0.1.3
   ```

2. **Release workflow triggered**:
   - Go to https://github.com/animeshkundu/oops/actions/workflows/release.yml
   - Look for a run triggered by tag `v0.1.3`
   - Status should be "in progress" or "success"

3. **Release created** (after ~10-15 minutes):
   - Go to https://github.com/animeshkundu/oops/releases
   - See release `v0.1.3` with 6 binaries + checksums

### Important Notes for Manual Release

⚠️ **PAT Requirement**: If you push a tag using `GITHUB_TOKEN` (default in workflows), the release workflow **will NOT trigger**. You need either:
- Push the tag manually via git CLI (uses your personal credentials)
- Configure `RELEASE_PAT` secret and use it in workflows
- Manually trigger the workflow via workflow_dispatch (requires enhancement)

✅ **Version Match**: The version in `Cargo.toml` MUST match the tag version:
- Tag `v0.1.3` → `Cargo.toml` must have `version = "0.1.3"`
- Release workflow verifies this and fails if mismatch detected

---

## Version Bump Logic

The `auto-release.yml` workflow creates a **MINOR version bump for every merged PR**.

### Minor Bump (0.1.1 → 0.2.0) - DEFAULT FOR ALL PRS

**Triggered by**:
- **ALL merged PRs** automatically create a minor version bump
- This includes any PR title or type:
  - `feat: new feature` → 0.1.1 → 0.2.0
  - `fix: bug fix` → 0.1.1 → 0.2.0
  - `docs: update readme` → 0.1.1 → 0.2.0
  - `chore: update deps` → 0.1.1 → 0.2.0
  - Any other PR type → 0.1.1 → 0.2.0

**Rationale**: For a pre-1.0 project, all changes are treated as minor updates to keep version progression simple and predictable.

### Skip Release

**Triggered by**:
- PR title contains `[skip release]` or `[no release]`

**Examples**:
- `docs: update readme [skip release]`
- `chore: fix typo [no release]`

### Code Reference

See `.github/workflows/auto-release.yml` lines 143-162:

```yaml
# Default to patch bump
BUMP_TYPE="patch"

# Check for breaking change indicators (major bump)
if echo "$PR_TITLE" | grep -qiE "^(feat!|fix!):" || echo "$PR_TITLE" | grep -qiE "breaking" || echo "$PR_LABELS" | grep -qi "breaking"; then
  BUMP_TYPE="major"
# Check for feature indicators (minor bump)
elif echo "$PR_TITLE" | grep -qiE "^feat:" || echo "$PR_LABELS" | grep -qiE "(feature|enhancement)"; then
  BUMP_TYPE="minor"
fi
```

---

## Testing Both Paths

### Test Plan: Automated PR Merge Release

#### Test 1: Any PR (Minor Bump - Default Behavior)

```bash
# 1. Create test branch
git checkout -b test-automated-release
echo "# Test Change" >> README.md
git commit -am "fix: test automated release"
git push origin test-automated-release

# 2. Create PR via GitHub UI
# Title: "fix: test automated release" (or any title)

# 3. Merge PR

# 4. Verify auto-release workflow
# Go to: Actions → Auto Release
# Should show: Running or Success
# Should create: Version bump PR (e.g., chore: release v0.2.0)

# 5. Merge version bump PR

# 6. Verify create-release-tag workflow
# Go to: Actions → Create Release Tag
# Should show: Success
# Should create: Tag v0.2.0

# 7. Verify release workflow
# Go to: Actions → Release
# Should show: Running or Success

# 8. Verify release published
# Go to: Releases
# Should see: v0.2.0 with 6 binaries + checksums
```

#### Test 2: Skip Release

```bash
# 1. Create test branch
git checkout -b test-skip-release
echo "Doc update" >> README.md
git commit -am "docs: update readme [skip release]"
git push origin test-skip-release

# 2. Create and merge PR
# Title: "docs: update readme [skip release]"

# 3. Verify auto-release workflow
# Should show: Success (event-guard succeeds, other jobs skipped)
# Should NOT create: Version bump PR
```

### Test Plan: Manual Tag Release

#### Test 1: Manual Tag from Master

```bash
# 1. Update version
vim Cargo.toml  # Change to 0.1.4
cargo update -p oops
git add Cargo.toml Cargo.lock
git commit -m "chore: bump version to 0.1.4"
git push origin master

# 2. Create and push tag
git tag -a v0.1.4 -m "Release v0.1.4 (manual test)"
git push origin v0.1.4

# 3. Verify release workflow triggered
# Go to: Actions → Release
# Should see: Run triggered by tag v0.1.4
# Status: Running or Success

# 4. Verify release published
# Go to: Releases
# Should see: v0.1.4 with 6 binaries + checksums
```

#### Test 2: Manual Tag with Wrong Version (Should Fail)

```bash
# 1. Create tag without updating Cargo.toml
git tag -a v0.1.5 -m "Test version mismatch"
git push origin v0.1.5

# 2. Verify release workflow
# Go to: Actions → Release
# Should see: Failed at "Verify Cargo.toml version matches tag"
# Error: "Version mismatch! Cargo.toml has 0.1.4 but tag is 0.1.5"

# 3. Fix by deleting tag
git tag -d v0.1.5
git push --delete origin v0.1.5
```

---

## PAT Setup

For fully automated releases, configure a Personal Access Token (PAT).

### Why PAT is Needed

**GITHUB_TOKEN Limitation**: 
- Actions performed with `GITHUB_TOKEN` do NOT trigger other workflows
- This is by design to prevent infinite loops
- When auto-release creates a tag using `GITHUB_TOKEN`, release workflow won't trigger

**PAT Solution**:
- Actions performed with PAT trigger workflows normally
- PAT acts as a user, not as GitHub Actions
- Allows full automation: PR merge → tag creation → release

### Creating PAT

1. Go to GitHub → Settings → Developer settings → Personal access tokens → Fine-grained tokens
2. Click "Generate new token"
3. Configure:
   - **Name**: `oops-release-automation`
   - **Expiration**: 90 days (or your preference)
   - **Repository access**: Only select repositories → `animeshkundu/oops`
   - **Permissions**:
     - Contents: Read and write (for pushing tags)
     - Pull requests: Read and write (for creating PRs, enabling auto-merge)
     - Workflows: Read and write (for triggering workflows)
4. Click "Generate token"
5. Copy the token (starts with `github_pat_`)

### Adding PAT to Repository

1. Go to repository → Settings → Secrets and variables → Actions
2. Click "New repository secret"
3. Name: `RELEASE_PAT`
4. Value: Paste your PAT
5. Click "Add secret"

### Verifying PAT Works

With PAT configured:
1. Merge a feature PR
2. Version bump PR is created automatically
3. Version bump PR has auto-merge enabled
4. After CI passes, version bump PR merges automatically
5. Tag is created automatically
6. Release workflow triggers automatically
7. Release is published automatically

**Without PAT**:
- Manual merge required for version bump PR
- Manual workflow trigger may be required for release

---

## Troubleshooting

### Auto-Release Workflow Doesn't Trigger

**Symptoms**: Merged PR but no auto-release workflow run

**Check**:
1. PR merged to `main` or `master`? (Not feature branch)
2. Workflow file exists at `.github/workflows/auto-release.yml`?
3. Check Actions tab for skipped runs

**Solution**:
- Ensure PR is merged to correct branch
- Check PR title doesn't contain `[skip release]`

### Version Bump PR Not Created

**Symptoms**: Auto-release runs but no version bump PR

**Check**:
1. Did tests pass? (Version bump only happens after successful tests)
2. Check workflow logs for errors
3. Look for "Skipping release due to [skip release]" message

**Solution**:
- Fix test failures
- Remove `[skip release]` from PR title if not intended

### Tag Not Created

**Symptoms**: Version bump PR merged but no tag

**Check**:
1. Did version bump PR have "release" label?
2. Did PR title start with "chore: release"?
3. Check create-release-tag workflow logs

**Solution**:
- Ensure version bump PR has correct label and title
- Labels/title are automatically set by auto-release workflow

### Release Workflow Doesn't Trigger

**Symptoms**: Tag exists but release workflow doesn't run

**Check**:
1. How was tag created? (Manual push or via workflow)
2. If via workflow: Is `RELEASE_PAT` configured?
3. Go to Actions → Release workflow and look for runs

**Possible Causes**:
- Tag pushed with `GITHUB_TOKEN` instead of PAT
- Workflow file missing or has syntax errors

**Solutions**:
1. **Configure PAT**: See [PAT Setup](#pat-setup)
2. **Manual trigger**: Run workflow manually (requires workflow_dispatch enhancement)
3. **Delete and recreate tag manually**:
   ```bash
   git tag -d v0.1.2
   git push --delete origin v0.1.2
   git tag -a v0.1.2 -m "Release v0.1.2"
   git push origin v0.1.2
   ```

### Release Build Fails

**Symptoms**: Release workflow runs but fails during build

**Check**:
1. Check workflow logs for specific error
2. Common issues:
   - Version mismatch (Cargo.toml vs tag)
   - Test failures
   - Clippy warnings
   - Build errors

**Solution**:
- Fix the reported error
- Delete failed release (if created)
- Delete tag
- Fix issue and recreate release

### Binaries Not Appearing in Release

**Symptoms**: Release created but no downloadable binaries

**Check**:
1. Did all 6 build jobs succeed?
2. Check "Create Release" job logs
3. Look for artifact upload/download errors

**Solution**:
- Re-run failed jobs
- If persistent, check for platform-specific build issues

---

## Enhancements Needed

### 1. Add workflow_dispatch to release.yml

**Current State**: Manual tags work, but requires git push

**Enhancement**: Add manual workflow trigger

```yaml
on:
  push:
    tags:
      - 'v*'
  workflow_dispatch:
    inputs:
      tag:
        description: 'Tag to release (e.g., v0.1.3)'
        required: true
        type: string
```

**Benefit**: Can trigger release workflow from GitHub UI without git push

### 2. Add Release Testing Script

**Current State**: Manual testing required

**Enhancement**: Create `scripts/test-release.sh`

```bash
#!/bin/bash
# Automated release testing script
# Tests both automated and manual release paths
```

**Benefit**: Automated verification of both release paths

---

## Verification Checklist

Before considering release workflows production-ready:

### Automated Release Path
- [ ] Feature PR creates version bump PR (minor bump)
- [ ] Patch PR creates version bump PR (patch bump)
- [ ] Breaking PR creates version bump PR (major bump)
- [ ] Skip release PR doesn't create version bump PR
- [ ] Version bump PR has correct labels
- [ ] Version bump PR auto-merges (if PAT configured)
- [ ] Tag created after version bump PR merge
- [ ] Release workflow triggers on tag creation
- [ ] All 6 binaries build successfully
- [ ] Checksums generated for all binaries
- [ ] GitHub Release created with all artifacts
- [ ] Release notes auto-generated from PRs

### Manual Release Path
- [ ] Manual tag push triggers release workflow
- [ ] Version mismatch detected and fails appropriately
- [ ] All 6 binaries build successfully
- [ ] Checksums generated for all binaries
- [ ] GitHub Release created with all artifacts
- [ ] Manual workflow dispatch works (if implemented)

### Both Paths
- [ ] Releases appear on GitHub releases page
- [ ] Binaries are downloadable
- [ ] Checksums verify correctly
- [ ] Release notes are clear and helpful
- [ ] Timeline is acceptable (~20-25 min automated, ~10-15 min manual)

---

## References

- Auto-Release Workflow: `.github/workflows/auto-release.yml`
- Create Release Tag Workflow: `.github/workflows/create-release-tag.yml`
- Release Workflow: `.github/workflows/release.yml`
- Release Workflow Fix Documentation: `docs/RELEASE_WORKFLOW_FIX.md`
