# Handoff Notes

This directory contains handoff notes that preserve context between LLM sessions and document significant development work.

## Purpose

Handoff notes serve multiple purposes:
1. **Context Preservation**: Help future LLM sessions understand previous work
2. **Knowledge Transfer**: Document decisions, approaches, and learnings
3. **Historical Record**: Track evolution of the codebase
4. **Debugging Aid**: Capture edge cases and solutions for future reference

## File Naming Convention

Use the format: `YYYY-MM-DD-descriptive-title.md`

Examples:
- `2026-01-22-reorganize-documentation.md`
- `2026-01-20-fix-windows-clippy-failures.md`
- `2026-01-15-implement-git-branch-rules.md`

## When to Create a Handoff Note

Create a handoff note when:
- ✅ Completing a major feature or refactoring (>1 hour of work)
- ✅ Solving a complex or subtle problem
- ✅ Making architectural decisions
- ✅ Discovering important patterns or anti-patterns
- ✅ Debugging issues that took significant effort
- ✅ Adding multiple related rules or features
- ✅ Making changes that affect multiple modules

Don't create handoff notes for:
- ❌ Simple typo fixes
- ❌ Trivial documentation updates
- ❌ Single-line bug fixes
- ❌ Routine maintenance tasks

## Template

See `.github/copilot-instructions.md` for the full handoff note template.

Quick template:
```markdown
# [Descriptive Title]

**Date**: YYYY-MM-DD  
**Time**: HH:MM UTC  
**Agent**: [Your identifier]  
**Context**: [Why this work was needed]

## Summary
[2-3 sentence overview]

## Key Decisions
- Decision and rationale

## Technical Details
### Changes Made
- Files and changes

### Challenges Faced
- Problems and solutions

## Testing
- What was tested

## Future Considerations
- Follow-up work needed

## References
- Links to resources

## Handoff Context for Next Session
[What the next session needs to know]
```

## Best Practices

1. **Be Specific**: Include concrete details with file paths, line numbers, error messages
2. **Explain Why**: Document rationale for decisions, not just what was done
3. **Include Failures**: Failed approaches are valuable knowledge
4. **Link Resources**: Add all web searches and references used
5. **Use ISO Timestamps**: Format dates as YYYY-MM-DD and times as HH:MM UTC
6. **Keep It Fresh**: Create notes soon after completing work while details are fresh
7. **Update Index**: Add significant notes to `/docs/README.md`

## Reading Handoff Notes

When starting a new task:
1. Check for recent handoff notes (last 7-30 days)
2. Search for notes related to your current area of work
3. Review notes from previous work on the same feature
4. Use notes to avoid repeating mistakes or reinventing solutions

## Maintenance

- Archive notes older than 6 months to `/docs/handoffs/archive/`
- Update the index in `/docs/README.md` when adding important notes
- Remove notes that are no longer relevant (after verifying with project maintainers)
