# Release PAT Setup Guide

## Quick Setup (For Repository Maintainers)

This guide helps you set up the Personal Access Token (PAT) for **fully automated releases** with the PR-based workflow.

### Why is this needed?

The automated release system uses a **PR-based workflow** to respect branch protection:

1. When PR merges → Version bump PR is created
2. With PAT → PR auto-merges when CI passes
3. Without PAT → Manual merge required

**Two Reasons for PAT**:
1. **Enable PR auto-merge**: The workflow needs permission to enable auto-merge on PRs
2. **Trigger release workflow**: Tag pushes from workflows need PAT to trigger other workflows (GitHub security feature)

### Without RELEASE_PAT

The workflow still works but requires manual steps:
- ✅ Version bump PR is created automatically
- ❌ PR does NOT auto-merge (requires manual merge)
- ⚠️ Tag may not trigger release (may need manual push)

### With RELEASE_PAT

Fully automated end-to-end:
- ✅ Version bump PR created automatically
- ✅ PR auto-merges when CI passes
- ✅ Tag created automatically
- ✅ Release workflow triggered automatically
- **Zero manual intervention**

### Step-by-Step Setup

#### 1. Create a Personal Access Token

1. Go to [GitHub Settings → Personal Access Tokens](https://github.com/settings/tokens)
2. Click **"Generate new token (classic)"**
3. Fill in the form:
   - **Note**: `oops-release-automation`
   - **Expiration**: Choose 90 days, 1 year, or no expiration (set calendar reminder if using expiration)
   - **Scopes**: 
     - ✅ `repo` (Full control of repositories) - Needed for:
       - Creating branches
       - Creating PRs
       - Enabling auto-merge
       - Creating tags
     - ✅ `workflow` (Update GitHub Actions workflows) - Needed for:
       - Triggering the release workflow when tag is pushed
4. Click **"Generate token"**
5. **Copy the token immediately** (you can't see it again!)

#### 2. Add PAT to Repository Secrets

1. Go to [Repository Settings → Secrets and Variables → Actions](https://github.com/animeshkundu/oops/settings/secrets/actions)
2. Click **"New repository secret"**
3. Configure:
   - **Name**: `RELEASE_PAT` (must be exactly this name)
   - **Secret**: Paste the token you copied
4. Click **"Add secret"**

#### 3. Verify Setup

The next time a PR is merged to `master`:
1. Auto-release workflow creates version bump PR
2. Version bump PR has auto-merge enabled
3. When CI passes, PR merges automatically
4. Tag is created automatically
5. Release workflow builds binaries

### Testing the Setup

#### Option 1: Quick Test with New PR

1. Create a simple PR (e.g., doc update)
2. Use conventional commit title: `fix: typo in README`
3. Merge the PR
4. Watch the automation:
   - Check Actions tab for "Auto Release" workflow
   - Look for new PR titled "chore: release vX.Y.Z"
   - Verify PR has auto-merge enabled
   - Wait for CI to pass and PR to auto-merge
   - Check Actions tab for "Create Release Tag" workflow
   - Check Actions tab for "Release" workflow
   - Verify release is published with binaries

#### Option 2: Test with Manual Tag Push

To verify the PAT can trigger workflows:

```bash
# From your local repository
git tag v0.1.3-test
git push origin v0.1.3-test
```

Check the Actions tab - you should see the release workflow triggered.

**Note**: This tests only the tag triggering, not the full PR-based flow.

### Security Considerations

- **Token Access**: Only repository admins need to create/manage this token
- **Token Scope**: The `repo` and `workflow` scopes are necessary and appropriate for this use case
- **Token Rotation**: Set an expiration and calendar reminder to rotate the token
- **Token Storage**: GitHub encrypts secrets at rest and in transit
- **Least Privilege**: The PAT is only used for:
  - Creating version bump PRs
  - Enabling auto-merge
  - Pushing tags
  - Triggering release workflow

### Troubleshooting

#### "Secret not found" error
- Verify the secret name is exactly `RELEASE_PAT` (case-sensitive)
- Ensure you're in the correct repository settings
- Check that the secret was saved successfully

#### Version bump PR not auto-merging
- Verify PAT has both `repo` and `workflow` scopes
- Check that the PAT hasn't expired
- Ensure branch protection allows merges (no required approvals from specific teams)
- Check workflow logs for auto-merge step errors

#### Release still not triggering
- Verify the PAT has `workflow` scope enabled
- Ensure the tag push succeeded (check create-release-tag workflow logs)
- Check that release workflow isn't disabled in repository settings

#### Token expired
- Create a new token following steps above
- Update the `RELEASE_PAT` secret with the new token
- No other changes needed

### Fallback Mode

If `RELEASE_PAT` is not configured:

**What still works:**
- ✅ Auto-release workflow runs
- ✅ Version bump PR is created
- ✅ All validation checks pass

**What requires manual action:**
- ❌ PR does not auto-merge - **Maintainer must merge manually**
- ⚠️ Tag may not trigger release - **May need manual push:** `git push origin vX.Y.Z`

**Workflow:**
1. Merge feature PR
2. Check Actions → See version bump PR created
3. **Manual step**: Review and merge version bump PR
4. **Manual step** (if release doesn't trigger): Push tag manually

## Detailed Explanation

### Why GitHub TOKEN Can't Trigger Workflows

GitHub Actions has a security feature to prevent infinite workflow loops:

> "When you use the repository's GITHUB_TOKEN to perform tasks, events triggered by the GITHUB_TOKEN, with the exception of `workflow_dispatch` and `repository_dispatch`, will not create a new workflow run."

**Example without PAT:**
1. Workflow A pushes a tag using GITHUB_TOKEN
2. Tag push event occurs
3. Workflow B configured to run on tag push
4. ❌ Workflow B does NOT trigger (security feature)

**Example with PAT:**
1. Workflow A pushes a tag using PAT
2. Tag push is attributed to PAT owner (a real user)
3. Tag push event occurs normally
4. ✅ Workflow B triggers as expected

### Why Auto-Merge Needs PAT

The `gh pr merge --auto` command requires enhanced permissions that GITHUB_TOKEN doesn't provide by default. The PAT ensures the workflow can:
- Enable auto-merge on PRs it creates
- Allow PRs to merge via automated means
- Maintain audit trail showing automation

### PR-Based vs Direct Push

The new workflow uses a PR-based approach instead of direct push:

**Old approach (direct push with PAT)**:
- Workflow pushes directly to protected branch using PAT
- Bypasses branch protection
- No PR for visibility
- Hard to review changes

**New approach (PR-based with auto-merge)**:
- Workflow creates PR with version changes
- PR goes through normal CI checks
- Branch protection fully respected
- Auto-merge when CI passes
- Full visibility and audit trail

## Additional Resources

- [GitHub Actions: Automatic token authentication](https://docs.github.com/en/actions/security-guides/automatic-token-authentication)
- [GitHub: Creating a personal access token](https://docs.github.com/en/authentication/keeping-your-account-and-data-secure/creating-a-personal-access-token)
- [Triggering a workflow from another workflow](https://docs.github.com/en/actions/using-workflows/triggering-a-workflow#triggering-a-workflow-from-a-workflow)
- [Auto-merging pull requests](https://docs.github.com/en/pull-requests/collaborating-with-pull-requests/incorporating-changes-from-a-pull-request/automatically-merging-a-pull-request)
- [Automated releases with PR-based workflows (2024 Best Practices)](https://dev.to/arpanaditya/automating-releases-with-semantic-versioning-and-github-actions-2a06)

## Summary

| Aspect | Without PAT | With PAT |
|--------|-------------|----------|
| Version bump PR created | ✅ Yes | ✅ Yes |
| PR auto-merge | ❌ No | ✅ Yes |
| Tag created | ⚠️ Manual | ✅ Automatic |
| Release triggered | ⚠️ Manual push | ✅ Automatic |
| Manual steps | 2-3 | 0 |
| Recommended for | Testing, small projects | Production, active projects |

**Recommendation**: Configure RELEASE_PAT for the best experience. The workflow gracefully degrades without it, but full automation provides the smoothest developer experience.
