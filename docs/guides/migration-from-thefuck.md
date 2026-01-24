# Migration from thefuck

This guide helps you migrate from the Python [thefuck](https://github.com/nvbn/thefuck) to oops.

## Quick Migration

### 1. Install oops

```bash
cargo install oops
# or download from releases
```

### 2. Update Shell Config

Replace:
```bash
eval "$(thefuck --alias)"
```

With:
```bash
eval "$(oops --alias)"
```

### 3. Convert Configuration

Convert your Python `settings.py` to TOML `config.toml`.

## Configuration Migration

### Python to TOML

**Before** (`~/.config/thefuck/settings.py`):
```python
rules = ['ALL']
exclude_rules = ['git_push_force']
require_confirmation = True
wait_command = 3
no_colors = False
priority = {'sudo': 10}
slow_commands = ['lein', 'gradle']
```

**After** (`~/.config/oops/config.toml`):
```toml
rules = ["ALL"]
exclude_rules = ["git_push_force"]
require_confirmation = true
wait_command = 3
no_colors = false
slow_commands = ["lein", "gradle"]

[priority]
sudo = 10
```

### Key Differences

| Python | TOML |
|--------|------|
| `True`/`False` | `true`/`false` |
| `['item']` | `["item"]` |
| `{'key': value}` | `[section]` with `key = value` |
| `None` | omit the key |
| Comments: `#` | Comments: `#` |

### Environment Variables

**Good news**: Environment variables work identically!

```bash
# These work the same in both
export THEFUCK_RULES="sudo:git_push"
export THEFUCK_REQUIRE_CONFIRMATION=false
export THEFUCK_WAIT_COMMAND=5
```

## Feature Comparison

| Feature | thefuck (Python) | oops (Rust) |
|---------|------------------|-------------|
| Rules | 170 | 175+ |
| Shells | 6 | 6 |
| Config format | Python | TOML |
| Startup time | ~300ms | ~30ms |
| Binary size | N/A (needs Python) | ~5MB |
| Custom rules | Yes (.py files) | No |
| Plugins | Yes (thefuck_contrib_*) | No |
| Instant mode | Yes | Partial |

## What's Different

### No Dynamic Rules

oops doesn't support loading rules from `~/.config/oops/rules/`. All rules are compiled in.

**Workaround**: If you have custom rules, consider:
1. Opening an issue to add the rule upstream
2. Forking and compiling your own version
3. Using environment-based workarounds

### No Plugin Support

Third-party `thefuck_contrib_*` packages don't work with oops.

**Workaround**: Most popular plugin functionality is built-in.

### TOML vs Python Config

Python expressions don't work in TOML:
```python
# This won't work in TOML
rules = [r for r in ALL if 'git' in r]
```

**Workaround**: Use explicit lists or environment variables.

## Shell Integration

Shell aliases are nearly identical. The main difference is the binary name.

### Bash/Zsh

```bash
# thefuck
eval "$(thefuck --alias)"

# oops
eval "$(oops --alias)"
```

### Fish

```fish
# thefuck
thefuck --alias | source

# oops
oops --alias | source
```

### PowerShell

```powershell
# thefuck
Invoke-Expression (thefuck --alias | Out-String)

# oops
Invoke-Expression (oops --alias | Out-String)
```

## Keeping Both

You can run both tools side-by-side:

```bash
# Use different aliases
eval "$(thefuck --alias fuck)"  # Python version as 'fuck'
eval "$(oops --alias)"           # Rust version as 'oops'
```

## Rollback

If you need to go back to thefuck:

1. Uninstall oops
2. Restore your shell config
3. Your Python thefuck installation should still work

## Troubleshooting

### Rules Not Matching

Enable debug mode to see what's happening:
```bash
oops --debug
```

### Wrong Shell Detected

Set explicitly:
```bash
export TF_SHELL=zsh
```

### Config Not Loading

Check file location:
```bash
# Should be at:
~/.config/oops/config.toml

# Not:
~/.config/thefuck/settings.toml  # Old location
```

### Missing Rule

If a thefuck rule isn't in oops:
1. Check if it has a different name
2. Open an issue on GitHub
3. The rule might be in a different category

## Getting Help

- [GitHub Issues](https://github.com/animeshkundu/oops/issues)
- [Documentation](../README.md)
