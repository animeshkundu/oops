# Release PAT Setup Guide

## Quick Setup (For Repository Maintainers)

This guide helps you set up the Personal Access Token (PAT) required for automated releases.

### Why is this needed?

The automated release system needs to trigger the release workflow when a tag is pushed. GitHub Actions has a security feature that prevents the default `GITHUB_TOKEN` from triggering new workflows (to avoid infinite loops). Therefore, we need a PAT.

### Step-by-Step Setup

#### 1. Create a Personal Access Token

1. Go to [GitHub Settings → Personal Access Tokens](https://github.com/settings/tokens)
2. Click **"Generate new token (classic)"**
3. Fill in the form:
   - **Note**: `oops-release-automation`
   - **Expiration**: Choose 90 days, 1 year, or no expiration (set calendar reminder if using expiration)
   - **Scopes**: 
     - ✅ `repo` (Full control of repositories)
     - ✅ `workflow` (Update GitHub Actions workflows)
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
1. The auto-release workflow will create a tag
2. The tag push will automatically trigger the release workflow
3. Binaries will be built and a GitHub Release will be created

### Testing the Setup

To test without merging a PR:

```bash
# From your local repository
git tag v0.1.3-test
git push origin v0.1.3-test
```

Check the Actions tab - you should see the release workflow triggered.

### Security Considerations

- **Token Access**: Only repository admins need to create/manage this token
- **Token Scope**: The `repo` and `workflow` scopes are necessary and appropriate
- **Token Rotation**: Set an expiration and calendar reminder to rotate the token
- **Token Storage**: GitHub encrypts secrets at rest and in transit

### Troubleshooting

#### "Secret not found" error
- Verify the secret name is exactly `RELEASE_PAT` (case-sensitive)
- Ensure you're in the correct repository settings

#### Release still not triggering
- Check that the workflow file has been updated to use `secrets.RELEASE_PAT`
- Verify the PAT has both `repo` and `workflow` scopes
- Ensure the PAT hasn't expired

#### Token expired
- Create a new token following steps above
- Update the `RELEASE_PAT` secret with the new token

### Fallback Mode

If `RELEASE_PAT` is not configured, the workflow uses `GITHUB_TOKEN` as a fallback. In this mode:
- ✅ Version bumps work
- ✅ Tags are created
- ❌ Release workflow is NOT automatically triggered
- 💡 You can manually push tags to trigger releases

## Additional Resources

- [GitHub Actions: Automatic token authentication](https://docs.github.com/en/actions/security-guides/automatic-token-authentication)
- [GitHub: Creating a personal access token](https://docs.github.com/en/authentication/keeping-your-account-and-data-secure/creating-a-personal-access-token)
- [Triggering a workflow from another workflow](https://docs.github.com/en/actions/using-workflows/triggering-a-workflow#triggering-a-workflow-from-a-workflow)
