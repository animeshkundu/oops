# Testing Auto-Release Workflow Locally

This guide explains how to test the auto-release workflow locally before pushing changes.

## Quick Testing with Test Script

The fastest way to test the version bump logic is using the provided test script:

```bash
# Make the script executable (first time only)
chmod +x test-version-bump.sh

# Run all test scenarios
./test-version-bump.sh
```

This script tests 30+ scenarios covering:
- ✅ Major bumps (breaking changes)
- ✅ Minor bumps (features)
- ✅ Patch bumps (fixes, docs, chores)
- ✅ Edge cases (spacing, case sensitivity)
- ✅ Priority handling (breaking > feature > patch)

### Custom Testing

You can also test specific PR titles manually:

```bash
# Test with just a title
./test-version-bump.sh "feat: add new command"

# Test with title and labels
./test-version-bump.sh "fix: resolve bug" "enhancement,bug"
```

## Testing with `act` (GitHub Actions Locally)

[`act`](https://github.com/nektos/act) allows you to run GitHub Actions workflows locally in Docker containers.

### Installation

**macOS:**
```bash
brew install act
```

**Linux:**
```bash
curl https://raw.githubusercontent.com/nektos/act/master/install.sh | sudo bash
```

**Windows:**
```bash
choco install act-cli
# or
scoop install act
```

### Basic Usage

```bash
# Run the auto-release workflow (requires Docker)
act pull_request -e .github/test-events/pr-merged.json

# Run with specific event
act pull_request:closed -e .github/test-events/pr-feat.json

# Run only the bump_type step (faster)
act pull_request -j create-version-bump-pr -s GITHUB_TOKEN
```

### Creating Test Event Files

Create test event files in `.github/test-events/`:

**pr-feat.json** - Test minor bump:
```json
{
  "action": "closed",
  "pull_request": {
    "merged": true,
    "number": 123,
    "title": "feat: add new command",
    "labels": [],
    "base": {
      "ref": "main"
    }
  }
}
```

**pr-breaking.json** - Test major bump:
```json
{
  "action": "closed",
  "pull_request": {
    "merged": true,
    "number": 124,
    "title": "feat!: redesign API",
    "labels": [
      {"name": "breaking"}
    ],
    "base": {
      "ref": "main"
    }
  }
}
```

**pr-fix.json** - Test patch bump:
```json
{
  "action": "closed",
  "pull_request": {
    "merged": true,
    "number": 125,
    "title": "fix: resolve segfault",
    "labels": [
      {"name": "bug"}
    ],
    "base": {
      "ref": "main"
    }
  }
}
```

### Limitations of `act`

⚠️ **Known Issues:**
- Cannot create actual PRs (requires GitHub API)
- Secrets must be provided manually with `-s` flag
- Some GitHub-specific actions may not work
- Platform-specific builds won't work perfectly

**What works:**
- ✅ Testing version bump logic
- ✅ Running cargo commands
- ✅ Validating bash scripts
- ✅ Checking workflow syntax

**What doesn't work:**
- ❌ Creating actual PRs (`gh pr create`)
- ❌ Enabling auto-merge
- ❌ Cross-platform matrix builds
- ❌ Workflow summaries

## Debugging the Workflow

### Check Workflow Syntax

```bash
# Validate YAML syntax
yamllint .github/workflows/auto-release.yml

# Check for common issues
act --dryrun pull_request -e .github/test-events/pr-feat.json
```

### View Workflow Logs

When running in GitHub Actions:

1. Go to Actions tab in GitHub
2. Click on failed workflow run
3. Expand the "Determine version bump type" step
4. Look for the decision summary:
   ```
   📊 Decision Summary
   ─────────────────────────────────────
   Bump Type: minor
   Reason: conventional commit 'feat:' or 'feat(...)'
   ─────────────────────────────────────
   ```

### Test Locally Without Docker

```bash
# Simulate the bump logic directly
export PR_TITLE="feat: add new command"
export PR_LABELS='["feature"]'

# Run the logic (extracted from workflow)
bash -c 'source .github/scripts/test-bump-logic.sh'
```

## Version Bump Decision Matrix

| PR Title / Label Pattern | Bump Type | Reason |
|--------------------------|-----------|---------|
| `feat!: ...` or `fix!: ...` | **MAJOR** | Conventional commit with `!` |
| `BREAKING CHANGE:` in title | **MAJOR** | Breaking change keyword |
| `[breaking]` tag | **MAJOR** | Breaking tag |
| Label: `breaking` or `breaking-change` | **MAJOR** | Breaking label |
| `feat: ...` or `feat(...):` | **MINOR** | Feature commit |
| `[feat]` tag | **MINOR** | Feature tag |
| Label: `feature` or `enhancement` | **MINOR** | Feature label |
| Everything else | **PATCH** | Default (fix, docs, chore, etc.) |

## Common Test Scenarios

### Test Breaking Changes

```bash
./test-version-bump.sh "feat!: redesign CLI"
# Expected: major

./test-version-bump.sh "fix: resolve issue" "breaking"
# Expected: major
```

### Test Features

```bash
./test-version-bump.sh "feat: add --verbose flag"
# Expected: minor

./test-version-bump.sh "Add new feature" "enhancement"
# Expected: minor
```

### Test Patches

```bash
./test-version-bump.sh "fix: memory leak"
# Expected: patch

./test-version-bump.sh "docs: update README"
# Expected: patch

./test-version-bump.sh "chore: update dependencies"
# Expected: patch
```

### Test Edge Cases

```bash
# No space after colon
./test-version-bump.sh "feat:add command"
# Expected: minor

# Case insensitivity
./test-version-bump.sh "FEAT: Add Command"
# Expected: minor

# Not a feature (contains 'feat' but not at start)
./test-version-bump.sh "defeat: fix typo"
# Expected: patch
```

## Troubleshooting

### Script fails with "grep: invalid option"

Make sure you're using GNU grep:
```bash
# macOS users may need to install GNU grep
brew install grep
export PATH="/usr/local/opt/grep/libexec/gnubin:$PATH"
```

### act fails with "Docker not found"

Install Docker Desktop:
- macOS: `brew install --cask docker`
- Linux: Follow [Docker installation guide](https://docs.docker.com/engine/install/)
- Windows: Install [Docker Desktop](https://www.docker.com/products/docker-desktop)

### Workflow runs but doesn't create PR

This is expected when testing locally with `act`. The PR creation step requires actual GitHub API access. The version bump logic will still execute correctly.

## Manual Workflow Testing Checklist

Before pushing workflow changes:

- [ ] Run `./test-version-bump.sh` - all tests pass
- [ ] Test MAJOR bump: `./test-version-bump.sh "feat!: breaking change"`
- [ ] Test MINOR bump: `./test-version-bump.sh "feat: new feature"`
- [ ] Test PATCH bump: `./test-version-bump.sh "fix: bug fix"`
- [ ] Check YAML syntax: `yamllint .github/workflows/auto-release.yml`
- [ ] Verify workflow file is committed
- [ ] Create test PR with different title formats
- [ ] Observe actual workflow execution in GitHub Actions

## Continuous Testing

Consider adding the test script to your CI:

```yaml
# .github/workflows/test-version-bump.yml
name: Test Version Bump Logic
on: [push, pull_request]
jobs:
  test-logic:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Run version bump tests
        run: ./test-version-bump.sh
```

## References

- [act Documentation](https://github.com/nektos/act)
- [Conventional Commits](https://www.conventionalcommits.org/)
- [Semantic Versioning](https://semver.org/)
- [GitHub Actions Events](https://docs.github.com/en/actions/using-workflows/events-that-trigger-workflows#pull_request)

---

**Pro Tip:** Use the test script in your development workflow to verify PR titles before creating them!
