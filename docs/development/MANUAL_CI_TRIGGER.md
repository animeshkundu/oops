# Manual CI Workflow Trigger

## Overview

The CI workflow (`.github/workflows/ci.yml`) can be manually triggered from the GitHub Actions UI to test the release pipeline on PR branches before merging.

## Use Case

**Pre-merge release testing**: Validate that version bumps and release automation work correctly before merging a PR to main.

## How to Use

### 1. Navigate to Actions Tab
- Go to your repository on GitHub
- Click the **Actions** tab
- Select **CI** workflow from the left sidebar

### 2. Run Workflow Manually
- Click the **Run workflow** dropdown (top right)
- Select the branch you want to test (typically your PR branch)
- Click **Run workflow** button

### 3. Review Results
The auto-release job will:
- Run after all tests pass (test, msrv, coverage, shell-tests)
- Create a test version bump PR with `[TEST]` prefix
- Label it with `test`, `do-not-merge`, `automated`
- Display results in the workflow summary

### 4. Cleanup
- Review the test version bump PR created
- **Do NOT merge** the test PR (it's labeled `do-not-merge`)
- Close the test PR manually after review

## What Gets Tested

✅ **Validates**:
- Version bump logic (major/minor/patch detection)
- `Cargo.toml` and `Cargo.lock` updates
- Branch creation and PR creation
- Labels and metadata
- Commit message formatting

❌ **Does NOT**:
- Create actual releases (test mode only)
- Enable auto-merge (test mode only)
- Trigger tag creation
- Build release binaries

## Security

**Fork Protection**: Manual triggers from forked repositories are blocked. The auto-release job only runs for same-repository branches.

## Comparison with PR-Based Testing

| Trigger Method | When to Use | Event Type |
|----------------|-------------|------------|
| **Pull Request** | Automatic testing when PR is created | `pull_request` |
| **Manual (workflow_dispatch)** | On-demand testing on any branch | `workflow_dispatch` |
| **Push to main** | Production releases after merge | `push` |

Both PR-based and manual triggers create test version bump PRs and behave identically.

## Example Workflow

```bash
# Developer has a branch: feature/new-command
git checkout feature/new-command
git push origin feature/new-command

# On GitHub:
# 1. Go to Actions → CI → Run workflow
# 2. Select branch: feature/new-command
# 3. Click "Run workflow"

# Wait for workflow to complete
# Check workflow summary for test results
# Review test PR if created
# Close test PR
# Continue working or create actual PR
```

## Benefits

- **Pre-PR testing**: Test release pipeline before creating a PR
- **Ad-hoc validation**: Test specific branches without creating PRs
- **Debugging**: Reproduce issues in isolation
- **Development**: Validate release logic changes

## Limitations

1. **Manual cleanup required**: Test PRs must be closed manually
2. **Branch must exist**: Can only trigger on pushed branches
3. **Same repository only**: Fork branches are blocked
4. **No auto-merge**: Auto-merge is disabled in test mode

## Related Documentation

- [PR-Based Release Testing](../handoffs/2026-01-23-pr-based-release-testing.md) - Automatic PR testing
- [Testing Auto-Release](../TESTING_AUTO_RELEASE.md) - Local testing with scripts
- [Automated Releases](../releases/AUTOMATED_RELEASES.md) - Production release flow

---

**Quick Reference**: Manual CI trigger = on-demand release pipeline testing without creating actual releases.
