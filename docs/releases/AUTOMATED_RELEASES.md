# Automated Release Workflow

This document explains how the automated release system works for the oops project.

## Overview

The oops project uses a **PR-based automated release pipeline** that creates new releases whenever a Pull Request is merged to the `master` branch. This approach respects branch protection rules and maintains version consistency between the master branch and release tags.

### Release Flow

```
PR Merged → Version Bump PR → Auto-merge → Tag Created → Binaries Built → Release Published
```

The system automatically:

1. **Analyzes the merged PR** to determine version bump type
2. **Creates a version bump PR** with updated Cargo.toml and Cargo.lock
3. **Runs CI checks** on the version bump PR
4. **Auto-merges the PR** (if RELEASE_PAT configured) or waits for manual merge
5. **Creates a git tag** pointing to the merged commit
6. **Builds binaries** for 6 targets (Linux, macOS, Windows)
7. **Creates a GitHub release** with all artifacts and checksums

### Pre-Merge Release Testing (New!)

🧪 **The release pipeline now runs as a test on PRs before merging!**

When you create a PR to main/master:
- All CI checks run as normal (tests, linting, etc.)
- **Auto-release job runs as final validation step**
- Creates a **test version bump PR** (marked `[TEST]` and `do-not-merge`)
- Validates version bump logic, Cargo.toml updates, branch creation
- **Does NOT publish any releases** - just tests the pipeline

Benefits:
- ✅ Catch version bump issues before merging
- ✅ Verify release pipeline works with your changes
- ✅ See what version bump will happen (major/minor/patch)
- ✅ No risk - test PRs clearly marked and won't be merged

**Important**: Test PRs are for validation only - **do not merge them**. After you merge your actual PR, the production release process will run automatically.

## Why PR-Based Approach?

The PR-based approach is the **industry standard for 2024** and provides:

✅ **Version Consistency**: Master branch always has the same version as the latest tag
✅ **Branch Protection**: Fully compatible with protected branches (no workarounds needed)
✅ **Audit Trail**: Every version bump visible as a PR with full CI history
✅ **Transparency**: Easy to see what version is on master vs what's released
✅ **Rollback Safety**: Version bumps can be reverted like any other PR
✅ **Standard Practice**: Used by semantic-release, changesets, release-please, and major open source projects
✅ **Pre-Merge Testing**: Validate release pipeline before merging (new feature!)

## Setup Requirements

### Personal Access Token (PAT) - Optional but Recommended

The automated release system can work in two modes:

#### With RELEASE_PAT (Fully Automated)
- Version bump PR is created
- PR auto-merges when CI passes
- Tag is created automatically
- Release workflow triggers automatically
- **Zero manual intervention**

#### Without RELEASE_PAT (Semi-Automated)
- Version bump PR is created
- ⚠️ **Manual merge required**
- Tag must be pushed manually
- **Some manual steps needed**

### Creating the PAT

To enable fully automated releases:

1. Go to GitHub Settings → Developer settings → Personal access tokens → Tokens (classic)
2. Click "Generate new token (classic)"
3. Set a descriptive note: "oops release automation"
4. Set expiration as appropriate (recommend: 90 days or 1 year with calendar reminder)
5. Select scopes:
   - ✅ `repo` (Full control of private repositories)
   - ✅ `workflow` (Update GitHub Action workflows)
6. Click "Generate token"
7. **Copy the token immediately** (you won't see it again!)

#### Adding PAT to Repository

1. Go to your repository Settings → Secrets and variables → Actions
2. Click "New repository secret"
3. Name: `RELEASE_PAT`
4. Value: Paste the PAT you copied
5. Click "Add secret"

See [RELEASE_PAT_SETUP.md](../RELEASE_PAT_SETUP.md) for detailed instructions.

## Workflows

The release system consists of three workflows:

### 1. Auto Release Workflow (`.github/workflows/auto-release.yml`)

**Trigger**: Runs when a PR is merged to `master` branch

**Jobs**:

#### Test Job (Pre-release validation)
- Runs on: Linux, macOS, Windows
- Checks:
  - Code formatting (`cargo fmt --check`)
  - Linting (`cargo clippy -- -D warnings`)
  - Build (`cargo build --release`)
  - All tests (`cargo test`)
- **Must pass** before version bump PR is created

#### Create Version Bump PR Job
**Purpose**: Creates a PR with version changes

**Steps**:
1. Analyze PR title/labels to determine bump type
2. Bump version in `Cargo.toml` using `cargo-edit`
3. Update `Cargo.lock`
4. Validate version bump
5. Create branch `release/vX.Y.Z`
6. Commit version changes
7. Create PR to master
8. Enable auto-merge (if RELEASE_PAT configured)

**PR Details**:
- **Title**: `chore: release vX.Y.Z`
- **Labels**: `release`, `automated`
- **Body**: Version info, source PR, next steps
- **Auto-merge**: Enabled with squash strategy (if PAT available)

### 2. Create Release Tag Workflow (`.github/workflows/create-release-tag.yml`)

**Trigger**: Runs when a PR with label `release` is merged to `master`

**Purpose**: Creates the release tag after version bump is on master

**Steps**:
1. Extract version from merged Cargo.toml
2. Verify tag doesn't already exist
3. Create annotated tag `vX.Y.Z` pointing to merge commit
4. Push tag to origin
5. Comment on version bump PR with release info

**Important**: This workflow ensures the tag points to a commit on master that has the correct version in Cargo.toml.

### 3. Release Workflow (`.github/workflows/release.yml`)

**Trigger**: Runs when a tag matching `v*` is pushed

**Jobs**:

#### Pre-release Tests
- Additional validation before building
- Runs formatting, clippy, and tests
- **Verifies Cargo.toml version matches tag version** ✨

#### Build Matrix
Builds for 6 targets:
- Linux x86_64 (GNU)
- Linux x86_64 (musl - static)
- Linux ARM64 (aarch64)
- macOS x86_64 (Intel)
- macOS ARM64 (Apple Silicon)
- Windows x86_64 (MSVC)

#### Release Creation
- Collects all binary artifacts
- Generates SHA256 checksums for each binary
- Creates GitHub Release with auto-generated notes
- Attaches all binaries and checksums

## Complete Flow Example

Let's walk through a complete release:

1. **Developer merges PR** `fix: handle git errors properly`
   - PR title starts with `fix:` → patch bump

2. **Auto-release workflow runs**
   - Tests pass on all platforms
   - Determines: patch bump (0.1.3 → 0.1.4)
   - Creates branch `release/v0.1.4`
   - Updates Cargo.toml: `version = "0.1.4"`
   - Creates PR #123: `chore: release v0.1.4`
   - Enables auto-merge

3. **CI runs on version bump PR**
   - All checks pass
   - PR auto-merges to master

4. **Create-release-tag workflow runs**
   - Checks out master (now has version 0.1.4)
   - Creates tag `v0.1.4` pointing to master
   - Pushes tag

5. **Release workflow triggered**
   - Checks out tag `v0.1.4`
   - Verifies: Cargo.toml has version 0.1.4 ✅
   - Builds binaries for all 6 targets
   - Creates GitHub Release `v0.1.4`
   - Uploads all binaries + checksums

**Total time**: ~15-20 minutes from PR merge to release published

## Version Bumping Logic

**All merged PRs trigger a MINOR version bump** (e.g., 0.1.0 → 0.2.0).

This simplified approach ensures:
- ✅ Consistent versioning across all changes
- ✅ No need to categorize PR types
- ✅ Clear expectations for contributors
- ✅ Faster release cycles

### Minor Version Bump (0.1.0 → 0.2.0)
**Triggered by:** Any PR merged to `master` (unless explicitly skipped)

Examples:
- `feat: add kubectl rules` → 0.1.0 → 0.2.0
- `fix: handle git errors` → 0.2.0 → 0.3.0
- `docs: update guide` → 0.3.0 → 0.4.0
- `chore: update dependencies` → 0.4.0 → 0.5.0

### Skipping Releases

To merge a PR without triggering a release, include one of these in the PR title:
- `[skip release]`
- `[no release]`

Example: `docs: update README [skip release]`

The auto-release workflow will skip the version bump entirely.

### Manual Version Bumps

If you need a specific version bump (major, patch, or custom version), you can create a manual release:

```bash
# For a major version bump (0.5.0 → 1.0.0)
cargo set-version --bump major

# For a patch version bump (0.5.0 → 0.5.1)  
cargo set-version --bump patch

# For a specific version
cargo set-version 2.0.0

# Then commit, tag, and push
git add Cargo.toml Cargo.lock
git commit -m "chore: bump version to X.Y.Z"
git tag vX.Y.Z
git push origin master --tags
```

## Manual Releases for Testing

For testing builds from feature branches or PRs before merging, use the **Release workflow's manual trigger**:

### Via GitHub Actions UI

1. Go to **Actions** → **Release** workflow
2. Click **Run workflow**
3. Select the branch or enter a ref:
   - **ref**: Branch name, PR ref, or commit SHA (e.g., `my-feature-branch`, `pr-123`, `abc1234`)
4. Click **Run workflow**

The workflow will:
- Build binaries for all 6 targets (Linux, macOS, Windows)
- Generate an auto-tagged pre-release: `manual-v{version}-{ref}-{sha}-{timestamp}`
- Create a GitHub pre-release with all binaries and SHA256 checksums
- Use `target_commitish` to point the release to your specified ref

### Via GitHub CLI

```bash
# Build from a feature branch
gh workflow run release.yml -f ref=my-feature-branch

# Build from a specific commit
gh workflow run release.yml -f ref=abc1234567

# Build from a PR (use the branch name)
gh workflow run release.yml -f ref=fix-memory-leak
```

### Use Cases

- 🧪 Testing binaries from feature branches before merging
- 🔍 Creating preview builds for PR reviewers
- 🚨 Quick hotfix releases from dedicated branches
- 📊 Building specific commits for performance testing

**Note**: Manual releases are always marked as pre-releases and don't interfere with the automated release process or version bumping.

## Manual Override (Advanced)

If needed, you can still create releases manually:

### Manual Version Bump + Release

```bash
# 1. Update version in Cargo.toml
cargo set-version 0.2.0

# 2. Update Cargo.lock
cargo update -p oops

# 3. Commit changes
git add Cargo.toml Cargo.lock
git commit -m "chore: bump version to 0.2.0"

# 4. Create PR or push to master (if you have permission)
git push origin master

# 5. Create and push tag
git tag v0.2.0
git push origin v0.2.0
```

The release workflow will automatically build and publish.

### Manual Tag Push (Re-trigger Release)

If a release failed or you want to rebuild:

```bash
# Delete tag locally and remotely
git tag -d v0.2.0
git push origin :refs/tags/v0.2.0

# Re-create and push tag
git tag v0.2.0
git push origin v0.2.0
```

## PR-Based Release Testing

🧪 **New Feature**: The release pipeline now runs as a test on every PR!

### How It Works

When you create a PR to `main` or `master`:

1. **All normal CI checks run** (tests, linting, coverage, shell tests)
2. **Auto-release job runs as final validation step**
   - Analyzes PR title and labels to determine version bump type
   - Bumps version in Cargo.toml and Cargo.lock
   - Creates a **test version bump PR** (not merged)
3. **Test PR is created** with special markers:
   - Title: `[TEST] chore: release vX.Y.Z`
   - Labels: `test`, `do-not-merge`, `automated`
   - Body explains this is a pre-merge validation

### What Gets Tested

- ✅ Version bump type determination (major/minor/patch)
- ✅ Cargo.toml version update
- ✅ Cargo.lock synchronization
- ✅ Git branch creation
- ✅ PR creation with correct metadata
- ✅ All version bump logic and error handling

### What Doesn't Happen (Test Mode)

- ❌ Test PR is **NOT** merged automatically
- ❌ No release tag is created
- ❌ No binaries are built
- ❌ No GitHub release is published
- ❌ Fork PRs don't run auto-release (security safeguard)

### Benefits

**Early Feedback**: See what version bump will happen before merging your PR.

**Catch Issues Early**: Identify problems with version bumping before they reach main:
- Incorrect bump type logic
- Cargo.toml/Cargo.lock conflicts
- Branch creation failures

**Confidence**: Know the release pipeline will work after merge.

**Transparency**: Version bump decisions are visible in PR checks, not hidden post-merge.

**Zero Risk**: Test PRs are clearly marked and won't be merged.

### Example Workflow

1. You create PR: `feat: add new command`
2. CI runs all checks (tests pass ✅)
3. Auto-release runs and creates test PR: `[TEST] chore: release v0.2.0`
4. You review test PR: "Looks good, minor bump is correct"
5. You merge your actual PR
6. Post-merge: Production auto-release creates real release PR: `chore: release v0.2.0`
7. Real release PR auto-merges → tag created → binaries built → release published

### Managing Test PRs

**What to do with test PRs?**

- **Review them**: Check if version bump type is correct
- **Do NOT merge them**: They're marked `do-not-merge` for a reason
- **Close them**: After reviewing, close the test PR manually
- **Ignore them**: They won't affect your workflow

**Filtering test PRs:**

```bash
# View only test PRs
gh pr list --label test

# View PRs excluding test PRs  
gh pr list --label "!test"

# View PRs excluding do-not-merge
gh pr list --label "!do-not-merge"
```

### Controlling Version Bump Type

The auto-release job determines version bump from:

**PR Title (analyzed in test mode)**:
- `feat!: breaking change` → **major** bump
- `feat: new feature` → **minor** bump  
- `fix: bug fix` → **patch** bump
- Any other → **patch** bump (default)

**PR Labels (analyzed in test mode)**:
- `breaking` or `breaking-change` → **major** bump
- `feature` or `enhancement` → **minor** bump

**Commit Message (analyzed in production mode after merge)**:
- `feat!: breaking change` → **major** bump
- `feat: new feature` → **minor** bump
- `fix: bug fix` → **patch** bump

### Skipping Release Testing

To skip the auto-release test on your PR, add to PR title:

```
feat: add new feature [skip release]
```

Or:

```
docs: update README [no release]
```

The auto-release job will still run but will skip all version bump logic.

### Fork Safety

**Fork PRs are blocked from running auto-release** for security:

- Prevents unauthorized release PRs
- Avoids secret access errors
- Reduces noise from external contributions

Condition: `github.event.pull_request.head.repo.full_name == github.repository`

### Troubleshooting PR-Based Testing

**Test PR not created?**

Check:
1. Is this a fork PR? (Fork PRs don't run auto-release)
2. Does PR title contain `[skip release]`?
3. Did previous CI jobs fail? (Auto-release needs them to pass)
4. Check GitHub Actions tab → CI workflow → auto-release job

**Wrong version bump type?**

- Review your PR title format
- Check if conventional commit format is used correctly
- Add appropriate labels (`feature`, `breaking`, etc.)
- Test locally with `cargo set-version --bump <type>`

**Test PR labeled incorrectly?**

This is a bug - test PRs should have:
- `test` label
- `do-not-merge` label  
- `automated` label
- `[TEST]` prefix in title

Please report if test PRs are missing these markers.

## Troubleshooting

### Version bump PR not created

**Symptoms**: PR merged but no version bump PR appeared

**Possible causes**:
1. PR title contains `[skip release]` or `[no release]`
2. Tests failed in the pre-release test job
3. PR was closed without merging
4. Workflow is disabled

**Solution**:
- Check GitHub Actions tab for workflow run details
- Review test job logs if tests failed
- Ensure PR was actually merged

### Version bump PR not auto-merging

**Symptoms**: Version bump PR created but not merged automatically

**Possible causes**:
1. `RELEASE_PAT` secret not configured
2. CI checks failing on version bump PR
3. Branch protection requires additional approvals

**Solution**:
- Configure `RELEASE_PAT` (see [setup guide](../RELEASE_PAT_SETUP.md))
- Review CI check failures and fix if needed
- Manually merge the PR if auto-merge is not possible

### Tag not created after version bump PR merge

**Symptoms**: Version bump PR merged but no tag created

**Possible causes**:
1. Create-release-tag workflow failed
2. Tag already exists
3. Version extraction failed

**Solution**:
- Check GitHub Actions → Create Release Tag workflow
- Verify tag doesn't already exist: `git tag -l`
- Review workflow logs for errors

### Release workflow not triggered

**Symptoms**: Tag exists but no release created, no binaries built

**Possible causes**:
1. `RELEASE_PAT` not configured (tag push from workflow doesn't trigger release)
2. Release workflow is disabled
3. Tag was created manually without push

**Solution**:
- Configure `RELEASE_PAT` for automatic triggering
- Or manually push the tag: `git push origin v0.2.0`
- Check Settings → Actions → Workflow permissions

### Version mismatch error

**Symptoms**: Release workflow fails with "Version mismatch! Cargo.toml has X but tag is Y"

**Possible causes**:
1. Tag was created before version bump was merged
2. Manual tag creation with wrong version
3. Using old workflow (orphan commit approach)

**Solution**:
- Delete the incorrect tag:
  ```bash
  git tag -d vX.Y.Z
  git push origin :refs/tags/vX.Y.Z
  ```
- Ensure version bump PR is merged first
- Then let create-release-tag workflow create the tag

### Build failed for specific platform

**Symptoms**: Some builds succeed, others fail

**Possible causes**:
- Platform-specific code issues
- Cross-compilation toolchain problems
- Dependency compatibility issues

**Solution**:
- Check the release workflow run details
- Look at the specific platform's build logs
- Test locally with: `cargo build --target <target-triple>`
- Common issues: musl tools not installed, ARM toolchain missing

## Monitoring

After each release:

1. **Check GitHub Actions**:
   - Auto-release workflow: Version bump PR created?
   - Create-release-tag workflow: Tag created?
   - Release workflow: Binaries built?

2. **Verify Release Page**:
   - Go to `https://github.com/<owner>/oops/releases`
   - Confirm new release exists
   - Check all 6 binaries are attached
   - Verify SHA256 checksums present

3. **Test Binary** (spot check):
   ```bash
   # Download a binary
   curl -L -o oops https://github.com/<owner>/oops/releases/download/v0.2.0/oops-linux-x86_64
   chmod +x oops
   ./oops --version  # Should show v0.2.0
   ```

## Benefits of This Approach

### For Developers
- **Transparent**: See exactly what version is on master
- **Predictable**: Know what version will be released
- **Reversible**: Can revert version bumps like any PR
- **Reviewable**: Can review version changes before release

### For Maintainers
- **Automated**: No manual version management
- **Safe**: Multiple quality gates prevent broken releases
- **Traceable**: Full audit trail of version changes
- **Flexible**: Works with or without full automation

### For Users
- **Reliable**: Only tested code is released
- **Consistent**: Every release follows same process
- **Fast**: Binaries available 15-20 minutes after PR merge
- **Verifiable**: SHA256 checksums for integrity

## Best Practices

1. **Clear PR Titles**: Use descriptive titles to document changes
   - `feat: add new kubectl rules`
   - `fix: resolve crash on invalid input`
   - `docs: update installation guide`
   - Note: All PRs trigger minor bumps automatically

2. **Test Before Merge**: Ensure all CI checks pass on your PR

3. **Review Version Bump PRs**: Quickly review auto-generated PRs to catch issues early

4. **Monitor Releases**: Check that releases complete successfully

5. **Keep PAT Updated**: Set calendar reminders for PAT expiration

6. **Document Breaking Changes**: Add clear notes for major version bumps

## Security & Quality Gates

### Before Version Bump PR
1. ✅ All tests must pass on Linux, macOS, and Windows
2. ✅ Formatting must be correct (`cargo fmt`)
3. ✅ No clippy warnings allowed (`cargo clippy -- -D warnings`)
4. ✅ Build must succeed in release mode

### Before Version Bump Merge
1. ✅ CI checks pass on version bump PR
2. ✅ Version format validated (semantic versioning)
3. ✅ Cargo.lock properly updated
4. ✅ No duplicate version tags

### Before Binary Distribution
1. ✅ **Version consistency check**: Cargo.toml version MUST match tag version
2. ✅ Additional test run (pre-release tests)
3. ✅ All unit and integration tests pass
4. ✅ Cross-platform builds succeed

**Result**: Only working, tested, version-consistent builds are released.

## Future Enhancements

Potential improvements to consider:
- [ ] Automatic changelog generation from PR descriptions
- [ ] Publish to crates.io automatically
- [ ] Add smoke tests for released binaries
- [ ] Notification to Discord/Slack on release
- [ ] Update Homebrew formula automatically
- [ ] Generate release notes from commit messages
- [ ] Support for pre-release versions (alpha, beta, rc)
