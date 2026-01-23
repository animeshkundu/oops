# Manual Build and Release Workflow

## Overview

The **Manual Build and Release** workflow allows you to build and publish binaries from any branch, PR, or commit without affecting the automated release process or PR approval workflows.

**Location**: `.github/workflows/manual-build.yml`

## Use Cases

- 🧪 **Testing feature branches**: Build binaries from a feature branch before merging
- 🔍 **PR preview builds**: Create binaries from a PR for reviewers to test
- 🚨 **Emergency hotfixes**: Quick releases from hotfix branches
- 📊 **Performance testing**: Build specific commits for benchmarking
- 🎯 **Beta testing**: Create pre-release builds for testers

## Key Features

### ✅ What It Does

- Builds binaries for 3 main platforms (Linux, macOS, Windows)
- Creates a GitHub Release with generated tag name
- Uses `target_commitish` to point release to specified commit
- Generates SHA256 checksums for all binaries
- Runs format checks, clippy, and tests before building
- Provides detailed release notes with commit information

### ✅ What It Doesn't Do (By Design)

- **Does NOT** modify `Cargo.toml` version
- **Does NOT** create or modify PRs
- **Does NOT** interfere with automated release workflows
- **Does NOT** affect PR approval requirements
- **Does NOT** modify any branch or create version bump commits

## How to Use

### Via GitHub UI

1. Go to **Actions** → **Manual Build and Release**
2. Click **Run workflow**
3. Fill in the inputs:
   - **ref**: Branch name, tag, or commit SHA (e.g., `my-feature-branch`, `pr-123`, `abc1234`)
   - **prerelease**: Check to mark as pre-release (default: `true`)
   - **draft**: Check to create as draft release (default: `false`)
   - **tag_suffix**: Optional suffix (e.g., `-rc1`, `-beta`). Leave empty for auto-generated timestamp
4. Click **Run workflow**

### Via GitHub CLI

```bash
# Build from current branch
gh workflow run manual-build.yml \
  -f ref=$(git branch --show-current) \
  -f prerelease=true

# Build from a specific PR
gh workflow run manual-build.yml \
  -f ref=refs/pull/123/head \
  -f prerelease=true \
  -f tag_suffix=-test

# Build from a commit SHA
gh workflow run manual-build.yml \
  -f ref=abc1234 \
  -f prerelease=true \
  -f draft=false

# Build with custom tag suffix
gh workflow run manual-build.yml \
  -f ref=feature/new-rules \
  -f prerelease=true \
  -f tag_suffix=-rc1
```

### Input Parameters

| Parameter | Required | Default | Description |
|-----------|----------|---------|-------------|
| `ref` | ✅ Yes | `main` | Git ref (branch, tag, or commit SHA) to build from |
| `prerelease` | ❌ No | `true` | Mark release as pre-release (recommended for manual builds) |
| `draft` | ❌ No | `false` | Create as draft release (useful for review before publishing) |
| `tag_suffix` | ❌ No | `""` | Optional tag suffix. Empty = auto-generated timestamp |

## Tag Naming Convention

The workflow generates unique tag names automatically:

### Format

```
manual-v{version}-{branch}-{sha}[-{suffix}]
```

### Examples

| Input Ref | Tag Suffix | Generated Tag |
|-----------|------------|---------------|
| `main` | *(empty)* | `manual-v0.1.1-main-abc1234-20260115-143022` |
| `feature/new-rules` | `-rc1` | `manual-v0.1.1-feature-new-rules-abc1234-rc1` |
| `refs/pull/45/head` | `-test` | `manual-v0.1.1-pr-45-head-abc1234-test` |
| `hotfix/critical-bug` | *(empty)* | `manual-v0.1.1-hotfix-critical-bug-def5678-20260115-143530` |

**Notes:**
- `{version}` is extracted from `Cargo.toml`
- `{branch}` is sanitized (special characters replaced with `-`)
- `{sha}` is the short commit SHA (8 characters)
- Timestamp format: `YYYYMMDD-HHMMSS` (UTC)

## Built Binaries

The workflow builds **3 binaries** (one per major platform):

| Platform | Binary | Target | Notes |
|----------|--------|--------|-------|
| 🐧 **Linux** | `oops-linux-x86_64` | `x86_64-unknown-linux-gnu` | Most compatible Linux binary (GNU libc) |
| 🍎 **macOS** | `oops-darwin-aarch64` | `aarch64-apple-darwin` | Apple Silicon (M1/M2/M3) |
| 🪟 **Windows** | `oops-windows-x86_64.exe` | `x86_64-pc-windows-msvc` | Standard Windows x64 |

**Why only 3?**
- Manual builds prioritize **speed** over comprehensive coverage
- These 3 binaries cover ~90% of users
- Full 6-binary builds (including musl, ARM64 Linux, Intel macOS) are reserved for official releases
- Faster iteration for testing and development

Each binary includes:
- ✅ SHA256 checksum file (`.sha256`)
- ✅ Verified checksums before release
- ✅ Optimized release build

## Workflow Jobs

### 1. `prepare` - Metadata Generation
- Extracts version from `Cargo.toml`
- Generates unique tag name
- Validates ref exists
- Checks tag doesn't already exist
- **Outputs**: Tag name, release name, commit SHA, version

### 2. `test` - Pre-build Validation
- Checks code formatting (`cargo fmt --check`)
- Runs Clippy lints (`cargo clippy -- -D warnings`)
- Runs test suite (`cargo test`)
- **Purpose**: Ensure build quality before spending time on multi-platform builds

### 3. `build` - Binary Compilation
- Matrix build across 3 platforms (Linux, macOS, Windows)
- Builds release optimized binaries
- Generates SHA256 checksums
- Uploads artifacts
- **Duration**: ~5-10 minutes

### 4. `release` - GitHub Release Creation
- Downloads all artifacts
- Verifies checksums
- Generates detailed release notes
- Creates GitHub Release
- Uses `target_commitish` to point to specified ref
- **Result**: Published release with binaries

## Release Notes

Automatically generated release notes include:

- ✅ Build metadata (version, ref, commit, timestamp)
- ✅ Commit details (author, date, message)
- ✅ Binary verification instructions
- ✅ Platform compatibility table
- ✅ Warning about non-official release status

**Example**: See [example release notes](#example-release-notes) below.

## Verification

### Linux/macOS

```bash
# Download binary and checksum
wget https://github.com/USER/oops/releases/download/TAG/oops-linux-x86_64
wget https://github.com/USER/oops/releases/download/TAG/oops-linux-x86_64.sha256

# Verify checksum
sha256sum -c oops-linux-x86_64.sha256

# Make executable
chmod +x oops-linux-x86_64

# Test
./oops-linux-x86_64 --version
```

### Windows (PowerShell)

```powershell
# Download binary and checksum
Invoke-WebRequest -Uri "https://github.com/USER/oops/releases/download/TAG/oops-windows-x86_64.exe" -OutFile "oops.exe"
Invoke-WebRequest -Uri "https://github.com/USER/oops/releases/download/TAG/oops-windows-x86_64.exe.sha256" -OutFile "oops.exe.sha256"

# Verify checksum
$hash = (Get-FileHash oops.exe).Hash.ToLower()
$expected = (Get-Content oops.exe.sha256).Split()[0]
if ($hash -eq $expected) {
    Write-Host "✅ Checksum verified" -ForegroundColor Green
} else {
    Write-Host "❌ Checksum mismatch!" -ForegroundColor Red
}

# Test
.\oops.exe --version
```

## Differences from Official Release Workflow

| Feature | Official Release (`release.yml`) | Manual Build (`manual-build.yml`) |
|---------|----------------------------------|-----------------------------------|
| **Trigger** | Git tag push (`v*`) | Manual (workflow_dispatch) |
| **Binary Count** | 6 platforms | 3 platforms |
| **Version Bump** | Automated via PR | None (uses current `Cargo.toml`) |
| **Tag Creation** | Via automated flow | Generated by workflow |
| **Pre-release** | Auto-detected (alpha/beta/rc) | User-specified input |
| **Use Case** | Production releases | Testing/preview builds |
| **PR Creation** | Yes (version bump) | No |
| **Build Time** | ~10-15 minutes | ~5-10 minutes |
| **Platforms** | Linux (x64, musl, ARM64), macOS (x64, ARM64), Windows | Linux (x64), macOS (ARM64), Windows |

## Best Practices

### ✅ Do

- **Use prerelease flag** for all manual builds (default: `true`)
- **Add descriptive tag suffix** for beta/RC builds (e.g., `-beta1`, `-rc2`)
- **Test binaries** after building before sharing with others
- **Document the build** in PR comments or testing notes
- **Clean up old manual releases** periodically to avoid clutter

### ❌ Don't

- **Don't use for production releases** - use the standard release process instead
- **Don't skip the prerelease flag** for untested builds
- **Don't reuse tag names** - each build should have unique tag
- **Don't bypass tests** - let the workflow run all quality checks
- **Don't modify workflow** to skip checksum verification

## Troubleshooting

### Error: "Tag already exists"

**Cause**: The generated tag name is already in use.

**Solution**:
1. Use a different `tag_suffix`, or
2. Wait a moment (timestamp will change), or
3. Delete the existing tag if it's a test build

```bash
# Delete local and remote tag
git tag -d TAG_NAME
git push origin :refs/tags/TAG_NAME
```

### Error: "Ref does not exist"

**Cause**: The specified ref is invalid or doesn't exist.

**Solution**:
1. Check branch/tag name spelling
2. For PRs, use format: `refs/pull/NUMBER/head`
3. For commits, use full or short SHA that exists in the repo

```bash
# Verify ref exists
git ls-remote origin | grep REF_NAME
```

### Build Fails on Test Job

**Cause**: Code doesn't pass formatting, clippy, or tests.

**Solution**:
1. Fix issues in the branch
2. Run locally before triggering workflow:
   ```bash
   cargo fmt --check
   cargo clippy -- -D warnings
   cargo test
   ```

### Wrong Version in Release

**Cause**: `Cargo.toml` version doesn't match expectations.

**Solution**:
- This workflow uses the version **as-is** from `Cargo.toml` in the specified ref
- Update `Cargo.toml` in your branch if needed
- For official releases, use the standard release process

## Integration with Existing Workflows

### No Conflicts

This workflow is designed to **not interfere** with existing workflows:

| Workflow | How Manual Build Interacts |
|----------|---------------------------|
| `ci.yml` | ✅ Independent - doesn't trigger CI |
| `release.yml` | ✅ Independent - uses different tag pattern |
| `auto-release.yml` | ✅ Independent - doesn't create PRs |
| `create-release-tag.yml` | ✅ Independent - doesn't touch release branches |
| `audit.yml` | ✅ Independent - runs on schedule |

### PR Approval Process

**Important**: This workflow **does NOT remove or bypass PR approval requirements**.

- ❌ Does not merge PRs
- ❌ Does not create or modify PRs
- ❌ Does not affect branch protection rules
- ❌ Does not bypass required reviews
- ✅ Only creates a separate release with binaries
- ✅ PR must still go through normal review process

### Tag Naming Strategy

Manual build tags use `manual-` prefix to avoid conflicts:

```
Official releases:  v0.1.0, v0.2.0-beta1, v1.0.0-rc2
Manual builds:      manual-v0.1.0-branch-abc1234, manual-v0.2.0-pr-45-def5678-rc1
```

This ensures:
- ✅ No collision with official release tags
- ✅ Easy to identify manual vs official releases
- ✅ Automatic cleanup scripts can target `manual-*` tags

## Example Release Notes

<details>
<summary>Click to expand example</summary>

```markdown
## 🔧 Manual Build Release

This is a manually triggered build for testing or preview purposes.

### Build Information
- **Version**: v0.1.1 *(from Cargo.toml)*
- **Source Ref**: `feature/new-git-rules`
- **Commit**: `abc12345`
- **Built**: 20260115-143022 UTC
- **Tag**: `manual-v0.1.1-feature-new-git-rules-abc12345-rc1`

### Commit Details
**Author**: Jane Developer  
**Date**: 2026-01-15 14:15:30 +0000  
**Message**:
```
feat: add Git command correction rules

- Add git push/pull/commit corrections
- Improve branch name suggestions
- Add tests for Git rules
```

### 📦 Included Binaries

| Platform | Binary | Architecture |
|----------|--------|--------------|
| 🐧 Linux | `oops-linux-x86_64` | x86_64 (GNU libc) |
| 🍎 macOS | `oops-darwin-aarch64` | ARM64 (Apple Silicon) |
| 🪟 Windows | `oops-windows-x86_64.exe` | x86_64 |

All binaries include SHA256 checksum files (*.sha256) for verification.

### 🔒 Verification

To verify binary integrity:
```bash
# Linux/macOS
sha256sum -c oops-linux-x86_64.sha256

# Windows (PowerShell)
$hash = (Get-FileHash oops-windows-x86_64.exe).Hash.ToLower()
$expected = (Get-Content oops-windows-x86_64.exe.sha256).Split()[0]
if ($hash -eq $expected) { Write-Host "✅ Checksum verified" } else { Write-Host "❌ Checksum mismatch" }
```

### ⚠️ Note

This is **not** an official release. For production use, please use the official releases from the [Releases page](https://github.com/USER/oops/releases).

---

🤖 *Built by [manual-build workflow](.github/workflows/manual-build.yml)*
```

</details>

## GitHub CLI Examples

```bash
# Quick build from current branch
gh workflow run manual-build.yml -f ref=$(git branch --show-current)

# Build from PR for reviewer testing
gh workflow run manual-build.yml \
  -f ref=refs/pull/123/head \
  -f prerelease=true \
  -f tag_suffix=-review

# Build release candidate
gh workflow run manual-build.yml \
  -f ref=release/v1.0.0 \
  -f prerelease=true \
  -f tag_suffix=-rc2

# Build draft release for review
gh workflow run manual-build.yml \
  -f ref=main \
  -f prerelease=false \
  -f draft=true

# Build from specific commit for bisecting
gh workflow run manual-build.yml \
  -f ref=abc1234567890 \
  -f tag_suffix=-bisect
```

## Monitoring

### View Workflow Runs

```bash
# List recent manual build runs
gh run list --workflow=manual-build.yml

# Watch a specific run
gh run watch RUN_ID

# View logs
gh run view RUN_ID --log
```

### GitHub UI

1. Go to **Actions** tab
2. Select **Manual Build and Release** workflow
3. View run history, logs, and artifacts

## Security Considerations

### Permissions

The workflow requires:
- `contents: write` - For creating releases and tags

### Best Practices

1. **Review before building** - Ensure the ref contains trusted code
2. **Verify checksums** - Always verify SHA256 before running binaries
3. **Use draft mode** - For sensitive testing, use `draft: true`
4. **Clean up** - Remove old manual build releases regularly
5. **Document usage** - Note in PR when manual builds are created

## Future Enhancements

Potential improvements (not currently implemented):

- [ ] Configurable binary set (select which platforms to build)
- [ ] Slack/Discord notifications on build completion
- [ ] Automatic cleanup of old manual builds (retention policy)
- [ ] Comment on PR with download links (if triggered from PR)
- [ ] Integration with artifact signing service
- [ ] Build metrics and performance tracking

## Related Documentation

- [Release Process](../../docs/releases/AUTOMATED_RELEASES.md) - Official release workflow
- [Contributing Guide](../../CONTRIBUTING.md) - How to contribute
- [CI/CD Overview](../../docs/ci-cd/) - Complete CI/CD documentation

## Support

For issues or questions:
1. Check [Troubleshooting](#troubleshooting) section above
2. Review [GitHub Actions logs](https://github.com/USER/oops/actions/workflows/manual-build.yml)
3. Open an issue with workflow run link and error details
