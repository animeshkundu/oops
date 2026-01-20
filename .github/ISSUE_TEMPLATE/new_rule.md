---
name: New Rule Request
about: Suggest a new correction rule
title: '[RULE] '
labels: new-rule
assignees: ''
---

## Rule Description

What command mistake should this rule fix?

## Example

```bash
# The wrong command
$ <wrong command>
<error output>

# After running oops, should suggest:
$ <corrected command>
```

## Detection Pattern

How can we detect this error? What patterns appear in:
- The command itself?
- The error output?

## Frequency

How often do you encounter this mistake?
- [ ] Daily
- [ ] Weekly
- [ ] Occasionally
- [ ] Rarely

## Additional Context

- Related tools or commands
- Edge cases to consider
- Any existing rules that are similar
