# CI Workflow Fix Summary

## Problem Statement

The CI workflow (`.github/workflows/ci.yml`) had been modified to run the `auto-release` job on both `push` and `pull_request` events. This caused GitHub to stop showing the "Approve workflows" button for pull requests.

### Root Cause

When a workflow explicitly filters for same-repository PRs only using:
```yaml
github.event.pull_request.head.repo.full_name == github.repository
```

GitHub treats these PRs as "trusted" and doesn't show the approval button, even for first-time contributors. This is a security concern as it allows untrusted code to run without manual approval.

## Solution

Restored the workflow from git and made minimal, targeted changes to:
1. Remove the `workflow_dispatch` trigger
2. Make `auto-release` job run ONLY on push events (not PRs)
3. Simplify all related steps to remove PR-specific logic

## Changes Made

### 1. Removed `workflow_dispatch` Trigger
**File**: `.github/workflows/ci.yml`, line 8
- Removed: `workflow_dispatch:`
- Effect: Workflow cannot be manually triggered anymore

### 2. Simplified Auto-Release Job Condition
**File**: `.github/workflows/ci.yml`, lines 145-167 → 148-152

**Before** (23 lines):
```yaml
if: |
  (
    (github.event_name == 'push' &&
     (github.ref == 'refs/heads/main' || github.ref == 'refs/heads/master')) ||
    (github.event_name == 'pull_request' &&
     github.event.pull_request.head.repo.full_name == github.repository) ||
    (github.event_name == 'workflow_dispatch' &&
     github.repository == github.repository)
  ) &&
  (github.event_name != 'push' || !contains(github.event.head_commit.message, 'chore: bump version')) &&
  (github.event_name != 'push' || !contains(github.event.head_commit.message, 'chore: release')) &&
  (github.event_name != 'pull_request' || !contains(github.event.pull_request.title, 'chore: bump version')) &&
  (github.event_name != 'pull_request' || !contains(github.event.pull_request.title, 'chore: release'))
```

**After** (5 lines):
```yaml
if: |
  github.event_name == 'push' &&
  (github.ref == 'refs/heads/main' || github.ref == 'refs/heads/master') &&
  !contains(github.event.head_commit.message, 'chore: bump version') &&
  !contains(github.event.head_commit.message, 'chore: release')
```

**Key Change**: Removed the problematic `github.event.pull_request.head.repo.full_name == github.repository` check.

### 3. Simplified Workflow Steps

#### Step: "Check if release needed" (lines 180-204 → 167-178)
- **Before**: Checked both commit message and PR title based on event type
- **After**: Only checks commit message
- **Removed**: `PR_TITLE` and `EVENT_NAME` environment variables

#### Step: "Determine version bump type" (lines 215-268 → 192-221)
- **Before**: Analyzed commit messages, PR titles, and PR labels
- **After**: Only analyzes commit messages using conventional commit format
- **Removed**: PR label checks (`breaking`, `feature`, `enhancement`)

#### Step: "Create version bump branch and PR" (lines 322-456 → 276-347)
- **Before**: Dual-mode logic (test mode for PRs, production mode for push)
- **After**: Single production mode only
- **Removed**: `[TEST]` PR prefix, `do-not-merge` label, test mode PR body

#### Step: "Create workflow summary" (lines 400-492 → 349-399)
- **Before**: Different summaries for test mode vs production mode
- **After**: Single production summary
- **Removed**: Test mode documentation

#### Step: "Enable auto-merge" (line 358)
- **Before**: `&& github.event_name == 'push'`
- **After**: Removed redundant check (job already ensures push event)

## Statistics

- **Lines changed**: 79 insertions, 223 deletions
- **Net reduction**: 144 lines (24% smaller)
- **File size**: 595 lines → 451 lines

## Expected Behavior

### For Pull Requests (All Cases)
1. PR is opened (external or same-repo)
2. **Approval button appears for external contributors** ✅
3. Tests run: `test`, `msrv`, `coverage`, `shell-tests`
4. `auto-release` job is **SKIPPED** (condition not met)

### After PR Merge to Main
1. Push event triggers workflow
2. All tests run and pass
3. `auto-release` job runs:
   - Determines version bump type from commit message
   - Bumps version in `Cargo.toml` and `Cargo.lock`
   - Creates `release/v{version}` branch
   - Creates PR with version bump
   - Enables auto-merge (if `RELEASE_PAT` secret is configured)
4. Version bump PR auto-merges (if enabled)
5. Subsequent push triggers tag creation (separate workflow)
6. Release workflow builds binaries for the tag

### Version Bump Type Detection

The workflow determines bump type from commit messages using conventional commit format:

| Commit Format | Bump Type | Example |
|---------------|-----------|---------|
| `feat!:` or `fix!:` | major | `feat!: breaking API change` |
| Contains "BREAKING CHANGE" | major | Any commit with this text |
| `feat:` | minor | `feat: add new feature` |
| Everything else | patch | `fix: bug fix`, `chore: update deps` |

## Security Considerations

✅ **No PR-based triggers for auto-release** - only push to main/master
✅ **Version bump commits skipped** - prevents infinite loops
✅ **No secrets exposed in PR context** - auto-release only runs post-merge
✅ **CodeQL security scan passed** - no vulnerabilities detected
✅ **Approval button restored** - external contributors require approval

## Testing Recommendations

1. **Test external PR approval**:
   - Create a PR from a fork
   - Verify "Approve workflows" button appears
   - Approve and verify tests run

2. **Test auto-release**:
   - Merge a PR with `feat:` prefix to main
   - Verify auto-release creates minor version bump PR
   - Check PR has correct labels (`release`, `automated`)
   - Verify auto-merge is enabled (if PAT configured)

3. **Test version bump detection**:
   - Merge PR with `feat:` → verify minor bump
   - Merge PR with `fix:` → verify patch bump
   - Merge PR with `feat!:` → verify major bump

4. **Test skip logic**:
   - Merge PR with `[skip release]` in commit message
   - Verify auto-release is skipped

## Files Changed

- `.github/workflows/ci.yml` (79 insertions, 223 deletions)

## Validation

✅ Code review passed (no issues found)
✅ CodeQL security scan passed (0 alerts)
✅ YAML syntax validated
✅ Git diff verified
