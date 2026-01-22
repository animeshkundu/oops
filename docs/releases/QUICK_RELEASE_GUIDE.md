# Quick Release Guide

## TL;DR

Releases happen automatically when you merge a PR. The PR title determines the version bump.

## PR Title Format

| Want | Use | Example |
|------|-----|---------|
| **Minor** bump (0.1.0 → 0.2.0) | Any PR title | `feat: add kubectl rules` |
| **Minor** bump (0.2.0 → 0.3.0) | Any PR title | `fix: handle empty input` |
| **Minor** bump (0.3.0 → 0.4.0) | Any PR title | `docs: update README` |
| **No release** | Add `[skip release]` | `docs: typo fix [skip release]` |

**Note:** All merged PRs trigger a minor version bump. For major or patch bumps, use manual releases.

## What Happens

1. You merge PR to master
2. Tests run on Linux, macOS, Windows
3. If all pass, version is bumped automatically
4. Tag is created (e.g., v0.2.0)
5. Binaries are built for 6 platforms
6. GitHub release is created with all binaries

## Common Patterns

```bash
# Any PR → minor bump
PR title: "feat: add support for podman commands"
Result: 0.1.0 → 0.2.0

# Another PR → minor bump
PR title: "fix: resolve crash on invalid input"
Result: 0.2.0 → 0.3.0

# Documentation → minor bump (unless skipped)
PR title: "docs: update installation guide"
Result: 0.3.0 → 0.4.0

# Skip release for minor changes
PR title: "docs: fix typo [skip release]"
Result: No release, no version change
```

## Alternative: Skip Release

Instead of relying on PR title prefixes, you can use `[skip release]` to skip a release:

- `[skip release]` in PR title → No release
- `[no release]` in PR title → No release

All other PRs will trigger a **minor version bump**.

## Manual Release (if needed)

```bash
# Bump version
cargo set-version 0.2.0

# Update lockfile
cargo update -p oops

# Commit
git add Cargo.toml Cargo.lock
git commit -m "chore: bump version to 0.2.0"
git push

# Tag and push
git tag v0.2.0
git push --tags
```

## Troubleshooting

**Release didn't happen?**
- Check if `[skip release]` is in PR title
- Verify PR was merged (not just closed)
- Check GitHub Actions tab for errors

**Wrong version bump?**
- All PRs trigger minor bumps automatically
- For major or patch bumps, create a manual release (see above)

**Need help?**
- See [AUTOMATED_RELEASES.md](./AUTOMATED_RELEASES.md) for details
- See [CONTRIBUTING.md](../development/CONTRIBUTING.md) for full guide
