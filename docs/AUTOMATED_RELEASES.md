# Automated Release Workflow

This document explains how the automated release system works for the oops project.

## Overview

The oops project uses a fully automated release pipeline that creates new releases whenever a Pull Request is merged to the `master` branch. The system automatically:

1. Runs comprehensive tests on all platforms
2. Determines the appropriate version bump
3. Updates version files
4. Creates a git tag
5. Builds binaries for 6 targets (3 platforms: Linux, macOS, Windows)
6. Creates a GitHub release with all artifacts

## Workflows

### 1. Auto Release Workflow (`.github/workflows/auto-release.yml`)

**Trigger**: Runs when a PR is merged to `master` branch

**Jobs**:

#### Test Job
- Runs on: Linux, macOS, Windows
- Checks:
  - Code formatting (`cargo fmt --check`)
  - Linting (`cargo clippy -- -D warnings`)
  - Build (`cargo build --release`)
  - All tests (`cargo test`)
- **Must pass** before release proceeds

#### Auto-Release Job
- Determines version bump type from PR title/labels
- Bumps version in `Cargo.toml` using `cargo-edit`
- Updates `Cargo.lock`
- Commits changes
- Creates and pushes git tag (e.g., `v0.1.1`)

### 2. Release Workflow (`.github/workflows/release.yml`)

**Trigger**: Runs when a tag matching `v*` is pushed (triggered by auto-release)

**Jobs**:

#### Pre-release Tests
- Additional validation before building
- Runs formatting, clippy, and tests

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

### 3. CI Workflow (`.github/workflows/ci.yml`)

**Trigger**: Runs on all pushes and PRs to `master`

**Note**: Skips execution for version bump commits to avoid infinite loops

## Version Bumping Logic

The version bump type is determined by the PR title and labels:

### Major Version Bump (0.1.0 → 1.0.0)
Triggered by:
- PR title starts with `feat!:` or `fix!:`
- PR title contains `breaking` or `BREAKING`
- PR has label `breaking`

Examples:
- `feat!: redesign API interface`
- `BREAKING: remove deprecated functions`

### Minor Version Bump (0.1.0 → 0.2.0)
Triggered by:
- PR title starts with `feat:`
- PR has label `feature`
- PR has label `enhancement`

Examples:
- `feat: add kubectl rules`
- `feat(rules): support docker-compose errors`

### Patch Version Bump (0.1.0 → 0.1.1)
Default for all other cases:
- PR title starts with `fix:`
- PR title starts with `docs:`, `chore:`, etc.
- No specific prefix

Examples:
- `fix: handle git detached HEAD state`
- `docs: update installation guide`
- `chore: update dependencies`

## Skipping Releases

To merge a PR without triggering a release, include one of these in the PR title:
- `[skip release]`
- `[no release]`

Example: `docs: update README [skip release]`

## Security & Quality Gates

### Before Version Bump
1. **All tests must pass** on Linux, macOS, and Windows
2. **Formatting** must be correct (`cargo fmt`)
3. **No clippy warnings** allowed (`cargo clippy -- -D warnings`)
4. **Build must succeed** in release mode

### Before Binary Distribution
1. Additional test run (pre-release tests)
2. Formatting and clippy checks (again)
3. All unit and integration tests pass

### Result
**Only working, tested builds are released**. If any check fails, the release is aborted.

## Manual Override

If needed, you can still create releases manually:

```bash
# Update version in Cargo.toml
cargo set-version 0.2.0

# Update Cargo.lock
cargo update -p oops

# Commit changes
git add Cargo.toml Cargo.lock
git commit -m "chore: bump version to 0.2.0"
git push

# Create and push tag
git tag v0.2.0
git push --tags
```

The release workflow will automatically build and publish.

## Troubleshooting

### Release didn't trigger after PR merge
- Check if PR title contains `[skip release]` or `[no release]`
- Verify PR was merged (not closed without merging)
- Check GitHub Actions tab for workflow runs
- Look for any test failures in the test job

### Version bump was wrong
- Check PR title format - it determines bump type
- Consider adding labels (`breaking`, `feature`, `enhancement`)
- For next release, adjust PR title before merging

### Build failed for specific platform
- Check the release workflow run details
- Look at the specific platform's build logs
- Common issues: cross-compilation toolchain, dependencies

### Release created but no binaries attached
- Check if all build jobs completed successfully
- Verify the release job successfully downloaded artifacts
- Check that files were copied to the release directory

## Monitoring

After each release:
1. Check GitHub Actions runs for both workflows
2. Verify the new tag was created
3. Verify the GitHub release exists with all binaries
4. Download and test at least one binary

## Benefits of This Approach

1. **Fully Automated**: No manual steps required
2. **Consistent**: Every release follows the same process
3. **Tested**: Only working builds are released
4. **Fast**: Parallel builds for all platforms
5. **Traceable**: All actions logged in GitHub Actions
6. **Safe**: Multiple quality gates prevent broken releases
7. **Semantic**: Versions follow SemVer based on change type

## Future Enhancements

Potential improvements to consider:
- [ ] Add changelog generation from PR descriptions
- [ ] Publish to crates.io automatically
- [ ] Add smoke tests for released binaries
- [ ] Notification to Discord/Slack on release
- [ ] Update Homebrew formula automatically
