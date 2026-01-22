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
4. Add comprehensive tests (see [Test Pattern](#test-pattern-comprehensive) below)

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

## Agent Workflows & Personas

This project uses specialized agents for different tasks. When working on specific areas, prefer using the appropriate agent:

### Available Agents

All agents listed below are available in `.github/agents/`:

| Agent | Purpose | When to Use |
|-------|---------|-------------|
| **rust-expert** | Rust code implementation | Writing/refactoring Rust code, performance optimization |
| **test-specialist** | Testing & QA | Writing tests, improving coverage, test infrastructure |
| **rule-creator** | Single rule creation | Adding one specific correction rule with tests |
| **rule-expander** | Multi-rule research & implementation | Researching tools, adding multiple rules, gap analysis |
| **shell-integration** | Shell script work | Modifying bash/zsh/fish/PowerShell integration |
| **ci-cd-expert** | Build & release automation | GitHub Actions, cross-platform builds, releases |

### Agent Invocation Guidelines

**Rule Creation Workflows:**
- **Single rule**: Use `@rule-creator` agent - optimized for quick, focused rule additions
- **Research & multiple rules**: Use `@rule-expander` agent - handles tool research, multiple error patterns
- **Just tests**: Use `@test-specialist` agent - comprehensive test coverage expert

**Code Quality:**
- All Rust code changes → Review with **rust-expert** agent
- All test changes → Review with **test-specialist** agent  
- Shell integration → Use **shell-integration** agent exclusively

### When NOT to Use Agents

**Handle directly** (faster than agent invocation):
- Trivial typo fixes in comments or documentation
- Running standard commands: `cargo test`, `cargo fmt`, `cargo clippy`
- Reading a single known file
- Adding obvious missing test case to existing test file
- Simple one-line code changes with clear fix

**Use agent** when:
- Implementing new features or rules
- Refactoring multiple files
- Need domain expertise (Rust patterns, shell syntax, etc.)
- Research required (tool error patterns, best practices)
- Coordinated changes across project

## Boundaries & Safety

### ✅ Always Do
- Use `is_app(cmd, &["tool", "tool.exe"])` for cross-platform command detection
- Write minimum 4 tests per rule (match, no-match, correction, edge case)
- Run quality checks before committing: `cargo test && cargo clippy -- -D warnings && cargo fmt`
- Use real error messages from actual tools in tests
- Prefer `&str` over `String` in function parameters
- Handle edge cases: empty input, special characters, unicode
- Use descriptive test names: `test_<what>_<scenario>`

### ❌ Never Do
- Modify `src/core/` without explicit requirement (use specialized agents)
- Change the `Rule` trait definition
- Commit secrets or credentials
- Add dependencies without security review
- Skip tests or ignore clippy warnings
- Use `unwrap()` or `expect()` in production code
- Hard-code platform-specific paths or commands
- Copy Python code from thefuck (translate concepts instead)

### Security Guidelines
- Never execute arbitrary commands without validation
- Validate all user input and command output
- Use `anyhow::Result` for error handling
- No network calls in rules (performance requirement: <50ms startup)
- Test for command injection vulnerabilities
- Don't trust command output format (handle variations)

### Cross-Platform Requirements
**Must test on Windows, macOS, and Linux before PR:**
- Use `is_app(cmd, &["tool", "tool.exe"])` for command detection
- Handle both `\n` and `\r\n` line endings (use `.lines()` iterator)
- Use `std::path::PathBuf` for file paths, not string concatenation
- Test common scenarios:
  - Windows: cmd.exe and PowerShell
  - macOS: bash and zsh  
  - Linux: bash, zsh, and fish

**GitHub Actions automatically tests all platforms in CI**

### Adding Dependencies
**Before adding a new crate:**
1. Check if functionality exists in std library or existing dependencies
2. Verify crate is actively maintained (commits in last 6 months)
3. Review security advisories: `cargo audit`
4. Check popularity (prefer >100k downloads)
5. Add with specific version in `Cargo.toml`: `crate = "1.2.3"`

**Prefer:**
- Well-established crates (serde, regex, clap, etc.)
- Minimal transitive dependencies
- Pure Rust (avoid C bindings unless necessary)

**Discuss with maintainers** before adding:
- New major dependencies
- Dependencies with many transitive deps
- Pre-1.0 crates

## Code Review Standards

### Before Submitting PR
1. **Tests**: All tests pass (`cargo test`)
2. **Linting**: No clippy warnings (`cargo clippy -- -D warnings`)
3. **Formatting**: Code formatted (`cargo fmt --check`)
4. **Coverage**: New code has tests (aim for 80%+)
5. **Documentation**: Public APIs have doc comments
6. **Performance**: No regressions (run `cargo bench` if touching hot paths)

### PR Description Template
```markdown
## What
Brief description of changes

## Why
Problem being solved or feature being added

## How
Technical approach and key changes

## Testing
- [ ] Unit tests added/updated
- [ ] Integration tests pass
- [ ] Manual testing done (describe)
- [ ] Cross-platform tested (Windows/macOS/Linux)

## Checklist
- [ ] Tests pass
- [ ] Clippy clean
- [ ] Formatted
- [ ] Documentation updated
```

## Common Patterns & Examples

### Rule Implementation Pattern
```rust
use crate::core::{is_app, Command, Rule};
use crate::utils::get_close_matches;

#[derive(Debug, Clone, Copy, Default)]
pub struct MyRule;

impl Rule for MyRule {
    fn name(&self) -> &str {
        "my_rule"
    }

    fn is_match(&self, cmd: &Command) -> bool {
        // ALWAYS check tool first (performance & correctness)
        is_app(cmd, &["mytool", "mytool.exe"]) 
            && cmd.output.to_lowercase().contains("specific error pattern")
    }

    fn get_new_command(&self, cmd: &Command) -> Vec<String> {
        // Use fuzzy matching for typo correction
        let parts = cmd.script_parts();
        if parts.len() < 2 {
            return vec![];
        }
        
        let typo = &parts[1];
        let valid_commands = vec!["correct1", "correct2"];
        
        get_close_matches(typo, &valid_commands, 3, 0.6)
            .into_iter()
            .map(|fix| cmd.script.replace(typo, &fix))
            .collect()
    }

    fn priority(&self) -> i32 {
        1000  // Lower = higher priority (100-500: critical, 1000: default, 1001+: fallback)
    }
}
```

### Test Pattern (Comprehensive)
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::Command;

    #[test]
    fn test_matches_expected_error() {
        // Use REAL error output from actual tool
        let cmd = Command::new(
            "mytool badcmd",
            "error: unknown command 'badcmd'\nDid you mean 'goodcmd'?"
        );
        let rule = MyRule;
        assert!(rule.is_match(&cmd));
    }

    #[test]
    fn test_not_matches_success() {
        let cmd = Command::new("mytool goodcmd", "Success");
        let rule = MyRule;
        assert!(!rule.is_match(&cmd));
    }

    #[test]
    fn test_not_matches_different_tool() {
        let cmd = Command::new("othertool badcmd", "error: unknown command");
        let rule = MyRule;
        assert!(!rule.is_match(&cmd));
    }

    #[test]
    fn test_generates_correction() {
        let cmd = Command::new("mytool badcmd", "error: unknown command");
        let rule = MyRule;
        let fixes = rule.get_new_command(&cmd);
        assert!(fixes.contains(&"mytool goodcmd".to_string()));
    }

    #[test]
    fn test_preserves_arguments() {
        let cmd = Command::new("mytool badcmd --flag", "error");
        let rule = MyRule;
        let fixes = rule.get_new_command(&cmd);
        if let Some(first_fix) = fixes.first() {
            assert!(first_fix.contains("--flag"));
        }
    }

    #[test]
    fn test_empty_command_edge_case() {
        let cmd = Command::new("", "");
        let rule = MyRule;
        assert!(!rule.is_match(&cmd));
    }
}
```

### Error Handling Pattern
```rust
use anyhow::{Context, Result, bail, ensure};

pub fn load_config() -> Result<Config> {
    let path = config_path()
        .context("failed to determine config path")?;
    
    let contents = std::fs::read_to_string(&path)
        .with_context(|| format!("failed to read {}", path.display()))?;
    
    toml::from_str(&contents)
        .context("failed to parse config TOML")
}

pub fn validate_input(name: &str) -> Result<()> {
    ensure!(!name.is_empty(), "name cannot be empty");
    ensure!(
        name.chars().all(|c| c.is_alphanumeric() || c == '_'),
        "name must be alphanumeric with underscores only"
    );
    Ok(())
}
```

## Performance Optimization Tips

### Do's
- Pre-compile regex with `once_cell::sync::Lazy`
- Use iterators over collecting intermediate vectors
- Cache expensive operations with `#[cached]` macro
- Use string slicing (`&str`) over allocating (`String`)
- Early return from `is_match()` if tool doesn't match

### Don'ts
- Don't recompile regex on every call
- Don't clone strings unnecessarily
- Don't use `.to_string()` when `.as_str()` works
- Don't allocate in hot paths (rule matching)

### Example: Optimized Regex
```rust
use once_cell::sync::Lazy;
use regex::Regex;

static ERROR_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"error: '([^']+)' not found").unwrap()
});

fn extract_missing(&self, output: &str) -> Option<&str> {
    ERROR_RE.captures(output)?.get(1).map(|m| m.as_str())
}
```

## Troubleshooting & Debugging

### Common Issues

**Tests failing with "command not found":**
- Tests use real error outputs, not live command execution
- Use verbatim error strings from the tool's output

**Rule not triggering:**
- Check `is_app()` is checking correct tool name
- Verify error pattern matching is case-insensitive
- Ensure `requires_output()` returns `true` if matching on output

**Clippy warnings:**
- `needless_borrow`: Remove `&` when not needed
- `redundant_clone`: Use references instead of cloning
- `missing_docs`: Add `///` doc comments to public items

### Debug Logging
```rust
use tracing::{debug, info, warn, error};

fn is_match(&self, cmd: &Command) -> bool {
    debug!("Checking rule {} against command: {}", self.name(), cmd.script);
    
    if !is_app(cmd, &["tool"]) {
        debug!("Not the right tool");
        return false;
    }
    
    let matches = cmd.output.contains("error");
    debug!("Match result: {}", matches);
    matches
}
```

Enable with: `RUST_LOG=debug cargo run`

## Git Workflow

### Branch Naming
- `feature/add-TOOL-support` - New tool support
- `fix/RULE-bug` - Bug fix in existing rule
- `refactor/MODULE` - Code refactoring
- `docs/SECTION` - Documentation updates
- `test/improve-coverage` - Test improvements

### Commit Message Format
Follow Conventional Commits:
```
feat(rules): add kubectl corrections for unknown resources
fix(git): handle git 2.40 error message format change
test(cargo): add edge cases for cargo build failures
docs(readme): update installation instructions
refactor(core): optimize rule matching performance
```

### Before Pushing
```bash
# Run full quality check
cargo test && cargo clippy -- -D warnings && cargo fmt

# Check diff
git diff

# Commit with descriptive message
git commit -m "feat(rules): add docker-compose corrections"

# Push to feature branch
git push -u origin feature/add-docker-compose-support
```

## Resources & Documentation

**Internal Documentation:**
- `CLAUDE.md` - Detailed architecture and conventions
- `CONTRIBUTING.md` - Contribution guidelines
- `README.md` - Project overview and usage
- `docs/` - Additional documentation

**Code References:**
- Well-tested rule example: `src/rules/git/not_command.rs`
- Core trait definitions: `src/core/rule.rs`
- Test helpers: `tests/common/mod.rs`

**External Resources:**
- Rust Book: https://doc.rust-lang.org/book/
- Rust by Example: https://doc.rust-lang.org/rust-by-example/
- Clippy Lints: https://rust-lang.github.io/rust-clippy/
- thefuck (inspiration): https://github.com/nvbn/thefuck

## Quick Command Reference

```bash
# Development cycle
cargo check              # Fast syntax check
cargo test               # Run all tests
cargo test my_rule       # Run specific test
cargo clippy             # Linting
cargo fmt                # Format code

# Testing variants
cargo test -- --nocapture       # Show println! output
cargo test -- --test-threads=1  # Run sequentially
cargo test --lib                # Unit tests only
cargo test --test integration   # Specific integration test

# Building
cargo build              # Debug build (fast)
cargo build --release    # Optimized build (slow, for benchmarking)

# Benchmarking
cargo bench              # Run all benchmarks
cargo bench rule_match   # Specific benchmark

# Documentation
cargo doc --open         # Generate and view docs

# Cleanup
cargo clean              # Remove build artifacts
```

## Remember

**Quality over speed**: Well-tested, idiomatic Rust is better than quick hacks  
**Performance matters**: This tool must start in <50ms  
**Cross-platform first**: Test on Windows, macOS, and Linux  
**Real errors only**: Use actual tool output in tests, not invented strings  
**Agents are your friends**: Use specialized agents for their expertise areas
