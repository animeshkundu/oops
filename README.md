# oops

A blazingly fast command-line typo corrector written in Rust.

```bash
$ git psuh
git: 'psuh' is not a git command. Did you mean 'push'?

$ oops
git push [enter/up/down/ctrl+c]
```

## Features

- **175+ correction rules** for common CLI mistakes
- **Sub-50ms startup time** - 10x faster than Python alternatives
- **Single binary** - no runtime dependencies
- **Cross-platform** - Linux, macOS, Windows
- **Shell integration** - Bash, Zsh, Fish, PowerShell, Tcsh

## Installation

### From Source (Recommended)

```bash
cargo install --git https://github.com/oops-cli/oops
```

### Pre-built Binaries

Download from [GitHub Releases](https://github.com/oops-cli/oops/releases).

### Package Managers

```bash
# Homebrew (macOS/Linux)
brew install oops-cli/tap/oops

# Cargo
cargo install oops

# Arch Linux (AUR)
yay -S oops

# Windows (Scoop)
scoop install oops
```

## Quick Start

After installation, add the shell integration to your config:

### Bash

Add to `~/.bashrc`:
```bash
eval "$(oops --alias)"
```

### Zsh

Add to `~/.zshrc`:
```zsh
eval "$(oops --alias)"
```

### Fish

Add to `~/.config/fish/config.fish`:
```fish
oops --alias | source
```

### PowerShell

Add to your `$PROFILE`:
```powershell
Invoke-Expression (oops --alias | Out-String)
```

Then restart your shell or source your config file.

## Usage

Just type `oops` after a failed command:

```bash
$ git statis
git: 'statis' is not a git command. Did you mean 'status'?

$ oops
git status [enter/up/down/ctrl+c]
```

### Automatic Execution

Use `-y` to automatically execute the first suggestion:

```bash
$ oops -y
```

### Navigation

When multiple corrections are available:
- **Up/Down arrows** or **j/k** - Navigate options
- **Enter** - Execute selected command
- **Ctrl+C** - Cancel

## Configuration

Create `~/.config/oops/config.toml`:

```toml
# Enable/disable rules
rules = ["ALL"]
exclude_rules = []

# Require confirmation before execution
require_confirmation = true

# Command timeout in seconds
wait_command = 3
wait_slow_command = 15

# Disable colors
no_colors = false

# Number of suggestions to show
num_close_matches = 3

# Commands that take longer to run
slow_commands = ["lein", "react-native", "gradle", "vagrant"]
```

### Environment Variables

All settings can be overridden via environment variables:

| Variable | Description |
|----------|-------------|
| `THEFUCK_RULES` | Colon-separated list of enabled rules |
| `THEFUCK_EXCLUDE_RULES` | Rules to disable |
| `THEFUCK_REQUIRE_CONFIRMATION` | `true` or `false` |
| `THEFUCK_WAIT_COMMAND` | Timeout in seconds |
| `THEFUCK_DEBUG` | Enable debug output |

## Supported Rules

oops includes 175+ rules for common mistakes:

### Git
- `git_push` - Set upstream branch
- `git_checkout` - Fix branch name typos
- `git_add` - Add untracked files
- `git_commit_amend` - Amend commits
- ... and 45+ more git rules

### Package Managers
- APT, Brew, Cargo, npm, pip, Pacman, dnf, yum, conda, gem, choco

### System Commands
- `sudo` - Add sudo for permission errors
- `cd_mkdir` - Create directory before cd
- `chmod_x` - Make scripts executable
- `mkdir_p` - Add -p flag to mkdir

### Development Tools
- Go, Java, Maven, Gradle, Terraform, Docker, Kubernetes

### And More
- Shell utilities, cloud CLIs (AWS, Azure), frameworks (Rails, React Native)

See the [full rules list](docs/guides/rules.md) for details.

## Migrating from thefuck

If you're coming from the Python `thefuck`:

1. **Shell alias**: Change `fuck` to `oops` in your shell config
2. **Config format**: Use TOML instead of Python (`config.toml` instead of `settings.py`)
3. **Environment variables**: Same names, fully compatible

See the [migration guide](docs/guides/migration-from-thefuck.md) for details.

## Performance

| Metric | oops (Rust) | thefuck (Python) |
|--------|-------------|------------------|
| Startup time | ~30ms | ~300ms |
| Binary size | ~5MB | N/A (requires Python) |
| Memory usage | ~10MB | ~50MB |

## Contributing

Contributions are welcome! See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

### Adding a New Rule

1. Create a new struct implementing the `Rule` trait
2. Add it to the appropriate module in `src/rules/`
3. Register it in `src/rules/mod.rs`
4. Add tests

## License

MIT License - see [LICENSE](LICENSE) for details.

## Acknowledgments

oops is inspired by [thefuck](https://github.com/nvbn/thefuck) by Vladimir Iakovlev. Configuration environment variables are backward compatible for easy migration.
