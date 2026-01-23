# PR Approval Workflow Fix

## Problem

After recent changes to the CI workflow, the **"Approve workflows to run"** button disappeared from the PR UI. This meant that external contributors' workflows would run automatically without manual approval, which is a security concern.

## Root Cause

The issue was introduced in commits `cfc8517` and `b936cba` when the `auto-release` job was modified to run on pull request events:

```yaml
if: |
  (github.event_name == 'pull_request' &&
   github.event.pull_request.head.repo.full_name == github.repository)
```

### Why This Caused the Issue

GitHub requires manual approval for workflows triggered by:
- First-time contributors
- External contributors from forked repositories

**However**, when a workflow explicitly checks `head.repo.full_name == github.repository` (same-repo PRs only), GitHub interprets this as:
- "This workflow only runs for trusted same-repo PRs"
- Therefore, no approval is needed

This security feature bypasses manual approval because the workflow author has explicitly limited execution to the same repository.

## The Problem with This Approach

While the intention was to test the release pipeline on PRs before merging, it had unintended consequences:

1. **Security Risk**: Even same-repo PRs from new contributors should require approval
2. **Lost Functionality**: The approval button completely disappeared
3. **Confusing Behavior**: Users couldn't see or control when workflows ran

## Solution

Revert the `auto-release` job to run **only on push events** (post-merge):

### Changes Made

1. **Removed `workflow_dispatch` trigger** from the top of the workflow
2. **Simplified auto-release condition** to:
   ```yaml
   if: |
     github.event_name == 'push' &&
     (github.ref == 'refs/heads/main' || github.ref == 'refs/heads/master') &&
     !contains(github.event.head_commit.message, 'chore: bump version') &&
     !contains(github.event.head_commit.message, 'chore: release')
   ```

3. **Removed PR-specific logic** from:
   - Version bump type detection (no PR labels)
   - Release skip check (no PR titles)
   - PR creation logic (no test mode)
   - Workflow summary (no test/production split)

4. **Removed event-specific conditionals**:
   - No more `EVENT_NAME` checks
   - No more `PR_TITLE` or `PR_LABELS` environment variables
   - Simpler, cleaner code

### File Changes

- `.github/workflows/ci.yml`: 79 insertions, 223 deletions (-144 lines, -24%)

## Result

### ✅ PRs Now:
- Show "Approve workflows" button for external/first-time contributors
- Run only the test jobs (test, msrv, coverage, shell-tests)
- Do NOT trigger auto-release
- Require manual approval as intended

### ✅ Post-Merge (Push to Main):
- Auto-release job runs automatically
- Creates version bump PR with `release` label
- Enables auto-merge (if RELEASE_PAT configured)
- Triggers full release pipeline:
  1. Version bump PR created
  2. CI tests run on PR
  3. PR auto-merges (or manual merge)
  4. Tag created by `create-release-tag.yml`
  5. Release workflow builds 3 binaries
  6. GitHub Release published

## Release Pipeline Flow

```
┌─────────────────────┐
│  PR Merged to Main  │
└──────────┬──────────┘
           │
           ▼
┌─────────────────────┐
│  CI Workflow Runs   │
│  - test            │
│  - msrv            │
│  - coverage        │
│  - shell-tests     │
│  - auto-release ◄──┼─── ONLY runs on push
└──────────┬──────────┘
           │
           ▼
┌─────────────────────┐
│ Version Bump PR     │
│ Created             │
│ Label: release      │
└──────────┬──────────┘
           │
           ▼
┌─────────────────────┐
│  CI Tests Run       │
│  (on version PR)    │
└──────────┬──────────┘
           │
           ▼
┌─────────────────────┐
│  Auto-Merge or      │
│  Manual Merge       │
└──────────┬──────────┘
           │
           ▼
┌─────────────────────┐
│  Tag Created        │
│  (create-release-   │
│   tag.yml)          │
└──────────┬──────────┘
           │
           ▼
┌─────────────────────┐
│  Release Workflow   │
│  Builds 3 Binaries  │
│  - Linux x86_64     │
│  - macOS ARM64      │
│  - Windows x86_64   │
└──────────┬──────────┘
           │
           ▼
┌─────────────────────┐
│  GitHub Release     │
│  Published          │
└─────────────────────┘
```

## Testing the Release Pipeline

With the auto-release job no longer running on PRs, how can you test it?

### Option 1: Test Branch Merge (Recommended)

```bash
# Create a test branch
git checkout -b test/release-pipeline

# Make a change and commit with feat: to trigger minor bump
git commit --allow-empty -m "feat: test release pipeline"

# Push to test branch
git push origin test/release-pipeline

# Merge to main via GitHub UI or CLI
gh pr create --fill
gh pr merge --squash

# Watch the CI workflow in GitHub Actions
```

### Option 2: Manual Trigger of create-release-tag Workflow

```bash
# Create a manual version bump
# Edit Cargo.toml manually, bump version
git add Cargo.toml Cargo.lock
git commit -m "chore: release v0.2.0"
git push origin release/v0.2.0

# Create PR with 'release' label
gh pr create --label release --title "chore: release v0.2.0"

# Merge PR
# Then manually trigger create-release-tag workflow if needed
```

## Alternative: Dedicated Test Workflow

If you want to test the release pipeline on PRs without affecting approval buttons, create a separate workflow:

```yaml
# .github/workflows/test-release-pipeline.yml
name: Test Release Pipeline

on:
  workflow_dispatch:  # Manual trigger only

permissions:
  contents: write
  pull-requests: write

jobs:
  test-release:
    runs-on: ubuntu-latest
    steps:
      # ... same steps as auto-release but with test labels
```

This approach:
- ✅ Doesn't affect PR approval buttons
- ✅ Can be triggered manually on any branch
- ✅ Clearly marked as "test"
- ✅ Doesn't interfere with production releases

## Security Considerations

### Why Push-Only is More Secure

1. **Code Review**: All code has been reviewed and merged before auto-release runs
2. **No PR Context**: Auto-release never has access to PR context (titles, labels, descriptions)
3. **Protected Branches**: Main/master branches typically have protection rules
4. **Audit Trail**: All releases trace back to merged PRs with approvals

### GitHub's Workflow Approval System

GitHub shows the approval button when:
- PR is from a fork
- PR is from a first-time contributor
- Workflow requires elevated permissions (write access)

GitHub **hides** the approval button when:
- Workflow explicitly filters for same-repo PRs only
- This signals to GitHub that the workflow author trusts same-repo PRs

**Best Practice**: Don't run privileged jobs (write permissions) on PR events. Save them for post-merge.

## Commit Reference

- **Fixed in**: `cd804d1` - "fix: revert auto-release to push-only to restore PR approval button"
- **Originally broken in**:
  - `cfc8517` - "Run auto-release as final CI step on same-repo PRs"
  - `b936cba` - "feat: add workflow_dispatch trigger to CI for manual testing"

## Related Documentation

- [Automated Releases](../releases/AUTOMATED_RELEASES.md)
- [Release Workflow](../releases/RELEASE_WORKFLOW_QA.md)
- [Contributing Guide](../../CONTRIBUTING.md)
- [GitHub Actions Best Practices](https://docs.github.com/en/actions/security-guides/security-hardening-for-github-actions)

## Conclusion

This fix restores proper workflow approval functionality while maintaining full release automation. The trade-off is that we can't test the release pipeline on PRs anymore, but this is the correct security posture for a public repository.

**Key Takeaway**: Workflows with write permissions should only run on protected branches (push events), not on pull requests.
