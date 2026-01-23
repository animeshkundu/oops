# GitHub Actions Workflows - Quick Reference

## Available Workflows

### 1. Continuous Integration (CI)
**File**: `.github/workflows/ci.yml`
**Trigger**: Push to main/master, Pull Requests
**Purpose**: Test, lint, and validate code on every change

**Jobs**:
- `test`: Matrix build across OSes (Ubuntu, macOS, Windows) and Rust versions (stable, beta)
- `msrv`: Minimum Supported Rust Version check (1.88)
- `coverage`: Code coverage with cargo-llvm-cov
- `shell-tests`: Integration tests for shell alias generation
- `auto-release`: Automated version bumping and PR creation (runs on merge to main)

### 2. Release
**File**: `.github/workflows/release.yml`
**Trigger**: Git tags matching `v*` or manual via workflow_dispatch
**Purpose**: Build and publish cross-platform binaries

**Binaries**: 6 platforms
- Linux: x86_64 (GNU), x86_64 (musl), ARM64
- macOS: x86_64 (Intel), ARM64 (Apple Silicon)
- Windows: x86_64

**Jobs**:
- `test`: Pre-release validation
- `build`: Cross-platform binary builds
- `release`: Create GitHub Release with artifacts

### 3. Manual Build and Release ⭐ NEW
**File**: `.github/workflows/manual-build.yml`
**Trigger**: Manual via workflow_dispatch (GitHub UI or CLI)
**Purpose**: Build and release binaries from any branch/PR for testing

**Binaries**: 3 platforms (fast builds)
- Linux: x86_64 (GNU)
- macOS: ARM64 (Apple Silicon)
- Windows: x86_64

**Use Cases**:
- Test builds from feature branches
- PR preview builds for reviewers
- Emergency hotfix releases
- Beta/RC testing

**Key Features**:
- ✅ Does NOT modify version or create PRs
- ✅ Does NOT affect PR approval workflows
- ✅ Unique tag naming (`manual-*`)
- ✅ Pre-release by default
- ✅ Fast builds (8-12 minutes)

**Usage**:
```bash
# Via GitHub CLI
gh workflow run manual-build.yml -f ref=my-branch

# Via GitHub UI
Actions → Manual Build and Release → Run workflow
```

📚 **Documentation**: [docs/MANUAL_BUILD_WORKFLOW.md](../docs/MANUAL_BUILD_WORKFLOW.md)

### 4. Auto Release (Legacy)
**File**: `.github/workflows/auto-release.yml`
**Trigger**: Manual via workflow_dispatch (backup)
**Purpose**: Legacy workflow for manual version bump PR creation
**Note**: Superseded by auto-release job in ci.yml

### 5. Create Release Tag
**File**: `.github/workflows/create-release-tag.yml`
**Trigger**: PR merge with 'release' label
**Purpose**: Create git tag after version bump PR merge

### 6. Security Audit
**File**: `.github/workflows/audit.yml`
**Trigger**: Weekly schedule + changes to Cargo.toml/lock
**Purpose**: Scan dependencies for vulnerabilities

## Quick Start

### Manual Build from Current Branch
```bash
gh workflow run manual-build.yml -f ref=$(git branch --show-current)
```

### Manual Build from PR
```bash
gh workflow run manual-build.yml -f ref=refs/pull/123/head -f tag_suffix=-test
```

### Manual Build with Custom Suffix
```bash
gh workflow run manual-build.yml -f ref=feature/new-rules -f tag_suffix=-rc1
```

### View Workflow Runs
```bash
gh run list --workflow=manual-build.yml
gh run watch <run-id>
```

## Workflow Comparison

| Feature | CI | Release | Manual Build |
|---------|----|---------|--------------| 
| **Trigger** | Auto (push/PR) | Tag push | Manual |
| **Platforms** | 3 (test) | 6 (all) | 3 (main) |
| **Duration** | 5-10 min | 15-20 min | 8-12 min |
| **Version Bump** | Auto (main) | Required | None |
| **Use Case** | Testing | Production | Preview/Test |

## Common Tasks

### Run CI Locally
```bash
cargo fmt --check
cargo clippy -- -D warnings
cargo test
```

### Trigger Official Release
1. Merge PR to main (auto-creates version bump PR)
2. Merge version bump PR (auto-creates tag)
3. Tag push triggers release workflow

### Create Test Build
1. Push branch to GitHub
2. Run: `gh workflow run manual-build.yml -f ref=your-branch`
3. Download binaries from Releases page

### Debug Failed Workflow
```bash
# View logs
gh run view <run-id> --log

# Re-run failed jobs
gh run rerun <run-id> --failed
```

## Tag Naming Patterns

```
Official releases:  v0.1.0, v0.2.0-beta1, v1.0.0-rc2
Manual builds:      manual-v0.1.0-branch-abc1234
                    manual-v0.2.0-pr-45-def5678-rc1
```

## Documentation

- [Manual Build Workflow](../docs/MANUAL_BUILD_WORKFLOW.md) - Complete guide
- [Automated Releases](../docs/releases/AUTOMATED_RELEASES.md) - Release process
- [CI/CD Expert System Prompt](copilot-instructions.md) - CI/CD agent knowledge

## Support

- 🐛 Issues: [GitHub Issues](https://github.com/animeshkundu/oops/issues)
- 💬 Discussions: [GitHub Discussions](https://github.com/animeshkundu/oops/discussions)
- 📖 Docs: [docs/](../docs/)
