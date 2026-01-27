<div align="center">

<!-- Logo/Banner Area -->
<h1>
  <br>
  🔧 oops
  <br>
</h1>

<h4>A blazingly fast command-line typo corrector written in Rust</h4>

<p>
  <em>Made a typo? Just say "oops" and fix it instantly.</em>
</p>

<!-- Badges -->
<p>
  <a href="https://github.com/animeshkundu/oops/actions/workflows/ci.yml">
    <img src="https://github.com/animeshkundu/oops/actions/workflows/ci.yml/badge.svg" alt="CI Status">
  </a>
  <a href="https://github.com/animeshkundu/oops/releases/latest">
    <img src="https://img.shields.io/github/v/release/animeshkundu/oops?color=success&label=version" alt="Release">
  </a>
  <a href="LICENSE">
    <img src="https://img.shields.io/badge/license-MIT-blue.svg" alt="License: MIT">
  </a>
  <a href="https://github.com/animeshkundu/oops">
    <img src="https://img.shields.io/badge/rust-1.88+-orange.svg" alt="Rust: 1.88+">
  </a>
  <a href="https://github.com/animeshkundu/oops/releases">
    <img src="https://img.shields.io/github/downloads/animeshkundu/oops/total?color=brightgreen" alt="Downloads">
  </a>
</p>

<!-- Quick Links -->
<p>
  <a href="#-features">Features</a> •
  <a href="#-quick-start">Quick Start</a> •
  <a href="#-installation">Installation</a> •
  <a href="#%EF%B8%8F-configuration">Configuration</a> •
  <a href="#-performance">Performance</a>
</p>

<!-- Demo GIF Area -->
<br>

```bash
$ git psuh
git: 'psuh' is not a git command. Did you mean 'push'?

$ oops
git push [enter/↑/↓/ctrl+c]
```

</div>

---

## ✨ Features

<table>
<tr>
<td width="50%">

### 🚀 Lightning Fast
**Sub-50ms startup** — 10x faster than Python alternatives. No waiting, just fixing.

### 📦 Zero Dependencies  
**Single static binary** — download and run. No Python, no Node, no runtime.

### 🌍 Cross-Platform
**Native support** for Linux, macOS, and Windows with shell integration for all major shells.

</td>
<td width="50%">

### 🧠 177+ Smart Rules
Intelligent corrections for Git, package managers, Docker, AWS, and more.

### 🔄 thefuck Compatible
**Drop-in replacement** — same environment variables, easy migration path.

### ⚡ Instant Mode Ready
Execute corrections automatically or navigate through suggestions.

</td>
</tr>
</table>

---

## 🎬 Examples

<details open>
<summary><strong>Git Commands</strong></summary>

```bash
$ git psuh
git: 'psuh' is not a git command. Did you mean 'push'?

$ oops
git push [enter/↑/↓/ctrl+c]
```

```bash
$ git statis
git: 'statis' is not a git command. Did you mean 'status'?

$ oops
git status
```

```bash
$ git push
fatal: The current branch feature has no upstream branch.

$ oops
git push --set-upstream origin feature
```

</details>

<details>
<summary><strong>Permission Errors</strong></summary>

```bash
$ apt-get install vim
E: Could not open lock file /var/lib/dpkg/lock - open (13: Permission denied)

$ oops
sudo apt-get install vim
```

</details>

<details>
<summary><strong>Typos & Misspellings</strong></summary>

```bash
$ pyhton --version
command not found: pyhton

$ oops
python --version
```

```bash
$ sl
command not found: sl

$ oops
ls
```

</details>

<details>
<summary><strong>Package Managers</strong></summary>

```bash
$ cargo run
error: could not find `Cargo.toml` in `/home/user` or any parent directory

$ oops
# Suggests: cd to project directory or cargo init
```

```bash
$ npm instal express
Unknown command: "instal"

$ oops
npm install express
```

</details>

---

## 🚀 Quick Start

### 1️⃣ Install

```bash
# macOS/Linux (Homebrew)
brew install animeshkundu/tap/oops

# Or download binary directly
curl -LO https://github.com/animeshkundu/oops/releases/latest/download/oops-$(uname -s | tr '[:upper:]' '[:lower:]')-$(uname -m)
chmod +x oops-* && sudo mv oops-* /usr/local/bin/oops
```

### 2️⃣ Setup Shell Integration

Add to your shell config (`.bashrc`, `.zshrc`, etc.):

```bash
eval "$(oops --alias)"
```

### 3️⃣ Use It!

```bash
$ git comit -m "fix"
git: 'comit' is not a git command. Did you mean 'commit'?

$ oops    # Just type oops!
git commit -m "fix"
```

---

## 📥 Installation

### Package Managers

| Manager | Command |
|---------|---------|
| **Homebrew** (macOS/Linux) | `brew install animeshkundu/tap/oops` |
| **Cargo** | `cargo install oops` |
| **AUR** (Arch Linux) | `yay -S oops` |
| **Scoop** (Windows) | `scoop install oops` |

### Pre-built Binaries

Download from [GitHub Releases](https://github.com/animeshkundu/oops/releases/latest):

| Platform | Architecture | Download |
|----------|--------------|----------|
| 🐧 Linux | x86_64 | [`oops-linux-x86_64`](https://github.com/animeshkundu/oops/releases/latest/download/oops-linux-x86_64) |
| 🐧 Linux | x86_64 (static) | [`oops-linux-x86_64-musl`](https://github.com/animeshkundu/oops/releases/latest/download/oops-linux-x86_64-musl) |
| 🐧 Linux | ARM64 | [`oops-linux-aarch64`](https://github.com/animeshkundu/oops/releases/latest/download/oops-linux-aarch64) |
| 🍎 macOS | Intel | [`oops-darwin-x86_64`](https://github.com/animeshkundu/oops/releases/latest/download/oops-darwin-x86_64) |
| 🍎 macOS | Apple Silicon | [`oops-darwin-aarch64`](https://github.com/animeshkundu/oops/releases/latest/download/oops-darwin-aarch64) |
| 🪟 Windows | x86_64 | [`oops-windows-x86_64.exe`](https://github.com/animeshkundu/oops/releases/latest/download/oops-windows-x86_64.exe) |

<details>
<summary><strong>📋 Manual Installation Instructions</strong></summary>

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

# Move to a directory in your PATH
mkdir -Force "$env:USERPROFILE\bin"
Move-Item oops.exe "$env:USERPROFILE\bin\oops.exe"

# Add to PATH (run once)
[Environment]::SetEnvironmentVariable("Path", $env:Path + ";$env:USERPROFILE\bin", "User")
```

</details>

<details>
<summary><strong>🔨 Build from Source</strong></summary>

Requires Rust 1.88 or newer:

```bash
# From GitHub
cargo install --git https://github.com/animeshkundu/oops

# Or clone and build
git clone https://github.com/animeshkundu/oops
cd oops
cargo build --release
```

</details>

---

## 🐚 Shell Integration

Add the integration to your shell configuration file:

<table>
<tr>
<th>Shell</th>
<th>Config File</th>
<th>Add This Line</th>
</tr>
<tr>
<td><strong>Bash</strong></td>
<td><code>~/.bashrc</code></td>
<td><code>eval "$(oops --alias)"</code></td>
</tr>
<tr>
<td><strong>Zsh</strong></td>
<td><code>~/.zshrc</code></td>
<td><code>eval "$(oops --alias)"</code></td>
</tr>
<tr>
<td><strong>Fish</strong></td>
<td><code>~/.config/fish/config.fish</code></td>
<td><code>oops --alias | source</code></td>
</tr>
<tr>
<td><strong>PowerShell</strong></td>
<td><code>$PROFILE</code></td>
<td><code>Invoke-Expression (oops --alias | Out-String)</code></td>
</tr>
<tr>
<td><strong>Tcsh</strong></td>
<td><code>~/.tcshrc</code></td>
<td><code>eval `oops --alias`</code></td>
</tr>
</table>

> 💡 **Tip:** Reload your shell config after adding: `source ~/.bashrc` (or your config file)

### Custom Alias

Want to use a different alias? (e.g., `fuck` for thefuck compatibility)

```bash
# Bash/Zsh
eval "$(TF_ALIAS=fuck oops --alias)"

# Fish
TF_ALIAS=fuck oops --alias | source

# PowerShell
$env:TF_ALIAS="fuck"; Invoke-Expression (oops --alias | Out-String)
```

---

## ⌨️ Usage

### Basic Usage

After a failed command, just type `oops`:

```bash
$ oops              # Show correction with confirmation
$ oops -y           # Auto-execute first suggestion
$ oops --help       # Show all options
```

### Navigation

When multiple corrections are available:

| Key | Action |
|-----|--------|
| `↑` / `↓` or `j` / `k` | Navigate options |
| `Enter` | Execute selected command |
| `Ctrl+C` | Cancel |

---

## ⚙️ Configuration

Create `~/.config/oops/config.toml`:

```toml
# Enable all rules except specific ones
rules = ["ALL"]
exclude_rules = ["rm_rf"]

# Require confirmation before execution (default: true)
require_confirmation = true

# Command timeout in seconds
wait_command = 3
wait_slow_command = 15

# UI options
no_colors = false
num_close_matches = 3

# Commands that need more time
slow_commands = ["lein", "react-native", "gradle", "vagrant"]
```

### Environment Variables

For thefuck compatibility, all settings can be overridden via environment variables:

| Variable | Description | Default |
|----------|-------------|---------|
| `THEFUCK_RULES` | Enabled rules (colon-separated) | `ALL` |
| `THEFUCK_EXCLUDE_RULES` | Disabled rules | - |
| `THEFUCK_REQUIRE_CONFIRMATION` | Require confirmation | `true` |
| `THEFUCK_WAIT_COMMAND` | Command timeout (seconds) | `3` |
| `THEFUCK_DEBUG` | Enable debug output | `false` |
| `TF_ALIAS` | Custom alias name | `oops` |

---

## 📚 Supported Rules

oops includes **177+ intelligent correction rules** organized by category:

<details>
<summary><strong>🔀 Git (50+ rules)</strong></summary>

| Rule | Description |
|------|-------------|
| `git_push` | Set upstream branch automatically |
| `git_checkout` | Fix branch name typos |
| `git_add` | Add untracked/modified files |
| `git_commit_amend` | Suggest amending commits |
| `git_branch_delete` | Change `-d` to `-D` when needed |
| `git_not_command` | Fix misspelled git commands |
| `git_stash` | Fix stash subcommand typos |
| ... | And 40+ more! |

</details>

<details>
<summary><strong>📦 Package Managers</strong></summary>

- **apt** / **apt-get** — Debian/Ubuntu
- **brew** — Homebrew (macOS/Linux)
- **cargo** — Rust
- **npm** / **yarn** — Node.js
- **pip** — Python
- **pacman** / **yay** — Arch Linux
- **dnf** / **yum** — Fedora/RHEL
- **conda** — Anaconda
- **gem** — Ruby
- **choco** — Windows Chocolatey

</details>

<details>
<summary><strong>🐳 Docker & Cloud</strong></summary>

- Docker commands and container management
- AWS CLI (`aws`)
- Azure CLI (`az`)
- Heroku CLI
- Kubernetes (`kubectl`)
- Terraform

</details>

<details>
<summary><strong>🖥️ System & Shell</strong></summary>

| Rule | Description |
|------|-------------|
| `sudo` | Add sudo for permission errors |
| `cd_mkdir` | Create directory before cd |
| `chmod_x` | Make scripts executable |
| `mkdir_p` | Add `-p` flag to mkdir |
| `rm_dir` | Use `rm -r` for directories |
| `cat_dir` | Use `ls` instead of `cat` on directories |
| `touch` | Create parent directories |
| `no_command` | Find similar commands |

</details>

<details>
<summary><strong>🛠️ Development Tools</strong></summary>

- Go (`go run`, `go build`)
- Java / Javac
- Maven (`mvn`)
- Gradle
- Composer (PHP)
- Ruby/Rails
- React Native

</details>

📖 [**Full rules documentation →**](docs/guides/rules.md)

---

## ⚡ Performance

oops is built for speed. Here's how it compares:

| Metric | oops (Rust) | thefuck (Python) | Improvement |
|--------|:-----------:|:----------------:|:-----------:|
| **Startup Time** | ~30ms | ~300ms | **10x faster** |
| **Binary Size** | ~5 MB | N/A (requires Python runtime) | **Self-contained** |
| **Memory Usage** | ~10 MB | ~50 MB | **5x less** |
| **Dependencies** | None | Python 3.5+ | **Zero deps** |

### Why is oops faster?

- 🦀 **Native Rust** — Compiled to machine code, no interpreter overhead
- 📦 **Single Binary** — No Python startup, no module imports
- 🧠 **Lazy Evaluation** — Rules are matched efficiently without unnecessary work
- ⚡ **Optimized Regex** — Uses Rust's high-performance regex engine

---

## 🔄 Migrating from thefuck

Already using thefuck? Migration is seamless:

| Aspect | thefuck | oops |
|--------|---------|------|
| **Shell alias** | `fuck` | `oops` (or `TF_ALIAS=fuck`) |
| **Config file** | `~/.config/thefuck/settings.py` | `~/.config/oops/config.toml` |
| **Config format** | Python | TOML |
| **Env variables** | `THEFUCK_*` | Same! ✅ |

```bash
# Keep using 'fuck' as the command if you prefer
eval "$(TF_ALIAS=fuck oops --alias)"
```

📖 [**Migration guide →**](docs/guides/migration-from-thefuck.md)

---

## 🤝 Contributing

We love contributions! Whether it's:

- 🐛 Bug reports
- 💡 Feature suggestions  
- 📝 Documentation improvements
- 🔧 New correction rules

### Quick Contribution

```bash
# Clone the repo
git clone https://github.com/animeshkundu/oops
cd oops

# Build and test
cargo build
cargo test

# Format and lint
cargo fmt
cargo clippy
```

📖 [**Contributing guide →**](docs/development/CONTRIBUTING.md)

### Adding a New Rule

1. Create a struct implementing the `Rule` trait
2. Add it to the appropriate module in `src/rules/`
3. Register it in `src/rules/mod.rs`
4. Add comprehensive tests
5. Submit a PR!

📖 [**Rule creation guide →**](docs/guides/creating-rules.md)

---

## 📄 License

MIT License — see [LICENSE](LICENSE) for details.

---

## 🙏 Acknowledgments

oops is inspired by the magnificent [**thefuck**](https://github.com/nvbn/thefuck) by Vladimir Iakovlev.

**Key differences:**
- ⚡ 10x faster (Rust vs Python)
- 📦 Single binary (no runtime dependencies)
- 🔄 100% environment variable compatible

---

<div align="center">

**[⬆ Back to Top](#-oops)**

Made with ❤️ by the oops contributors

<a href="https://github.com/animeshkundu/oops/stargazers">
  <img src="https://img.shields.io/github/stars/animeshkundu/oops?style=social" alt="GitHub stars">
</a>

</div>
