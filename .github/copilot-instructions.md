# oops - Copilot Instructions

## Project Overview

oops is a blazingly fast command-line typo corrector written in Rust. It fixes
your previous console command when you make a mistake. Inspired by thefuck but
optimized for performance (<50ms startup vs ~300ms for Python).

## Architecture

```
CLI (clap) -> Config -> Shell Detection -> Command Capture -> Rule Matching -> UI -> Execution
```

### Directory Structure

- `src/main.rs` - Entry point, CLI dispatch
- `src/cli.rs` - Argument parsing with clap derive
- `src/config/` - Settings struct, config file/env loading
- `src/core/` - Command, Rule trait, CorrectedCommand, Corrector engine
- `src/rules/` - 175+ correction rules organized by category
- `src/shells/` - Bash, Zsh, Fish, PowerShell, Tcsh integrations
- `src/output/` - Command execution and output capture
- `src/ui/` - Terminal UI, colors, interactive selector
- `src/utils/` - Caching, fuzzy matching, executable lookup

## Key Types

### Command (src/core/command.rs)
```rust
pub struct Command {
    pub script: String,  // The command that was run
    pub output: String,  // stderr + stdout combined
}
```

### Rule Trait (src/core/rule.rs)
```rust
pub trait Rule: Send + Sync {
    fn name(&self) -> &str;
    fn is_match(&self, cmd: &Command) -> bool;
    fn get_new_command(&self, cmd: &Command) -> Vec<String>;
    fn priority(&self) -> i32 { 1000 }  // Lower = higher priority
    fn enabled_by_default(&self) -> bool { true }
    fn requires_output(&self) -> bool { true }
    fn side_effect(&self, _: &Command, _: &str) -> Result<()> { Ok(()) }
}
```

### CorrectedCommand (src/core/corrected.rs)
```rust
pub struct CorrectedCommand {
    pub script: String,
    pub priority: i32,
    pub side_effect: Option<Box<dyn Fn(&Command, &str) -> Result<()>>>,
}
```

## Coding Conventions

1. **Error handling**: Use `Result<T>` with `anyhow` for fallible operations
2. **String parameters**: Prefer `&str` over `String` in function parameters
3. **Struct derives**: Always use `#[derive(Debug, Clone)]` on structs
4. **Testing**: Write tests for every rule using the pattern below
5. **Logging**: Use `tracing` macros (debug!, info!, warn!, error!)
6. **Formatting**: Run `cargo fmt` before committing
7. **Linting**: Run `cargo clippy -- -D warnings`

## Creating a New Rule

1. Create struct implementing `Rule` trait
2. Place in appropriate category under `src/rules/`
3. Register in `src/rules/mod.rs` via `get_all_rules()`
4. Add comprehensive tests

### Rule Test Pattern
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::Command;

    #[test]
    fn test_rule_matches() {
        let cmd = Command::new("git psuh", "git: 'psuh' is not a git command");
        let rule = MyRule::new();
        assert!(rule.is_match(&cmd));
    }

    #[test]
    fn test_rule_correction() {
        let cmd = Command::new("git psuh", "git: 'psuh' is not a git command");
        let rule = MyRule::new();
        let corrections = rule.get_new_command(&cmd);
        assert!(corrections.contains(&"git push".to_string()));
    }

    #[test]
    fn test_rule_not_matches() {
        let cmd = Command::new("git push", "Everything up-to-date");
        let rule = MyRule::new();
        assert!(!rule.is_match(&cmd));
    }
}
```

## Build Commands

```bash
cargo build            # Debug build
cargo build --release  # Release build (LTO enabled)
cargo test             # Run all tests
cargo fmt --check      # Check formatting
cargo clippy           # Run linter
cargo bench            # Run benchmarks
cargo run -- --alias   # Generate shell alias
cargo run -- --version # Show version
```

## Environment Variables (thefuck compatible)

| Variable | Description |
|----------|-------------|
| `TF_SHELL` | Current shell (bash, zsh, fish, powershell, tcsh) |
| `TF_ALIAS` | Alias name (default: oops) |
| `TF_HISTORY` | Recent command history |
| `THEFUCK_RULES` | Enabled rules (colon-separated) |
| `THEFUCK_EXCLUDE_RULES` | Disabled rules |
| `THEFUCK_REQUIRE_CONFIRMATION` | true/false |
| `THEFUCK_WAIT_COMMAND` | Timeout in seconds |
| `THEFUCK_DEBUG` | Enable debug output |

## Dependencies

- `clap` - CLI argument parsing with derive
- `serde` + `toml` - Configuration
- `regex` + `fancy-regex` - Pattern matching
- `strsim` - Fuzzy string matching (Levenshtein)
- `crossterm` - Terminal manipulation
- `dirs` - XDG directory paths
- `anyhow` + `thiserror` - Error handling
- `tracing` - Structured logging
- `which` - Executable lookup

## Performance Guidelines

- Target startup time: <50ms
- Rules are evaluated lazily
- Use `strsim` for fuzzy matching (already optimized)
- Cache `which` lookups using `cached` crate
- LTO and single codegen unit enabled in release

## Shell Integration

Each shell in `src/shells/` implements the `Shell` trait:
- `app_alias()` - Generate the shell alias/function
- `get_history()` - Retrieve command history
- `get_aliases()` - Parse existing shell aliases
- `put_to_history()` - Add command to history
- `and_()` / `or_()` - Command chaining syntax

## Rule Categories

Rules are organized by category in `src/rules/`:
- `git/` - Git operations (push, checkout, add, branch, etc.)
- `package_managers/` - apt, brew, cargo, npm, pip, etc.
- `system.rs` - File operations, permissions
- `cloud.rs` - AWS, Azure, Heroku
- `devtools.rs` - Go, Java, Maven, Gradle
- `frameworks.rs` - Python, Rails, React Native
- `shell_utils.rs` - grep, sed, history
- `misc.rs` - Other utilities
