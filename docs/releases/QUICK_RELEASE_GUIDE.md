# Quick Release Guide

## TL;DR

Releases happen automatically when you merge a PR. The PR title determines the version bump.

## PR Title Format

| Want | Use | Example |
|------|-----|---------|
| **Patch** bump (0.1.0 → 0.1.1) | `fix:` or `docs:` or `chore:` | `fix: handle empty input` |
| **Minor** bump (0.1.0 → 0.2.0) | `feat:` | `feat: add kubectl rules` |
| **Major** bump (0.1.0 → 1.0.0) | `feat!:` or `fix!:` | `feat!: redesign CLI interface` |
| **No release** | Add `[skip release]` | `docs: typo fix [skip release]` |

## What Happens

1. You merge PR to master
2. Tests run on Linux, macOS, Windows
3. If all pass, version is bumped automatically
4. Tag is created (e.g., v0.2.0)
5. Binaries are built for 6 platforms
6. GitHub release is created with all binaries

## Common Patterns

```bash
# Bug fix → patch bump
PR title: "fix: resolve crash on invalid input"
Result: 0.1.0 → 0.1.1

# New feature → minor bump
PR title: "feat: add support for podman commands"
Result: 0.1.0 → 0.2.0

# Breaking change → major bump
PR title: "feat!: change configuration file format"
Result: 0.1.0 → 1.0.0

# Documentation only → skip release
PR title: "docs: update README [skip release]"
Result: No release, no version change
```

## Alternative: Labels

Instead of PR title prefix, you can use labels:

- Label `breaking` → Major bump
- Label `feature` or `enhancement` → Minor bump
- Label `bug` or no label → Patch bump

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
- Check PR title format - must match patterns above
- Next time, adjust title before merging

**Need help?**
- See [AUTOMATED_RELEASES.md](./AUTOMATED_RELEASES.md) for details
- See [CONTRIBUTING.md](../development/CONTRIBUTING.md) for full guide
