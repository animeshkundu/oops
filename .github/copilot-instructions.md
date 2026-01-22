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

## Agentic Workflow Guidelines

### Working with AI Agents

When working with GitHub Copilot agents on tasks:

1. **Clear Task Scoping**
   - Define clear, focused issues with specific acceptance criteria
   - Include file hints and impacted areas
   - Specify what "done" means (tests pass, docs updated, etc.)
   - Avoid multi-topic or overly broad tasks

2. **Acceptance Criteria Template**
   ```markdown
   ## Acceptance Criteria
   - Implementation passes all affected tests
   - Code follows project style and conventions
   - Changes are minimal and focused
   - No modifications to unrelated files
   - Documentation updated if needed
   - Security checks pass (cargo clippy, no vulnerabilities)
   ```

3. **Incremental Work Pattern**
   - Plan first, then implement in small steps
   - Verify each step before moving to the next
   - Commit frequently with descriptive messages
   - Request reviews at logical checkpoints

4. **Agent Boundaries**
   - Agents should NOT modify core system files without explicit need
   - Agents should NOT skip quality checks (tests, clippy, fmt)
   - Agents should NOT commit directly to main/master
   - Agents MUST verify changes don't break existing functionality

### Task Suitability for AI Agents

**Well-suited tasks:**
- Bug fixes with clear reproduction steps
- Adding new rules following established patterns
- Writing tests for existing functionality
- Documentation updates and improvements
- Refactoring within well-defined boundaries
- Code style and formatting fixes

**Less suitable tasks:**
- Large architectural changes
- Core trait modifications
- Complex cross-cutting changes
- Security-critical modifications requiring deep context
- Changes requiring significant domain expertise

### Review and Iteration

- Treat agent output as a starting point, not final solution
- Always review generated code for:
  - Correctness and edge cases
  - Performance implications
  - Security considerations
  - Test coverage
  - Code style consistency
- Provide specific feedback and iterate
- Run all quality checks before merging

## Security Guidelines

1. **Dependency Management**
   - Always check dependencies for vulnerabilities before adding
   - Use `cargo audit` to scan for known security issues
   - Prefer well-maintained, popular crates
   - Document security decisions

2. **Code Safety**
   - All code must be memory-safe (Rust guarantees this mostly)
   - Avoid `unsafe` blocks unless absolutely necessary
   - Never execute arbitrary commands without validation
   - Don't modify system files or environment
   - Handle errors explicitly, never unwrap() in production code paths

3. **Rule Safety**
   - Rules must not have side effects by default
   - Use `side_effect()` sparingly and document why
   - Never make network calls from rules
   - Keep rule evaluation fast (<1ms) to avoid DoS
   - Validate all inputs before using in commands

4. **Testing for Security**
   - Test edge cases: empty input, special characters, very long input
   - Test for injection vulnerabilities in command generation
   - Verify rules don't match unintended commands
   - Test cross-platform behavior

## Code Quality Standards

### Rust-Specific

1. **Error Handling**
   - Use `Result<T>` for fallible operations with `anyhow::Result`
   - Use `?` operator for propagating errors
   - Provide context with `.context()` when errors occur
   - Never use `.unwrap()` or `.expect()` in library code

2. **Type Safety**
   - Prefer owned `String` in structs, `&str` in function parameters
   - Use `#[derive(Debug, Clone)]` on public structs
   - Implement traits explicitly when logic is non-trivial
   - Use type aliases for complex types to improve readability

3. **Documentation**
   - Document all public APIs with `///` doc comments
   - Include examples in doc comments for complex functions
   - Explain the "why" not just the "what"
   - Keep docs in sync with code

4. **Performance**
   - Prefer iterator chains over explicit loops where readable
   - Use `Cow` for conditional cloning
   - Cache expensive operations (already done for `which` lookups)
   - Profile before optimizing

### Testing Philosophy

1. **Test Coverage**
   - Every rule needs minimum 6 tests:
     - 2+ positive matches (different error patterns)
     - 2+ negative matches (shouldn't trigger)
     - 1+ correction verification
     - 1+ edge case (empty output, special chars)
   - Use real error output from actual tools
   - Test cross-platform behavior when relevant

2. **Test Structure**
   ```rust
   #[cfg(test)]
   mod tests {
       use super::*;
       use crate::core::Command;
   
       #[test]
       fn test_matches_when_error_occurs() {
           let cmd = Command::new("tool cmd", "error: something");
           let rule = MyRule::default();
           assert!(rule.is_match(&cmd));
       }
   
       #[test]
       fn test_no_match_when_success() {
           let cmd = Command::new("tool cmd", "success");
           let rule = MyRule::default();
           assert!(!rule.is_match(&cmd));
       }
   
       #[test]
       fn test_generates_correct_fix() {
           let cmd = Command::new("tool cmd", "error: typo");
           let rule = MyRule::default();
           let fixes = rule.get_new_command(&cmd);
           assert!(fixes.contains(&"tool command".to_string()));
       }
   }
   ```

3. **Test Data**
   - Use realistic error messages from actual tool runs
   - Test with different versions' error formats when known
   - Include unicode, spaces, and special characters
   - Test boundary conditions (empty, very long, null-like)

## Project Memory & Context

### Architecture Principles

1. **Rule System**
   - Rules are independent, stateless, and thread-safe
   - Rules are evaluated lazily for performance
   - Multiple rules can match; user selects from corrections
   - Priority determines order (lower = higher priority)

2. **Shell Integration**
   - Each shell implements the `Shell` trait
   - Shell-specific syntax (alias, history, chaining) isolated
   - Environment variables for backward compatibility with thefuck

3. **Performance First**
   - Target: <50ms cold start
   - Rules evaluate in microseconds
   - Lazy evaluation of rules
   - Efficient fuzzy matching with `strsim`

### Common Patterns

1. **Command Detection**
   ```rust
   use crate::utils::is_app;
   
   // Check if command uses specific tool
   is_app(cmd, &["git", "g"])  // Handles aliases too
   ```

2. **Fuzzy Matching**
   ```rust
   use crate::utils::get_close_matches;
   
   // Find similar commands
   let suggestions = get_close_matches("psh", &["push", "pull"], 3);
   ```

3. **Output Parsing**
   ```rust
   // Check for specific error patterns
   if cmd.output.contains("not a git command") {
       // Extract typo, suggest fix
   }
   ```

### Commit Message Guidelines

Follow conventional commit format:
- `feat(rules): add support for kubectl`
- `fix(git): handle branch names with spaces`
- `test(npm): add edge cases for package.json errors`
- `docs(readme): update installation instructions`
- `refactor(core): simplify rule matching logic`
- `perf(fuzzy): optimize string comparison`
- `chore(deps): update dependencies`

### PR Best Practices

1. **Title**: Clear, concise, follows commit convention
2. **Description**:
   - What problem does this solve?
   - What changes were made?
   - How was it tested?
   - Any breaking changes or migration needed?
3. **Scope**: Keep PRs focused (single rule, single fix)
4. **Tests**: Include test output showing coverage
5. **Examples**: Show before/after for rule corrections

## File Organization

```
src/
├── main.rs           # CLI entry point - minimal, just dispatch
├── lib.rs            # Public library interface
├── cli.rs            # clap-based argument parsing
├── config/           # Configuration loading (env, file, defaults)
├── core/             # Core types and rule engine (DO NOT MODIFY lightly)
│   ├── command.rs    # Command struct
│   ├── rule.rs       # Rule trait definition
│   ├── corrected.rs  # CorrectedCommand with side effects
│   └── corrector.rs  # Rule matching and filtering engine
├── rules/            # All correction rules (ADD NEW RULES HERE)
│   ├── mod.rs        # Rule registry
│   └── <category>/   # Rules grouped by tool category
├── shells/           # Shell-specific integrations
├── output/           # Command execution and output capture
├── ui/               # Terminal UI and user interaction
└── utils/            # Shared utilities (caching, fuzzy matching)
```

**Key principle**: New rules go in `src/rules/<category>/` and are registered in `src/rules/mod.rs`. Core files should rarely need changes.
