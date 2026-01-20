# ADR-004: Shell Detection Strategy

## Status

Accepted

## Context

oops needs to know which shell is running to:
- Generate the correct alias function
- Parse history files correctly
- Combine commands with proper operators
- Handle shell-specific behaviors

## Decision

Shell detection uses a layered approach:

1. **Environment variable** (highest priority): `TF_SHELL`
2. **Process tree inspection** (Unix): Walk parent processes
3. **Default fallback**: Bash

## Implementation

### Layer 1: Environment Variable

The shell alias sets `TF_SHELL`:
```bash
export TF_SHELL=bash
```

This is the most reliable method since the alias knows its own shell.

### Layer 2: Process Tree (Unix)

On Unix systems, we can inspect the parent process:
```rust
fn detect_shell_from_process() -> Option<String> {
    // Walk up process tree looking for known shell names
    // bash, zsh, fish, tcsh, etc.
}
```

This works when oops is run directly without the alias.

### Layer 3: Default

If all else fails, default to Bash as it's the most common.

## Consequences

### Positive

- **Reliable**: Environment variable set by alias is always correct
- **Flexible**: Works even when run without alias
- **Explicit**: User can override with `TF_SHELL=fish oops`

### Negative

- **Process tree inspection**: Platform-specific code
- **No alias**: May detect wrong shell if run directly

### Shell-Specific Behaviors

Each shell has unique characteristics:

| Shell | History Command | Alias Format | Command Join |
|-------|-----------------|--------------|--------------|
| Bash | `fc -ln -10` | `function` | `&&` |
| Zsh | `fc -ln -10` | `function` | `&&` |
| Fish | `$history[1]` | `function` | `; and` |
| PowerShell | `Get-History` | `function` | `-and` |
| Tcsh | `history -h` | `alias` | `&&` |

## Notes

The environment variable approach was inherited from the Python thefuck and has proven reliable. Process tree inspection provides a useful fallback but is not the primary mechanism.

Users experiencing detection issues can always set `TF_SHELL` explicitly.
