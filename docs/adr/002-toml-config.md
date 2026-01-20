# ADR-002: TOML Configuration Format

## Status

Accepted

## Context

The Python thefuck uses `settings.py` files for configuration:

```python
# ~/.config/thefuck/settings.py
rules = ['ALL']
exclude_rules = ['git_push_force']
require_confirmation = True
wait_command = 3
```

This approach has trade-offs:
- **Pro**: Full Python expressiveness
- **Pro**: Familiar to Python users
- **Con**: Requires Python interpreter to parse
- **Con**: Security risk (arbitrary code execution)
- **Con**: Not portable to non-Python implementations

## Decision

Use TOML for configuration:

```toml
# ~/.config/oops/config.toml
rules = ["ALL"]
exclude_rules = ["git_push_force"]
require_confirmation = true
wait_command = 3
```

## Consequences

### Positive

- **Safe**: No arbitrary code execution
- **Portable**: Standard format, works with any language
- **Simple**: Easy to read and write
- **Fast**: Quick to parse without interpreter
- **Validated**: Schema validation possible
- **Tooling**: Good editor support

### Negative

- **Less flexible**: Can't compute values dynamically
- **Migration**: Existing Python configs need conversion
- **Learning**: Users must learn TOML syntax

### Compatibility

Environment variables remain fully compatible:
- `THEFUCK_RULES`
- `THEFUCK_EXCLUDE_RULES`
- `THEFUCK_REQUIRE_CONFIRMATION`
- etc.

This means users can migrate incrementally.

## Migration Path

1. Environment variables work immediately
2. Convert Python config to TOML manually
3. Use migration script for automated conversion

Example migration:
```python
# Python settings.py
rules = ['sudo', 'git_*']
wait_command = 5
```

Becomes:
```toml
# TOML config.toml
rules = ["sudo", "git_*"]
wait_command = 5
```

## Notes

TOML was chosen over alternatives:
- **JSON**: No comments, verbose
- **YAML**: Whitespace-sensitive, complex
- **INI**: Limited data types
- **TOML**: Simple, readable, typed
