# Installation Guide

## Quick Install

### Pre-built Binaries

Download the latest release for your platform from [GitHub Releases](https://github.com/oops-cli/oops/releases).

```bash
# Linux (x86_64)
curl -LO https://github.com/oops-cli/oops/releases/latest/download/oops-linux-x86_64
chmod +x oops-linux-x86_64
sudo mv oops-linux-x86_64 /usr/local/bin/oops

# macOS (Intel)
curl -LO https://github.com/oops-cli/oops/releases/latest/download/oops-darwin-x86_64
chmod +x oops-darwin-x86_64
sudo mv oops-darwin-x86_64 /usr/local/bin/oops

# macOS (Apple Silicon)
curl -LO https://github.com/oops-cli/oops/releases/latest/download/oops-darwin-aarch64
chmod +x oops-darwin-aarch64
sudo mv oops-darwin-aarch64 /usr/local/bin/oops
```

### Windows

Download `oops-windows-x86_64.exe` and add to your PATH.

## Package Managers

### Cargo (Rust)

```bash
cargo install oops
```

### Homebrew (macOS/Linux)

```bash
brew install oops-cli/tap/oops
```

### Arch Linux (AUR)

```bash
yay -S oops
# or
paru -S oops
```

### Scoop (Windows)

```powershell
scoop install oops
```

### Chocolatey (Windows)

```powershell
choco install oops
```

## Build from Source

### Prerequisites

- Rust toolchain (1.70 or later)
- Git

### Steps

```bash
# Clone the repository
git clone https://github.com/oops-cli/oops.git
cd oops

# Build release binary
cargo build --release

# Install (Linux/macOS)
sudo cp target/release/oops /usr/local/bin/

# Or add to PATH
export PATH="$PATH:$(pwd)/target/release"
```

## Shell Setup

After installing the binary, set up shell integration.

### Bash

Add to `~/.bashrc`:
```bash
eval "$(oops --alias)"
```

Then reload:
```bash
source ~/.bashrc
```

### Zsh

Add to `~/.zshrc`:
```zsh
eval "$(oops --alias)"
```

Then reload:
```zsh
source ~/.zshrc
```

### Fish

Add to `~/.config/fish/config.fish`:
```fish
oops --alias | source
```

Then reload:
```fish
source ~/.config/fish/config.fish
```

### PowerShell

Add to your profile (`$PROFILE`):
```powershell
Invoke-Expression (oops --alias | Out-String)
```

To find your profile location:
```powershell
echo $PROFILE
```

### Tcsh

Add to `~/.tcshrc`:
```tcsh
eval `oops --alias tcsh`
```

## Verification

Test the installation:
```bash
oops --version
```

Test shell integration:
```bash
# Type a wrong command
git statsu

# Then run oops
oops
# Should suggest: git status
```

## Updating

### Cargo

```bash
cargo install oops --force
```

### Package Managers

Use your package manager's update command.

### Manual

Download the new binary and replace the old one.

## Uninstalling

### Remove Binary

```bash
# If installed to /usr/local/bin
sudo rm /usr/local/bin/oops

# If installed via cargo
cargo uninstall oops
```

### Remove Shell Integration

Remove the `eval "$(oops --alias)"` line from your shell config.

### Remove Configuration

```bash
rm -rf ~/.config/oops
```
