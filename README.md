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

### Pre-built Binaries

Download the appropriate binary for your system from [GitHub Releases](https://github.com/animeshkundu/oops/releases/latest):

| Platform | Architecture | Binary |
|----------|--------------|--------|
| Linux | x86_64 | `oops-linux-x86_64` |
| Linux | x86_64 (static) | `oops-linux-x86_64-musl` |
| Linux | ARM64 | `oops-linux-aarch64` |
| macOS | Intel | `oops-darwin-x86_64` |
| macOS | Apple Silicon | `oops-darwin-aarch64` |
| Windows | x86_64 | `oops-windows-x86_64.exe` |

#### Linux/macOS

```bash
# Download (example for Linux x86_64)
curl -LO https://github.com/animeshkundu/oops/releases/latest/download/oops-linux-x86_64

# Make executable
chmod +x oops-linux-x86_64

# Move to PATH
sudo mv oops-linux-x86_64 /usr/local/bin/oops

# Verify installation
oops --version
```

#### Windows

```powershell
# Download using PowerShell
Invoke-WebRequest -Uri "https://github.com/animeshkundu/oops/releases/latest/download/oops-windows-x86_64.exe" -OutFile "oops.exe"

# Move to a directory in your PATH, or add to PATH
mkdir -Force "$env:USERPROFILE\bin"
Move-Item oops.exe "$env:USERPROFILE\bin\oops.exe"

# Add to PATH (run once)
[Environment]::SetEnvironmentVariable("Path", $env:Path + ";$env:USERPROFILE\bin", "User")
```

### From Source

```bash
cargo install --git https://github.com/animeshkundu/oops
```

### Package Managers

```bash
# Homebrew (macOS/Linux)
brew install animeshkundu/tap/oops

# Cargo (from crates.io when published)
cargo install oops

# Arch Linux (AUR)
yay -S oops

# Windows (Scoop)
scoop bucket add extras
scoop install oops
```

## Shell Integration

After installing `oops`, add the shell integration to your config file:

### Bash

Add to `~/.bashrc`:
```bash
eval "$(oops --alias)"
```

Reload: `source ~/.bashrc`

### Zsh

Add to `~/.zshrc`:
```zsh
eval "$(oops --alias)"
```

Reload: `source ~/.zshrc`

### Fish

Add to `~/.config/fish/config.fish`:
```fish
oops --alias | source
```

Reload: `source ~/.config/fish/config.fish`

### PowerShell

Add to your PowerShell profile (`$PROFILE`):
```powershell
Invoke-Expression (oops --alias | Out-String)
```

To find your profile location: `echo $PROFILE`

Reload: `. $PROFILE`

### Tcsh

Add to `~/.tcshrc`:
```tcsh
eval `oops --alias`
```

Reload: `source ~/.tcshrc`

### Custom Alias Name

To use a different alias (e.g., `fuck` for thefuck compatibility):

```bash
# Bash/Zsh
eval "$(TF_ALIAS=fuck oops --alias)"

# Fish
TF_ALIAS=fuck oops --alias | source

# PowerShell
$env:TF_ALIAS="fuck"; Invoke-Expression (oops --alias | Out-String)

# Tcsh
setenv TF_ALIAS fuck && eval `oops --alias`
```

## Shell Reference

| Shell | Config File | Integration Command | Reload Command |
|-------|-------------|---------------------|----------------|
| Bash | `~/.bashrc` | `eval "$(oops --alias)"` | `source ~/.bashrc` |
| Zsh | `~/.zshrc` | `eval "$(oops --alias)"` | `source ~/.zshrc` |
| Fish | `~/.config/fish/config.fish` | `oops --alias \| source` | `source ~/.config/fish/config.fish` |
| PowerShell | `$PROFILE` | `Invoke-Expression (oops --alias \| Out-String)` | `. $PROFILE` |
| Tcsh | `~/.tcshrc` | `` eval `oops --alias` `` | `source ~/.tcshrc` |

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

### Minimum Supported Rust Version

Oops requires Rust 1.83 or newer to build.

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

1. **Shell alias**: Change `fuck` to `oops` in your shell config (or use `TF_ALIAS=fuck`)
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
