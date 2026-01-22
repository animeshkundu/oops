# Auto-Release Workflow Documentation

## Overview

The oops project uses a two-workflow approach for automated releases:

1. **auto-release.yml** - Handles version bumping and tag creation on PR merge
2. **release.yml** - Builds cross-platform binaries and creates GitHub releases when tags are pushed

This follows GitHub Actions best practices for semantic versioning and release automation.

## Workflow Architecture

```
┌─────────────────┐
│   PR Merged     │
│   to main       │
└────────┬────────┘
         │
         ▼
┌─────────────────────────┐
│  auto-release.yml       │
│  1. Run tests (3 OS)    │
│  2. Determine version   │
│     bump (major/minor/  │
│     patch)              │
│  3. Update Cargo.toml   │
│  4. Commit & push       │
│  5. Create & push tag   │
└────────┬────────────────┘
         │
         ▼
┌─────────────────────────┐
│  release.yml            │
│  (Triggered by tag)     │
│  1. Run tests           │
│  2. Build for 6 targets:│
│     - Linux x86_64      │
│     - Linux x86_64-musl │
│     - Linux ARM64       │
│     - macOS x86_64      │
│     - macOS ARM64       │
│     - Windows x86_64    │
│  3. Generate checksums  │
│  4. Create GitHub       │
│     release with all    │
│     binaries            │
└─────────────────────────┘
```

## auto-release.yml Workflow

### Trigger
- **Event**: Pull request closed
- **Branches**: main, master
- **Condition**: PR must be merged (not just closed)

### Jobs

#### 1. Test Job
Runs comprehensive tests on all three main platforms before proceeding with release:
- **Platforms**: Ubuntu, macOS, Windows
- **Steps**:
  - Checkout code
  - Install Rust toolchain with clippy and rustfmt
  - Cache cargo dependencies
  - Check code formatting
  - Run clippy linter
  - Build in release mode
  - Run all tests

#### 2. Auto-Release Job
Creates a new version and triggers the release workflow:

**Version Bump Logic**:
- **Major bump** (X.0.0): PR title starts with `feat!:` or `fix!:`, contains "breaking", or has "breaking" label
- **Minor bump** (0.X.0): PR title starts with `feat:` or has "feature"/"enhancement" label
- **Patch bump** (0.0.X): Default for bug fixes and other changes

**Steps**:
1. Check if release should be skipped (if PR title contains `[skip release]` or `[no release]`)
2. Install Rust and cargo-edit
3. Determine version bump type based on PR title and labels
4. Bump version in Cargo.toml
5. Update Cargo.lock
6. Commit version changes
7. Push to main branch
8. Create and push version tag (e.g., `v0.2.0`)

### Skipping Releases

Add `[skip release]` or `[no release]` to PR title to prevent automatic release:
```
fix: minor docs update [skip release]
```

## release.yml Workflow

### Trigger
- **Event**: Push to tags matching `v*` pattern (e.g., v0.1.0, v1.2.3)

### Jobs

#### 1. Test Job
Quick validation build on Ubuntu to catch any issues early.

#### 2. Build Job
Builds binaries for all supported platforms:

| Platform | Architecture | Target Triple | Artifact Name |
|----------|--------------|---------------|---------------|
| Linux | x86_64 | x86_64-unknown-linux-gnu | oops-linux-x86_64 |
| Linux | x86_64 (musl) | x86_64-unknown-linux-musl | oops-linux-x86_64-musl |
| Linux | ARM64 | aarch64-unknown-linux-gnu | oops-linux-aarch64 |
| macOS | x86_64 | x86_64-apple-darwin | oops-darwin-x86_64 |
| macOS | ARM64 (M1/M2) | aarch64-apple-darwin | oops-darwin-aarch64 |
| Windows | x86_64 | x86_64-pc-windows-msvc | oops-windows-x86_64.exe |

Each build:
- Installs necessary cross-compilation tools
- Builds in release mode with LTO optimization
- Generates SHA256 checksum for verification
- Uploads as GitHub Actions artifact

#### 3. Release Job
Creates the GitHub release:
- Downloads all build artifacts
- Prepares release assets
- Creates GitHub release with:
  - All 6 platform binaries
  - SHA256 checksums
  - Auto-generated release notes
  - Pre-release flag for alpha/beta/rc versions

## Conventional Commits

The workflow uses Conventional Commits to determine version bumps:

### Major Version (Breaking Changes)
```
feat!: redesign CLI interface
fix!: remove deprecated --legacy flag
chore: refactor core [breaking]
```

### Minor Version (New Features)
```
feat: add docker command corrections
feat(rules): support kubectl errors
```

### Patch Version (Bug Fixes, Docs, etc.)
```
fix: handle empty command output
docs: update installation guide
chore: update dependencies
test: add missing test cases
```

## Usage Examples

### Standard Feature Release
1. Create PR with title: `feat: add new correction rule for npm`
2. Merge PR to main
3. auto-release.yml runs:
   - Tests pass on all platforms
   - Version bumps from 0.1.0 → 0.2.0
   - Commits version bump
   - Creates tag v0.2.0
4. release.yml triggers automatically:
   - Builds 6 platform binaries
   - Creates GitHub release v0.2.0 with all binaries

### Bug Fix Release
1. PR title: `fix: correct git branch detection`
2. Merge → version 0.2.0 → 0.2.1
3. Tag v0.2.1 created
4. Release created with binaries

### Breaking Change Release
1. PR title: `feat!: redesign rule matching engine`
2. Merge → version 0.2.1 → 1.0.0
3. Tag v1.0.0 created
4. Release created with binaries

### Skip Release
1. PR title: `docs: fix typo in README [skip release]`
2. Merge → no version bump, no release

## Benefits of This Approach

1. **Separation of Concerns**: Version management separate from binary building
2. **Cross-Platform Support**: Automatic builds for 6 different targets
3. **Semantic Versioning**: Automatic version bumping based on commit conventions
4. **Safety**: Tests run before any version changes or releases
5. **Flexibility**: Easy to skip releases when needed
6. **Transparency**: Clear commit history with version bump commits
7. **Automation**: Zero manual steps required for releases
8. **Best Practices**: Follows GitHub Actions and Rust community standards

## Troubleshooting

### Version Not Bumping
- Check if PR title contains `[skip release]`
- Verify PR was merged (not just closed)
- Check workflow logs in GitHub Actions tab

### Build Failures
- Check if code compiles locally: `cargo build --release`
- Verify all tests pass: `cargo test`
- Check for platform-specific issues in workflow logs

### Release Not Created
- Ensure tag was pushed successfully
- Check release.yml workflow logs
- Verify permissions are set correctly (contents: write)

### Wrong Version Bump Type
- Adjust PR title to match Conventional Commits format
- Use `feat:` for features, `fix:` for fixes
- Add `!` suffix for breaking changes: `feat!:` or `fix!:`
- Add labels: "breaking", "feature", "enhancement"

## Manual Overrides

### Manually Create a Release
If you need to manually trigger a release:
```bash
# Bump version
cargo install cargo-edit
cargo set-version --bump patch  # or minor, major

# Commit and tag
git add Cargo.toml Cargo.lock
git commit -m "chore: bump version to X.Y.Z"
git tag -a vX.Y.Z -m "Release vX.Y.Z"
git push origin main --tags
```

### Test Workflow Locally
Use [act](https://github.com/nektos/act) to test workflows locally:
```bash
# Install act
brew install act  # macOS
# or download from https://github.com/nektos/act

# Test auto-release workflow (requires setting up event JSON)
act pull_request -j test

# Test release workflow
act -j build --secret GITHUB_TOKEN=your_token
```

## Security Considerations

1. **GitHub Token**: Uses built-in `GITHUB_TOKEN` with write permissions
2. **Branch Protection**: Recommended to protect main branch and require PR reviews
3. **Checksums**: SHA256 checksums provided for binary verification
4. **Code Signing**: Not currently implemented (future enhancement)

## Future Enhancements

Possible improvements:
- [ ] Add code signing for macOS and Windows binaries
- [ ] Publish to package managers (Homebrew, Chocolatey, cargo)
- [ ] Add automatic changelog generation
- [ ] Implement pre-release builds for feature branches
- [ ] Add Docker image releases
- [ ] Integrate with deployment tracking systems
