# ADR-003: Compiled Rules

## Status

Accepted

## Context

The Python thefuck supports dynamic rule loading:
- Built-in rules in `thefuck/rules/`
- User rules in `~/.config/thefuck/rules/`
- Third-party rules via `thefuck_contrib_*` packages

This provides flexibility but adds complexity and startup time.

## Decision

All rules are compiled into the oops binary. No dynamic rule loading.

## Consequences

### Positive

- **Performance**: No file system scanning or dynamic imports
- **Simplicity**: Single binary contains everything
- **Reliability**: No missing dependency issues
- **Security**: No arbitrary code execution
- **Consistency**: Same rules on every system

### Negative

- **No custom rules**: Users can't add their own rules without recompiling
- **No plugins**: Third-party rule packages don't work
- **Updates**: New rules require new binary release

### Mitigation Strategies

1. **Comprehensive built-in rules**: 175+ rules cover most use cases
2. **Rule enable/disable**: Users can customize via configuration
3. **Priority override**: Adjust rule priorities in config
4. **Contributions welcome**: Easy to add rules upstream

### Configuration

Rules can still be controlled via config:

```toml
# Enable only specific rules
rules = ["sudo", "git_push", "git_checkout"]

# Or enable all except some
rules = ["ALL"]
exclude_rules = ["git_push_force"]

# Override priorities
[priority]
sudo = 10
no_command = 5000
```

## Future Considerations

If user demand is high, we could explore:
- Plugin system with WASM rules
- Scripted rules in a safe DSL
- Rule definitions in TOML

For now, the compiled approach provides the best balance of performance, simplicity, and reliability.
