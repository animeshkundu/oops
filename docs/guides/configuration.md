# Configuration Guide

oops can be configured via a TOML file or environment variables.

## Configuration File

Create `~/.config/oops/config.toml`:

```toml
# Rule Configuration
rules = ["ALL"]                    # Enable all rules
exclude_rules = ["git_push_force"] # Disable specific rules

# Behavior
require_confirmation = true        # Ask before executing
wait_command = 3                   # Command timeout (seconds)
wait_slow_command = 15             # Timeout for slow commands

# Display
no_colors = false                  # Disable colored output
num_close_matches = 3              # Number of suggestions

# History
history_limit = 1000               # Max history entries to check
alter_history = true               # Add corrections to history

# Performance
slow_commands = [                  # Commands that take longer
    "lein",
    "react-native",
    "gradle",
    "./gradlew",
    "vagrant"
]

# Environment
excluded_search_path_prefixes = [] # Paths to skip when searching
```

## Configuration Options

### `rules`

Controls which rules are enabled.

```toml
# Enable all rules (default)
rules = ["ALL"]

# Enable only specific rules
rules = ["sudo", "git_push", "git_checkout"]

# Enable rules matching a pattern
rules = ["git_*", "sudo"]
```

### `exclude_rules`

Disable specific rules even when using `ALL`:

```toml
rules = ["ALL"]
exclude_rules = [
    "git_push_force",  # Don't suggest force push
    "rm_root"          # Don't suggest rm with sudo
]
```

### `require_confirmation`

Whether to ask for confirmation before executing:

```toml
# Ask for confirmation (default)
require_confirmation = true

# Auto-execute first suggestion
require_confirmation = false
```

Can also be overridden with `-y` flag:
```bash
oops -y  # Skip confirmation
```

### `wait_command`

Timeout in seconds for re-running failed commands:

```toml
wait_command = 3  # Default: 3 seconds
```

### `wait_slow_command`

Timeout for commands in `slow_commands` list:

```toml
wait_slow_command = 15  # Default: 15 seconds
```

### `slow_commands`

Commands that need longer timeout:

```toml
slow_commands = [
    "lein",
    "react-native",
    "gradle",
    "./gradlew",
    "vagrant",
    "docker build"
]
```

### `no_colors`

Disable colored output:

```toml
no_colors = true  # Default: false
```

### `num_close_matches`

Number of suggestions to offer:

```toml
num_close_matches = 3  # Default: 3
```

### `history_limit`

Maximum history entries to search:

```toml
history_limit = 1000  # Default: unlimited
```

### `alter_history`

Whether to add corrected commands to shell history:

```toml
alter_history = true  # Default: true
```

### `priority`

Override rule priorities (lower = higher priority):

```toml
[priority]
sudo = 10       # Run sudo rule first
no_command = 5000  # Run fuzzy match last
```

### `env`

Extra environment variables when running commands:

```toml
[env]
LC_ALL = "C"
GIT_TRACE = "1"
```

## Environment Variables

All settings can be overridden via environment variables:

| Variable | Type | Example |
|----------|------|---------|
| `THEFUCK_RULES` | colon-separated | `sudo:git_push:git_checkout` |
| `THEFUCK_EXCLUDE_RULES` | colon-separated | `git_push_force:rm_root` |
| `THEFUCK_REQUIRE_CONFIRMATION` | bool | `true` or `false` |
| `THEFUCK_WAIT_COMMAND` | integer | `5` |
| `THEFUCK_WAIT_SLOW_COMMAND` | integer | `30` |
| `THEFUCK_NO_COLORS` | bool | `true` or `false` |
| `THEFUCK_NUM_CLOSE_MATCHES` | integer | `5` |
| `THEFUCK_HISTORY_LIMIT` | integer | `500` |
| `THEFUCK_ALTER_HISTORY` | bool | `true` or `false` |
| `THEFUCK_SLOW_COMMANDS` | colon-separated | `lein:gradle:vagrant` |
| `THEFUCK_DEBUG` | bool | `true` or `false` |
| `THEFUCK_PRIORITY` | key=value pairs | `sudo=10:no_command=5000` |

Example:
```bash
export THEFUCK_REQUIRE_CONFIRMATION=false
export THEFUCK_NUM_CLOSE_MATCHES=5
```

## Configuration Priority

Settings are applied in this order (later overrides earlier):

1. Default values
2. Config file (`~/.config/oops/config.toml`)
3. Environment variables
4. Command-line arguments

## Debug Mode

Enable debug output to troubleshoot:

```bash
oops --debug
# or
THEFUCK_DEBUG=true oops
```

This shows:
- Which rules matched
- Why rules didn't match
- Configuration values
- Timing information

## Example Configurations

### Minimal (power user)

```toml
require_confirmation = false
```

### Safe (cautious user)

```toml
require_confirmation = true
exclude_rules = ["git_push_force", "rm_root", "sudo"]
```

### Developer

```toml
rules = ["ALL"]
slow_commands = ["npm", "yarn", "gradle", "docker"]
wait_slow_command = 30
num_close_matches = 5
```
