# Documentation Update Checklist for Workflow Change

**Purpose:** Track documentation updates required for the "always minor" version bump policy

---

## Priority: HIGH (REQUIRED for merge)

### 1. ✅ `docs/releases/AUTOMATED_RELEASES.md`

**Location:** Lines 191-218 (Version Bumping Logic section)

**Current State:**
```markdown
### Version Bumping Logic

The system analyzes your PR title and labels to determine the version bump:

### Major Version Bump (0.1.0 → 1.0.0)
**Triggered by:**
- PR title starts with `feat!:` or `fix!:`
- PR title contains word `breaking`
- PR has label `breaking`

### Minor Version Bump (0.1.0 → 0.2.0)
**Triggered by:**
- PR title starts with `feat:`
- PR has label `feature` or `enhancement`

### Patch Version Bump (0.1.0 → 0.1.1)
**Triggered by:**
- PR title starts with `fix:` (without `!`)
- PR title starts with `docs:`, `chore:`, etc.
- Default if no special indicators
```

**Required Change:**
```markdown
### Version Bumping Logic

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
```

**Status:** ⬜ Not started

---

### 2. ✅ `docs/releases/QUICK_RELEASE_GUIDE.md`

**Location:** Lines 8-14 (PR Title Format table)

**Current State:**
```markdown
| Want | Use | Example |
|------|-----|---------|
| **Minor** bump (0.1.0 → 0.2.0) | Any PR title | `feat: add kubectl rules` |
| **Minor** bump (0.2.0 → 0.3.0) | Any PR title | `fix: handle empty input` |
| **Minor** bump (0.3.0 → 0.4.0) | Any PR title | `docs: update README` |
| **No release** | Add `[skip release]` | `docs: typo fix [skip release]` |
```

**Required Change:**
Already correct! Just verify consistency with other examples in the file.

**Status:** ✅ Already correct (verify only)

---

## Priority: MEDIUM (RECOMMENDED)

### 3. ⚠️ `docs/releases/auto-release-workflow.md`

**Location:** Flowchart section

**Current State:** Contains flowchart showing "Determine version bump (major/minor/patch)"

**Required Change:** Simplify flowchart to show "Determine version bump (always minor)"

**Status:** ⬜ Not started

---

### 4. ⚠️ `docs/RELEASE_WORKFLOW_QA.md`

**Location:** Test cases section

**Current State:** Contains test cases for patch/major bumps

**Required Change:** Update test expectations to reflect minor-only bumps

**Status:** ⬜ Not started

---

## Priority: LOW (OPTIONAL)

### 5. 💡 `docs/summaries/RELEASE_FIX_SUMMARY.md`

**Location:** Historical summary

**Required Change:** Add note about policy change (optional - this is a historical doc)

**Status:** ⬜ Not started

---

### 6. 💡 `docs/summaries/IMPLEMENTATION_SUMMARY.md`

**Location:** Historical summary

**Required Change:** Update for completeness (optional)

**Status:** ⬜ Not started

---

## Verification Checklist

After updating documentation:

- [ ] All examples show minor bumps (0.x.0 → 0.x+1.0)
- [ ] No references to "patch bump" unless in manual override section
- [ ] No references to "major bump" unless in manual override section
- [ ] `[skip release]` documented clearly
- [ ] Manual override options documented
- [ ] CONTRIBUTING.md is consistent (already done)
- [ ] QUICK_RELEASE_GUIDE.md is consistent (already done)
- [ ] AUTOMATED_RELEASES.md updated (priority 1)
- [ ] auto-release-workflow.md updated (priority 2)

---

## Search and Replace Patterns

To help with updates, here are some common phrases to replace:

**Remove these concepts (unless in "manual override" context):**
- "PR title starts with `feat:`" → "Any PR merged to master"
- "PR title starts with `fix:`" → "Any PR merged to master"
- "PR title starts with `feat!:`" → Move to "Manual Version Bumps" section
- "patch bump" → "minor bump" (unless describing manual override)
- "major bump" → Move to "Manual Version Bumps" section

**Update examples:**
- `fix: bug fix` → 0.1.0 → 0.1.1 ❌ Old
- `fix: bug fix` → 0.1.0 → 0.2.0 ✅ New

---

## Testing After Documentation Update

Once documentation is updated, verify:

1. **Read through user perspective:**
   - Can a new contributor understand the release process?
   - Is the "always minor" policy clear?
   - Are manual override options clearly documented?

2. **Check cross-references:**
   - Do all documents point to the same policy?
   - Are there any contradictions?

3. **Verify examples:**
   - Do all version bump examples show minor bumps?
   - Are edge cases covered?

---

## Completion Status

**Required (HIGH):**
- ⬜ AUTOMATED_RELEASES.md updated
- ✅ QUICK_RELEASE_GUIDE.md verified

**Recommended (MEDIUM):**
- ⬜ auto-release-workflow.md updated
- ⬜ RELEASE_WORKFLOW_QA.md updated

**Optional (LOW):**
- ⬜ RELEASE_FIX_SUMMARY.md noted
- ⬜ IMPLEMENTATION_SUMMARY.md noted

**Overall:** 1/6 complete (16%)

---

**Last Updated:** 2025-01-22  
**Tracking:** Documentation updates for "always minor" workflow change
