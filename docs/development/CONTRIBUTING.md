# Contributing to oops

Thank you for your interest in contributing to oops! This document provides guidelines and instructions for contributing.

## Code of Conduct

Please be respectful and constructive in all interactions. We welcome contributors of all experience levels.

## Getting Started

### Prerequisites

- Rust toolchain (stable, 1.88+)
- Git

### Development Setup

1. Clone the repository:
   ```bash
   git clone https://github.com/animeshkundu/oops.git
   cd oops
   ```

2. Build the project:
   ```bash
   cargo build
   ```

3. Run tests:
   ```bash
   cargo test
   ```

4. Check formatting and lints:
   ```bash
   cargo fmt --check
   cargo clippy
   ```

## Making Changes

### Branch Naming

- `feature/description` - New features
- `fix/description` - Bug fixes
- `docs/description` - Documentation changes
- `refactor/description` - Code refactoring

### Commit Messages

Follow conventional commits format:

```
type(scope): description

[optional body]

[optional footer]
```

Types: `feat`, `fix`, `docs`, `style`, `refactor`, `test`, `chore`

Examples:
- `feat(rules): add kubectl rules`
- `fix(git): handle detached HEAD state`
- `docs: update installation instructions`
- `chore: update dependencies`

**Important for PR titles**: All merged PRs trigger a **minor version bump** (e.g., 0.1.0 → 0.2.0):
- Use clear, descriptive PR titles following conventional commit format
- Add `[skip release]` to PR title to prevent automatic release
- For major or patch bumps, manual releases can be created

### Pull Request Process

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Ensure all tests pass
5. Submit a pull request

## Adding a New Rule

Rules are the core of oops. Here's how to add a new one:

### 1. Create the Rule Struct

Create a new file or add to an existing module in `src/rules/`:

```rust
use crate::core::{Command, Rule};

/// Fixes [describe what this rule fixes]
pub struct MyNewRule;

impl Rule for MyNewRule {
    fn name(&self) -> &str {
        "my_new_rule"
    }

    fn is_match(&self, cmd: &Command) -> bool {
        // Return true if this rule applies to the command
        cmd.output.contains("specific error pattern")
    }

    fn get_new_command(&self, cmd: &Command) -> Vec<String> {
        // Return the corrected command(s)
        vec![format!("corrected {}", cmd.script)]
    }

    // Optional: Override defaults
    fn priority(&self) -> i32 {
        1000 // Lower = higher priority
    }

    fn enabled_by_default(&self) -> bool {
        true
    }

    fn requires_output(&self) -> bool {
        true // Set false if rule doesn't need command output
    }
}
```

### 2. Register the Rule

Add your rule to `src/rules/mod.rs`:

```rust
rules.push(Box::new(my_module::MyNewRule));
```

### 3. Write Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_my_new_rule_matches() {
        let cmd = Command::new("original command", "error output");
        let rule = MyNewRule;
        assert!(rule.is_match(&cmd));
    }

    #[test]
    fn test_my_new_rule_correction() {
        let cmd = Command::new("original command", "error output");
        let rule = MyNewRule;
        let corrections = rule.get_new_command(&cmd);
        assert_eq!(corrections[0], "expected correction");
    }

    #[test]
    fn test_my_new_rule_no_match() {
        let cmd = Command::new("unrelated command", "different output");
        let rule = MyNewRule;
        assert!(!rule.is_match(&cmd));
    }
}
```

### Rule Guidelines

1. **Be specific**: Rules should match precisely, not broadly
2. **Test thoroughly**: Include both positive and negative test cases
3. **Document**: Add doc comments explaining what the rule does
4. **Consider edge cases**: Handle unusual inputs gracefully
5. **Keep it simple**: One rule, one purpose

## Project Structure

```
oops/
├── src/
│   ├── main.rs          # Entry point
│   ├── lib.rs           # Library root
│   ├── cli.rs           # CLI argument parsing
│   ├── config/          # Configuration system
│   ├── core/            # Core types (Command, Rule, etc.)
│   ├── rules/           # All correction rules
│   │   ├── git/         # Git-related rules
│   │   ├── package_managers/
│   │   └── ...
│   ├── shells/          # Shell integrations
│   ├── output/          # Command execution
│   ├── ui/              # Terminal UI
│   └── utils/           # Utilities
├── tests/               # Integration tests
├── benches/             # Benchmarks
└── docs/                # Documentation
```

## Testing

### Unit Tests

```bash
cargo test
```

### Integration Tests

```bash
cargo test --test cli_tests
```

### Benchmarks

```bash
cargo bench
```

## Code Style

- Run `cargo fmt` before committing
- Run `cargo clippy` and address warnings
- Follow Rust naming conventions
- Write documentation for public APIs
- Keep functions focused and small

## Release Process

Releases are fully automated via GitHub Actions when a PR is merged to master.

**For detailed information**, see [Auto-Release Workflow Documentation](docs/auto-release-workflow.md).

### Automatic Release (Recommended)

1. Create a PR with your changes
2. Use a clear, descriptive PR title (conventional commit format recommended):
   - **Examples**: `feat: add new rules`, `fix: resolve crash`, `docs: update guide`
   - **All PRs trigger minor bumps**: 0.1.0 → 0.2.0 → 0.3.0
   - **Skip release**: Add `[skip release]` to title
3. Merge the PR
4. The `auto-release` workflow will:
   - Run tests on Linux, macOS, and Windows
   - Automatically bump the version in `Cargo.toml` (minor bump)
   - Create and push a git tag (e.g., `v0.1.1`)
5. The `release` workflow (triggered by tag) will:
   - Build binaries for 6 platforms (Linux x86_64/ARM64, macOS x86_64/ARM64, Windows x86_64)
   - Generate SHA256 checksums
   - Create a GitHub release with all artifacts

**To skip automatic release**: Include `[skip release]` or `[no release]` in your PR title.

### Manual Release

If needed, you can manually trigger a release:

1. Bump version: `cargo set-version --bump patch` (or minor/major)
2. Commit: `git add Cargo.toml Cargo.lock && git commit -m "chore: bump version to X.Y.Z"`
3. Tag and push: `git tag -a vX.Y.Z -m "Release vX.Y.Z" && git push origin main --tags`
4. The release workflow will build and publish automatically

## Getting Help

- Open an issue for bugs or feature requests
- Start a discussion for questions
- Check existing issues before creating new ones

## License

By contributing, you agree that your contributions will be licensed under the MIT License.
