# CLAUDE.md - oops

## Project Overview

oops is a blazingly fast command-line typo corrector written in Rust. Inspired by [thefuck](https://github.com/nvbn/thefuck), it's designed for performance and ease of distribution. Configuration environment variables are backward compatible.

## Build Commands

```bash
# Build debug
cargo build

# Build release
cargo build --release

# Run tests
cargo test

# Run with arguments
cargo run -- --version
cargo run -- --alias

# Check formatting
cargo fmt --check

# Run clippy
cargo clippy

# Run benchmarks
cargo bench
```

## Project Structure

```
oops/
├── src/
│   ├── main.rs          # Entry point, CLI dispatch
│   ├── lib.rs           # Library root
│   ├── cli.rs           # Argument parsing (clap)
│   ├── config/          # Configuration system
│   │   ├── mod.rs       # Module root
│   │   ├── settings.rs  # Settings struct
│   │   └── loader.rs    # Config loading
│   ├── core/            # Core types
│   │   ├── mod.rs       # fix_command, get_corrected_commands
│   │   ├── command.rs   # Command struct
│   │   ├── rule.rs      # Rule trait
│   │   ├── corrected.rs # CorrectedCommand struct
│   │   └── corrector.rs # Rule matching engine
│   ├── rules/           # All correction rules
│   │   ├── mod.rs       # Rule registry
│   │   ├── git/         # 49 git rules
│   │   ├── package_managers/ # 27 pkg manager rules
│   │   ├── system.rs    # 18 system rules
│   │   └── ...
│   ├── shells/          # Shell integrations
│   │   ├── mod.rs       # Shell trait, detection
│   │   ├── bash.rs
│   │   ├── zsh.rs
│   │   ├── fish.rs
│   │   ├── powershell.rs
│   │   └── tcsh.rs
│   ├── output/          # Command execution
│   │   └── rerun.rs     # Execute and capture
│   ├── ui/              # Terminal UI
│   │   ├── colors.rs
│   │   └── selector.rs
│   └── utils/           # Utilities
│       ├── cache.rs
│       ├── executables.rs
│       └── fuzzy.rs
├── tests/               # Integration tests
├── benches/             # Performance benchmarks
└── docs/                # Documentation
```

## Key Types

### Command
```rust
pub struct Command {
    pub script: String,      // The command that was run
    pub output: String,      // stderr + stdout combined
}
```

### Rule Trait
```rust
pub trait Rule: Send + Sync {
    fn name(&self) -> &str;
    fn is_match(&self, cmd: &Command) -> bool;
    fn get_new_command(&self, cmd: &Command) -> Vec<String>;
    fn priority(&self) -> i32 { 1000 }           // Lower = higher priority
    fn enabled_by_default(&self) -> bool { true }
    fn requires_output(&self) -> bool { true }
    fn side_effect(&self, _: &Command, _: &str) -> Result<()> { Ok(()) }
}
```

### CorrectedCommand
```rust
pub struct CorrectedCommand {
    pub script: String,
    pub priority: i32,
    pub side_effect: Option<Box<dyn Fn(&Command, &str) -> Result<()>>>,
}
```

## Environment Variables

| Variable | Description |
|----------|-------------|
| `TF_SHELL` | Current shell (bash, zsh, fish, powershell, tcsh) |
| `TF_ALIAS` | Alias name (default: oops) |
| `TF_HISTORY` | Recent command history |
| `TF_SHELL_ALIASES` | Shell alias definitions |
| `THEFUCK_RULES` | Enabled rules (colon-separated) |
| `THEFUCK_EXCLUDE_RULES` | Disabled rules |
| `THEFUCK_REQUIRE_CONFIRMATION` | true/false |
| `THEFUCK_WAIT_COMMAND` | Timeout in seconds |
| `THEFUCK_DEBUG` | Enable debug output |

## Testing a Rule

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::Command;

    #[test]
    fn test_rule_matches() {
        let cmd = Command::new("git psuh", "git: 'psuh' is not a git command");
        let rule = GitNotCommand::new();
        assert!(rule.is_match(&cmd));
    }

    #[test]
    fn test_rule_correction() {
        let cmd = Command::new("git psuh", "git: 'psuh' is not a git command");
        let rule = GitNotCommand::new();
        let corrections = rule.get_new_command(&cmd);
        assert!(corrections.contains(&"git push".to_string()));
    }
}
```

## Dependencies

- `clap` - CLI argument parsing
- `serde` + `toml` - Configuration
- `regex` + `fancy-regex` - Pattern matching
- `strsim` - Fuzzy string matching
- `crossterm` - Terminal manipulation
- `dirs` - XDG directory paths
- `anyhow` + `thiserror` - Error handling
- `tracing` - Logging
- `which` - Executable lookup

## Coding Guidelines

1. Use `Result<T>` for fallible operations
2. Prefer `&str` over `String` in function parameters
3. Use `#[derive(Debug, Clone)]` on structs
4. Write tests for every rule
5. Keep rules in separate files by category
6. Use `tracing` for debug logging
7. Run `cargo fmt` and `cargo clippy` before committing

## Rule Organization

Rules are organized by category:
- `git/` - Git operations (push, checkout, add, branch, etc.)
- `package_managers/` - apt, brew, cargo, npm, pip, etc.
- `system.rs` - File operations, permissions
- `cloud.rs` - AWS, Azure, Heroku
- `devtools.rs` - Go, Java, Maven, Gradle
- `frameworks.rs` - Python, Rails, React Native
- `shell_utils.rs` - grep, sed, history
- `misc.rs` - Everything else

## Parity Checking

Check which thefuck rules are missing from oops:

```bash
# Check parity with default 7-day activity window
cargo run --bin check_parity

# Check with custom time window
cargo run --bin check_parity -- --days 30

# Get JSON output for automation
cargo run --bin check_parity -- --output json

# Or use the convenience script
./scripts/check-parity.sh
```

The parity checker:
- Scans `src/rules/` to find all oops rules
- Compares against known thefuck rules
- Reports missing rules and coverage percentage
- Can detect recently updated rules if a local thefuck clone is available

## Performance Notes

- Startup target: <50ms
- Rules are evaluated lazily
- Fuzzy matching uses strsim for efficiency
- Command output is captured once and shared
