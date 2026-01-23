# PR-Based Release Testing Implementation

**Date**: 2026-01-23  
**Author**: CI/CD Expert Agent  
**Status**: Completed

## Summary

Updated the CI workflow to run auto-release pipeline as a final test step for pull requests (same-repo PRs only). This allows testing the release pipeline pre-merge without actually publishing releases, catching version bump issues before they reach main.

## Changes Made

### 1. Updated `.github/workflows/ci.yml`

#### Modified `auto-release` Job Trigger
**Before**: Only ran on `push` events to main/master (post-merge)
**After**: Runs on both:
- `push` events to main/master (production mode - creates release)
- `pull_request` events to main/master (test mode - validates pipeline)

**Key safeguards added**:
- Fork PRs are blocked: `github.event.pull_request.head.repo.full_name == github.repository`
- Version bump commits are skipped for both events
- Auto-merge only enabled for `push` events (not PR tests)

#### Updated Version Bump Logic

**Event-aware processing**:
- **On `pull_request`**: Derives bump type from PR title and labels
- **On `push`**: Derives bump type from commit message (existing behavior)

**Base branch handling**:
- **On `pull_request`**: Uses `github.event.pull_request.base.ref` (correct PR target)
- **On `push`**: Uses `github.ref_name` (existing behavior)

#### Test vs Production Mode

**Test Mode (PR events)**:
- Creates version bump PR with `[TEST]` prefix
- Adds labels: `test`, `do-not-merge`, `automated`
- PR body explains this is a pre-merge test
- Auto-merge is **NOT** enabled
- Creates test branch but doesn't publish

**Production Mode (push events)**:
- Creates version bump PR without prefix
- Adds labels: `release`, `automated`
- Enables auto-merge (if RELEASE_PAT configured)
- Full release pipeline proceeds

#### Updated Workflow Summary

The job now generates different summaries based on mode:
- **Test mode**: Explains what's being tested, what to do with test PR
- **Production mode**: Shows release timeline and next steps

### 2. Documentation Updates

#### Updated Files
- Created this handoff document

#### Documentation To Update (Recommendations)
- `docs/releases/AUTOMATED_RELEASES.md` - Add section about PR-based testing
- `docs/TESTING_AUTO_RELEASE.md` - Update to explain new PR testing mode
- `README.md` - Mention PR-based release validation if relevant

## Technical Details

### Fork Safety
The condition blocks fork PRs from running auto-release:
```yaml
github.event.pull_request.head.repo.full_name == github.repository
```

This prevents:
- Security issues (forks can't create release PRs)
- Noise (fork PRs don't need release testing)
- Token errors (forks can't access org secrets)

### Skip Conditions Extended
Now checks both commit messages and PR titles:
```yaml
!contains(github.event.head_commit.message, 'chore: bump version') &&
!contains(github.event.head_commit.message, 'chore: release') &&
!contains(github.event.pull_request.title, 'chore: bump version') &&
!contains(github.event.pull_request.title, 'chore: release')
```

### Base Branch Correctness
Critical fix for PR context - uses the PR's actual base branch:
```bash
if [ "$EVENT_NAME" = "pull_request" ]; then
  BASE_BRANCH="${{ github.event.pull_request.base.ref }}"
else
  BASE_BRANCH="${{ github.ref_name }}"
fi
```

Previously would have used `github.ref_name` which is incorrect for PRs (points to merge ref).

## Usage

### For Developers

When creating a PR to main/master:
1. All normal CI checks run (test, msrv, coverage, shell-tests)
2. If checks pass, `auto-release` job runs as final step
3. Creates a test version bump PR marked with `[TEST]` prefix
4. Review the test PR to verify version bump logic
5. **Do not merge the test PR** - it's automatically labeled `do-not-merge`
6. Merge your actual PR
7. Post-merge: Production auto-release runs and creates real release PR

### For Maintainers

**To skip release testing on a PR**:
Add `[skip release]` or `[no release]` to PR title:
```
feat: add new feature [skip release]
```

**To test specific bump types**:
Use conventional commit format in PR title:
- `feat: new feature` → minor bump
- `fix: bug fix` → patch bump  
- `feat!: breaking change` → major bump

Or use labels:
- `feature` or `enhancement` → minor bump
- `breaking` or `breaking-change` → major bump

## Benefits

### 1. Early Validation
Catches version bump issues before merge:
- Incorrect bump type logic
- Cargo.toml/Cargo.lock update failures
- Branch/PR creation problems
- Label and metadata issues

### 2. Confidence
Developers can see the release pipeline will work before merging their PR.

### 3. Transparency
Version bump decisions are visible in PR checks, not hidden until post-merge.

### 4. Zero Production Impact
Test PRs are clearly marked and won't be merged:
- `[TEST]` prefix in title
- `do-not-merge` label
- Explicit warning in PR body

### 5. Maintains Security
Fork PRs cannot trigger auto-release (security safeguard).

## Testing

### Recommended Tests

1. **Test PR from same repo**:
   - Create a PR with `feat:` title
   - Verify auto-release runs and creates test PR
   - Verify test PR has correct labels
   - Close test PR

2. **Test PR from fork**:
   - Create a PR from a fork
   - Verify auto-release does NOT run
   - No test PR should be created

3. **Test production mode**:
   - Merge a PR to main
   - Verify production auto-release runs
   - Verify production PR created (no `[TEST]` prefix)
   - Verify auto-merge enabled (if RELEASE_PAT configured)

4. **Test skip release**:
   - Create PR with `[skip release]` in title
   - Verify auto-release job runs but skips
   - No version bump PR created

## Edge Cases Handled

1. **Fork PRs**: Blocked via repository comparison
2. **Version bump PRs**: Skipped via title check
3. **Already exists**: Checks if version bump branch exists before creating
4. **Base branch**: Correctly uses PR base, not merge ref
5. **Auto-merge**: Only enabled for production (push events)
6. **Labels**: Different labels for test vs production

## Known Limitations

1. **Test PRs require manual cleanup**: Must close test PRs manually or set up auto-cleanup
2. **Noise in PR list**: Test PRs appear in PR list (filtered by `do-not-merge` label)
3. **No dry-run for auto-merge**: Can't test auto-merge behavior in test mode

## Future Improvements

### Potential Enhancements
1. **Auto-close test PRs**: Add job to close test PRs after validation
2. **Comment on source PR**: Post test results as comment on triggering PR
3. **Dry-run mode**: Simulate full pipeline without creating actual PRs
4. **Test PR lifetime**: Auto-close after 1 hour
5. **More bump indicators**: Support more label types (bugfix, chore, etc.)

### Documentation
1. Add animated GIF showing PR testing in action
2. Update CONTRIBUTING.md with PR testing info
3. Create troubleshooting guide for test failures

## Rollback Plan

If issues arise, revert to push-only auto-release:

```yaml
if: |
  github.event_name == 'push' &&
  (github.ref == 'refs/heads/main' || github.ref == 'refs/heads/master') &&
  !contains(github.event.head_commit.message, 'chore: bump version') &&
  !contains(github.event.head_commit.message, 'chore: release')
```

Remove event-aware logic from:
- Check release step
- Determine bump type step
- Create PR step
- Enable auto-merge step
- Workflow summary step

## References

- Original CI workflow: `.github/workflows/ci.yml`
- Release tag workflow: `.github/workflows/create-release-tag.yml` (unchanged)
- Auto-release workflow: `.github/workflows/auto-release.yml` (legacy, unchanged)

## Questions & Answers

**Q: Will this double the number of PRs?**  
A: Yes, but test PRs are clearly marked with `[TEST]` and `do-not-merge` labels for easy filtering.

**Q: What if someone accidentally merges a test PR?**  
A: The create-release-tag workflow checks for `release` label, so test PRs won't trigger releases. However, the version bump will be merged.

**Q: Can I disable this for my PR?**  
A: Yes, add `[skip release]` to your PR title.

**Q: Does this work with forks?**  
A: No, intentionally blocked for security. Fork PRs won't run auto-release.

**Q: What about performance impact?**  
A: Minimal - auto-release only runs after all other jobs pass. Adds ~30-60 seconds for version bump logic.

## Conclusion

This change improves the release pipeline by adding pre-merge validation while maintaining all existing functionality. The implementation is conservative (test PRs clearly marked) and safe (forks blocked, auto-merge only in production).

Key achievement: **Shift-left testing** for the release pipeline - catch issues before merge, not after.
