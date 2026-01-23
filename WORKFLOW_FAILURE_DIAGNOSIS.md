# Workflow Failure Diagnosis and Fix Plan

## Executive Summary

**Problem**: `auto-release.yml` and `create-release-tag.yml` are failing, and auto-release is not integrated into the CI workflow as required.

**Root Causes**:
1. ❌ **Auto-release NOT integrated into CI** - Runs as separate workflow, doesn't wait for CI to pass
2. ❌ **No CI dependency** - Auto-release can trigger even if tests fail
3. ⚠️ **Workflow triggers** - Only on PR merge, not on direct push to main
4. ✅ Event guard pattern is correct (no false failures expected there)

**Solution**: Integrate auto-release as a final job in `ci.yml` that runs only after all tests pass on pushes to main/master.

---

## Detailed Analysis

### Issue 1: Missing CI Integration (PRIMARY ISSUE)

**Requirement from user**: "Check CI integration requirement: adding auto-release as final step of CI build"

**Current State**:
- `auto-release.yml` is a **separate workflow file**
- Triggers: `on: pull_request: types: [closed]`
- Runs **independently** of CI workflow
- **No dependency** on CI jobs passing

**Problem**:
```
PR Merged → auto-release.yml starts
               (CI might still be running!)
               (CI might have failed!)
```

**Expected State**:
```
PR Merged → push to main → ci.yml runs
  ├─ test (all platforms)
  ├─ msrv
  ├─ coverage  
  ├─ shell-tests
  └─ auto-release ← RUNS LAST, only if all above pass
```

### Issue 2: Workflow Structure

**Current CI workflow** (`ci.yml`):
```yaml
jobs:
  test:        # ✅ Runs on push/PR
  msrv:        # ✅ Runs on push/PR
  coverage:    # ✅ Runs on push/PR
  shell-tests: # ✅ Runs on push/PR
  # ❌ NO auto-release job here!
```

**Current auto-release workflow** (`auto-release.yml`):
```yaml
on:
  pull_request:
    types: [closed]  # ❌ Separate trigger, no CI dependency
    branches: [main, master]

jobs:
  event-guard:              # Checks if should run
  test:                     # ❌ DUPLICATE tests!
  create-version-bump-pr:   # Creates version PR
```

**Problems**:
1. **Tests run twice** - Once in CI, once in auto-release
2. **No guarantee CI passed** - Auto-release doesn't check CI status
3. **Slower execution** - Separate workflow has startup delay (~30s)
4. **More complex** - Two workflows to maintain

### Issue 3: Why Workflows Might Be "Failing"

Based on the documentation (`RELEASE_WORKFLOW_FIX.md`), the workflows were previously showing failures when:
- Triggered on wrong event type (push instead of pull_request)
- All jobs skipped due to conditions → "0 jobs = failure"

**Current Fix Applied**: Event guard pattern
- ✅ `event-guard` job always runs (prevents 0 jobs)
- ✅ Other jobs conditionally run based on `event-guard` output
- ✅ Should prevent false failures

**But**: If workflows are still "failing", it might be because:
1. They're not running at all (not integrated into CI)
2. Tests are failing (no CI dependency means bad code can trigger auto-release)
3. Permission issues (though job-level permissions look correct)

### Issue 4: Trigger Timing

**Current**:
```yaml
on:
  pull_request:
    types: [closed]  # ← Only when PR closes
```

**Problem**: Won't run on direct pushes to main (bypassing PRs)

**Better**:
```yaml
on:
  push:
    branches: [main, master]  # ← Runs after every push to main
```

---

## Recommended Solution

### Option A: Integrate into CI (RECOMMENDED) ⭐

**Why**: Simplest, fastest, most reliable

**Changes Required**:

1. **Modify `.github/workflows/ci.yml`**:
   - Add `auto-release` job at the end
   - Add dependency: `needs: [test, msrv, coverage, shell-tests]`
   - Add condition: only run on push to main/master
   - Move auto-release logic from `auto-release.yml`

2. **Keep `.github/workflows/auto-release.yml`** as backup:
   - Add `workflow_dispatch` trigger for manual runs
   - Keep existing logic but make it optional

3. **Keep `.github/workflows/create-release-tag.yml`** as-is:
   - Already working correctly
   - Has proper event guards

**Benefits**:
- ✅ Auto-release only runs after CI passes
- ✅ Single workflow = easier to understand
- ✅ Faster (no workflow startup delay)
- ✅ Matches requirement: "auto-release as final step of CI build"

---

## Concrete YAML Edits Needed

### Edit 1: `.github/workflows/ci.yml`

**Add at end of file** (after `shell-tests` job):

```yaml
  auto-release:
    name: Auto Release
    # Only run on push to main/master (after PR merge)
    # Skip if this is a version bump commit from the bot
    if: |
      github.event_name == 'push' && 
      (github.ref == 'refs/heads/main' || github.ref == 'refs/heads/master') &&
      !contains(github.event.head_commit.message, 'chore: bump version')
    # Wait for all tests to pass first
    needs: [test, msrv, coverage, shell-tests]
    runs-on: ubuntu-latest
    permissions:
      contents: write        # Required for creating branches
      pull-requests: write   # Required for creating PRs
    
    steps:
      - uses: actions/checkout@v4
        with:
          fetch-depth: 0
          token: ${{ secrets.RELEASE_PAT || secrets.GITHUB_TOKEN }}

      - name: Check if release needed
        id: check_release
        env:
          COMMIT_MSG: ${{ github.event.head_commit.message }}
        run: |
          # Skip if commit message contains [skip release] or [no release]
          if echo "$COMMIT_MSG" | grep -qiE "\[(skip|no).?release\]"; then
            echo "skip=true" >> $GITHUB_OUTPUT
            echo "Skipping release due to [skip release] in commit message"
          else
            echo "skip=false" >> $GITHUB_OUTPUT
          fi

      - name: Install Rust
        if: steps.check_release.outputs.skip == 'false'
        uses: dtolnay/rust-toolchain@stable

      - name: Install cargo-edit
        if: steps.check_release.outputs.skip == 'false'
        uses: taiki-e/install-action@v2
        with:
          tool: cargo-edit@0.12.2

      - name: Determine version bump type
        if: steps.check_release.outputs.skip == 'false'
        id: bump_type
        env:
          COMMIT_MSG: ${{ github.event.head_commit.message }}
        run: |
          set -e
          
          echo "🔍 Analyzing commit for version bump type"
          echo "Commit: $COMMIT_MSG"
          
          BUMP_TYPE="patch"
          REASON="default"
          COMMIT_LOWER=$(echo "$COMMIT_MSG" | tr '[:upper:]' '[:lower:]')
          
          # Check for breaking changes
          if echo "$COMMIT_LOWER" | grep -qE '(^|[^a-z])(feat|fix|chore)!\s*(\(|:)'; then
            BUMP_TYPE="major"
            REASON="breaking change (! marker)"
          elif echo "$COMMIT_LOWER" | grep -qE '\bbreaking\s+change\b'; then
            BUMP_TYPE="major"
            REASON="BREAKING CHANGE in message"
          # Check for features
          elif echo "$COMMIT_LOWER" | grep -qE '(^|[^a-z])feat\s*(\(|:)'; then
            BUMP_TYPE="minor"
            REASON="feature (feat:)"
          fi
          
          echo "Bump Type: $BUMP_TYPE"
          echo "Reason: $REASON"
          echo "bump_type=$BUMP_TYPE" >> $GITHUB_OUTPUT
          echo "bump_reason=$REASON" >> $GITHUB_OUTPUT

      - name: Bump version in Cargo.toml
        if: steps.check_release.outputs.skip == 'false'
        id: version
        run: |
          set -e
          
          OLD_VERSION=$(cargo metadata --no-deps --format-version 1 | jq -r '.packages[] | select(.name == "oops") | .version')
          echo "Current version: $OLD_VERSION"
          
          BUMP_TYPE="${{ steps.bump_type.outputs.bump_type }}"
          cargo set-version --bump "$BUMP_TYPE"
          
          NEW_VERSION=$(cargo metadata --no-deps --format-version 1 | jq -r '.packages[] | select(.name == "oops") | .version')
          
          if [ "$OLD_VERSION" = "$NEW_VERSION" ]; then
            echo "::error::Version did not change"
            exit 1
          fi
          
          echo "Version bumped: $OLD_VERSION → $NEW_VERSION"
          echo "old_version=$OLD_VERSION" >> $GITHUB_OUTPUT
          echo "new_version=$NEW_VERSION" >> $GITHUB_OUTPUT

      - name: Update Cargo.lock
        if: steps.check_release.outputs.skip == 'false'
        run: cargo update -p oops

      - name: Check for existing version bump PR
        if: steps.check_release.outputs.skip == 'false'
        id: check_pr
        env:
          GH_TOKEN: ${{ secrets.GITHUB_TOKEN }}
        run: |
          NEW_VERSION="${{ steps.version.outputs.new_version }}"
          BRANCH_NAME="release/v$NEW_VERSION"
          
          if git ls-remote --heads origin "$BRANCH_NAME" | grep -q "$BRANCH_NAME"; then
            echo "exists=true" >> $GITHUB_OUTPUT
            echo "Branch $BRANCH_NAME already exists"
          else
            echo "exists=false" >> $GITHUB_OUTPUT
          fi

      - name: Create version bump branch and PR
        if: steps.check_release.outputs.skip == 'false' && steps.check_pr.outputs.exists == 'false'
        env:
          GH_TOKEN: ${{ secrets.GITHUB_TOKEN }}
        run: |
          NEW_VERSION="${{ steps.version.outputs.new_version }}"
          OLD_VERSION="${{ steps.version.outputs.old_version }}"
          BRANCH_NAME="release/v$NEW_VERSION"
          
          git config --local user.email "github-actions[bot]@users.noreply.github.com"
          git config --local user.name "github-actions[bot]"
          
          git checkout -b "$BRANCH_NAME"
          git add Cargo.toml Cargo.lock
          
          git commit -m "chore: bump version to $NEW_VERSION" \
                     -m "Version bump: $OLD_VERSION → $NEW_VERSION" \
                     -m "Type: ${{ steps.bump_type.outputs.bump_type }}" \
                     -m "Reason: ${{ steps.bump_type.outputs.bump_reason }}"
          
          git push origin "$BRANCH_NAME"
          
          gh pr create \
            --base main \
            --head "$BRANCH_NAME" \
            --title "chore: release v$NEW_VERSION" \
            --body "Automated version bump from $OLD_VERSION to $NEW_VERSION" \
            --label "release" \
            --label "automated"
          
          echo "✅ Created version bump PR for v$NEW_VERSION"

      - name: Enable auto-merge
        if: steps.check_release.outputs.skip == 'false' && steps.check_pr.outputs.exists == 'false' && secrets.RELEASE_PAT != ''
        env:
          GH_TOKEN: ${{ secrets.RELEASE_PAT }}
        run: |
          NEW_VERSION="${{ steps.version.outputs.new_version }}"
          BRANCH_NAME="release/v$NEW_VERSION"
          
          sleep 2
          PR_NUMBER=$(gh pr list --head "$BRANCH_NAME" --json number --jq '.[0].number')
          
          if [ -n "$PR_NUMBER" ]; then
            gh pr merge "$PR_NUMBER" --auto --squash
            echo "✅ Enabled auto-merge for PR #$PR_NUMBER"
          fi
```

**Key Points**:
- Only runs on push to main/master (not on PRs)
- Waits for all CI jobs to pass (`needs:` clause)
- Skips version bump commits to avoid loops
- Uses simplified logic from auto-release.yml
- Requires `RELEASE_PAT` for auto-merge

### Edit 2: `.github/workflows/auto-release.yml`

**Option A: Keep as manual fallback**

Add at the top (after line 2):

```yaml
on:
  workflow_dispatch:  # Allow manual triggering
    inputs:
      skip_ci_check:
        description: 'Skip CI status check (use with caution)'
        required: false
        type: boolean
        default: false
  # Keep existing pull_request trigger for backwards compatibility
  pull_request:
    types: [closed]
    branches: [main, master]
```

**Option B: Delete file entirely**

Since logic is now in CI, this file is redundant. However, keeping it as manual fallback is safer during transition.

### Edit 3: `.github/workflows/create-release-tag.yml`

**No changes needed** - This workflow is working correctly.

---

## Testing Plan

### 1. Test CI Integration
```bash
# After changes, push to main
git push origin main

# Expected behavior:
# ✅ CI runs: test, msrv, coverage, shell-tests
# ✅ Auto-release runs last (only if all pass)
# ✅ Version bump PR created
```

### 2. Test Skip Logic
```bash
# Create commit with [skip release]
git commit -m "docs: update README [skip release]"
git push origin main

# Expected behavior:
# ✅ CI runs normally
# ✅ Auto-release job skips
# ❌ No version bump PR created
```

### 3. Test Version Bumping
```bash
# Feature commit
git commit -m "feat: add new feature"
git push origin main

# Expected behavior:
# ✅ CI passes
# ✅ Auto-release creates PR with minor bump
# ✅ Version bump PR auto-merges (if RELEASE_PAT configured)
# ✅ Create-release-tag workflow creates tag
# ✅ Release workflow builds binaries
```

---

## Migration Strategy

### Phase 1: Add auto-release to CI (Non-Breaking)
1. Add `auto-release` job to `ci.yml`
2. Keep `auto-release.yml` as-is
3. Both workflows will run (redundant but safe)
4. Monitor for 1-2 releases

### Phase 2: Consolidate (After Validation)
1. Modify `auto-release.yml` to only run on `workflow_dispatch`
2. Remove PR trigger from `auto-release.yml`
3. Update documentation

### Phase 3: Cleanup (Optional)
1. Delete `auto-release.yml` if manual trigger not needed
2. Simplify documentation

---

## Risk Assessment

| Change | Risk | Mitigation |
|--------|------|------------|
| Add job to CI | Low | Job is conditional, won't affect existing jobs |
| Modify auto-release.yml | Low | Keep file, only add manual trigger |
| Version bump logic | Medium | Test thoroughly before relying on it |
| Auto-merge requirement | Low | Falls back gracefully if RELEASE_PAT not set |

---

## Success Criteria

✅ Auto-release runs as final step of CI
✅ Auto-release only runs after all tests pass  
✅ Auto-release only runs on push to main/master
✅ Version bump PRs created automatically
✅ No duplicate test runs
✅ No false failures in workflow status
✅ Backwards compatible with existing setup

---

## References

- Current workflows: `.github/workflows/ci.yml`, `auto-release.yml`, `create-release-tag.yml`
- Documentation: `docs/releases/AUTOMATED_RELEASES.md`
- Previous fix: `docs/RELEASE_WORKFLOW_FIX.md`
- GitHub Actions: [Using conditions](https://docs.github.com/en/actions/using-workflows/workflow-syntax-for-github-actions#jobsjob_idif)
