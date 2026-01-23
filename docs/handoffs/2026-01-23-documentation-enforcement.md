# Documentation Structure Enforcement

**Date**: 2026-01-23  
**Time**: 00:26 UTC  
**Agent**: GitHub Copilot Agent (Claude-based)  
**Context**: Follow-up to documentation reorganization (2026-01-22). User requested enforcement of documentation placement policy through automated tests and completion of handoff documentation.

## Summary

Added automated test to enforce documentation structure policy: only README.md, CLAUDE.md, and AGENT.md are allowed at repository root, all other markdown files must be under `/docs`. Completed the documentation reorganization by moving two remaining summary files (RELEASE_WORKFLOW_FIX.md and SOLUTION_SUMMARY.md) and creating this handoff note.

## Key Decisions

- **Test-based enforcement**: Implemented integration test rather than git hook or CI script
  - **Rationale**: Integrates naturally with existing test suite, runs automatically in CI/CD, fails fast before merge, consistent with project testing practices

- **Case-insensitive checking**: Test accepts README.md, readme.md, ReadMe.md, etc.
  - **Rationale**: Prevents false positives on case-insensitive filesystems (macOS, Windows), matches common GitHub convention variations

- **Minimal test design**: Single test function, clear violation messages, no external dependencies
  - **Rationale**: Follows existing test patterns in `tests/cli_tests.rs` and `tests/parity_tests.rs`, easy to understand and maintain

## Technical Details

### Changes Made

**New test file**: `tests/docs_structure.rs`
- Reads repository root using `env!("CARGO_MANIFEST_DIR")`
- Checks all `.md` files at root level
- Allows only: `readme.md`, `claude.md`, `agent.md` (case-insensitive)
- Reports violations with clear error message listing offending files

**Documentation moves** (completed by user prior):
- `RELEASE_WORKFLOW_FIX.md` → `/docs/summaries/RELEASE_WORKFLOW_FIX.md`
- `SOLUTION_SUMMARY.md` → `/docs/summaries/SOLUTION_SUMMARY.md`
- Updated `/docs/README.md` to reference these files in Summaries section

**Handoff documentation**:
- Created this handoff note: `/docs/handoffs/2026-01-23-documentation-enforcement.md`
- Updated `/docs/README.md` Handoffs section to include this note

### Implementation Details

Test implementation approach:
```rust
// Use manifest dir to find repo root
let root_path = Path::new(env!("CARGO_MANIFEST_DIR"));

// Case-insensitive allowed list
let allowed_files = ["readme.md", "claude.md", "agent.md"];

// Check all .md files at root
for entry in fs::read_dir(root_path) {
    if filename_lower.ends_with(".md") {
        if !allowed_files.contains(&filename_lower.as_str()) {
            violations.push(filename_str);
        }
    }
}
```

The test will fail with a clear message:
```
Found markdown files at repository root that should be in /docs:
  - SOME_FILE.md
  - ANOTHER_FILE.md

Only README.md, CLAUDE.md, and AGENT.md are allowed at root.
All other markdown files should be under /docs.
```

### Why This Approach Works

1. **Automatic enforcement**: Runs with `cargo test`, CI/CD catches violations
2. **Clear guidance**: Error message tells developers exactly what to fix
3. **Future-proof**: New contributors can't accidentally add root-level docs
4. **Zero maintenance**: No manual checks needed, test runs automatically

## Testing

**Manual verification**:
- ✅ Test file created in `tests/docs_structure.rs`
- ✅ Follows existing test structure and conventions
- ✅ Uses `env!("CARGO_MANIFEST_DIR")` for repo root resolution
- ✅ Case-insensitive comparison implemented
- ✅ Clear assertion messages with violating file list

**Not executed** (per user request):
- Test suite run (`cargo test`)
- Linters/formatters (`cargo clippy`, `cargo fmt`)

## Future Considerations

**Potential enhancements**:
1. **Directory structure validation**: Could extend test to verify `/docs` subdirectory structure (development/, releases/, summaries/, handoffs/)
2. **Link validation**: Test that all documentation links are valid and unbroken
3. **Handoff note validation**: Test that handoff notes follow naming convention (YYYY-MM-DD-*.md)
4. **Summary cleanup**: Archive old summary documents (6+ months) to `/docs/summaries/archive/`

**Technical debt**:
- None introduced

## References

**Related files**:
- `tests/docs_structure.rs` - New test file
- `tests/cli_tests.rs` - Existing test pattern reference
- `/docs/handoffs/2026-01-22-documentation-reorganization.md` - Previous handoff note
- `/docs/README.md` - Documentation index

**Related decisions**:
- Documentation reorganization (2026-01-22)
- Handoff note system establishment (2026-01-22)

## Handoff Context for Next Session

**What's been done**:
- Documentation structure enforcement test added
- Final summary files moved to `/docs/summaries/`
- Handoff documentation completed for this work

**What works well**:
- Test is minimal, clear, and follows project conventions
- Policy is now automatically enforced via test suite
- Documentation structure is complete and enforced

**If you need to modify allowed root files**:
- Edit the `allowed_files` array in `tests/docs_structure.rs`
- Test will immediately reflect new policy
- Remember to use lowercase names for case-insensitive matching

**Known limitations**:
- Test only checks `.md` files, not other documentation formats
- Test doesn't validate `/docs` subdirectory structure
- Test doesn't check for broken links in documentation

**If something breaks**:
- Test failure will clearly list violating files
- Move violating files to appropriate `/docs` subdirectory
- Run `cargo test` to verify fix
