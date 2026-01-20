# Creating Rules Guide

This guide explains how to add new correction rules to oops.

## Rule Basics

A rule is a Rust struct that implements the `Rule` trait:

```rust
use crate::core::{Command, Rule};
use anyhow::Result;

pub struct MyRule;

impl Rule for MyRule {
    fn name(&self) -> &str {
        "my_rule"
    }

    fn is_match(&self, cmd: &Command) -> bool {
        // Return true if this rule applies
        cmd.output.contains("some error")
    }

    fn get_new_command(&self, cmd: &Command) -> Vec<String> {
        // Return corrected command(s)
        vec![format!("fixed {}", cmd.script)]
    }
}
```

## Step-by-Step Guide

### 1. Choose the Right Module

Rules are organized by category in `src/rules/`:

- `git/` - Git commands
- `package_managers/` - apt, brew, npm, etc.
- `system.rs` - File operations
- `cloud.rs` - AWS, Azure, etc.
- `devtools.rs` - Go, Java, etc.
- `misc.rs` - Everything else

### 2. Create the Rule

Add to the appropriate file or create a new one:

```rust
// src/rules/misc.rs

/// Fixes the common typo of typing 'sl' instead of 'ls'
pub struct SlLs;

impl Rule for SlLs {
    fn name(&self) -> &str {
        "sl_ls"
    }

    fn is_match(&self, cmd: &Command) -> bool {
        cmd.script.starts_with("sl ")
            || cmd.script == "sl"
    }

    fn get_new_command(&self, cmd: &Command) -> Vec<String> {
        vec![cmd.script.replacen("sl", "ls", 1)]
    }

    // This rule doesn't need command output
    fn requires_output(&self) -> bool {
        false
    }
}
```

### 3. Register the Rule

Add to `src/rules/mod.rs`:

```rust
pub fn get_all_rules() -> Vec<Box<dyn Rule>> {
    vec![
        // ... existing rules ...
        Box::new(misc::SlLs),
    ]
}
```

### 4. Write Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::Command;

    #[test]
    fn test_sl_ls_matches() {
        let rule = SlLs;

        // Should match
        assert!(rule.is_match(&Command::new("sl", "")));
        assert!(rule.is_match(&Command::new("sl -la", "")));

        // Should not match
        assert!(!rule.is_match(&Command::new("ls", "")));
        assert!(!rule.is_match(&Command::new("sleep 5", "")));
    }

    #[test]
    fn test_sl_ls_correction() {
        let rule = SlLs;
        let cmd = Command::new("sl -la", "");
        let fixes = rule.get_new_command(&cmd);

        assert_eq!(fixes, vec!["ls -la"]);
    }
}
```

### 5. Build and Test

```bash
cargo build
cargo test
```

## Rule Trait Methods

### Required Methods

```rust
fn name(&self) -> &str;
fn is_match(&self, cmd: &Command) -> bool;
fn get_new_command(&self, cmd: &Command) -> Vec<String>;
```

### Optional Methods

```rust
// Priority (lower = higher priority, default: 1000)
fn priority(&self) -> i32 { 1000 }

// Whether rule is enabled by default
fn enabled_by_default(&self) -> bool { true }

// Whether rule needs command output
fn requires_output(&self) -> bool { true }

// Side effect after correction runs
fn side_effect(&self, old_cmd: &Command, new_script: &str) -> Result<()> {
    Ok(())
}
```

## Common Patterns

### Pattern Matching with Regex

```rust
use regex::Regex;
use once_cell::sync::Lazy;

static ERROR_PATTERN: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"command not found: (\w+)").unwrap()
});

impl Rule for MyRule {
    fn is_match(&self, cmd: &Command) -> bool {
        ERROR_PATTERN.is_match(&cmd.output)
    }

    fn get_new_command(&self, cmd: &Command) -> Vec<String> {
        if let Some(caps) = ERROR_PATTERN.captures(&cmd.output) {
            let typo = &caps[1];
            // ... generate correction
        }
        vec![]
    }
}
```

### Fuzzy Matching

```rust
use crate::utils::fuzzy::get_close_matches;

impl Rule for MyRule {
    fn get_new_command(&self, cmd: &Command) -> Vec<String> {
        let typo = extract_typo(&cmd.script);
        let valid_commands = get_valid_commands();

        get_close_matches(&typo, &valid_commands, 3, 0.6)
            .into_iter()
            .map(|match_| cmd.script.replace(&typo, &match_))
            .collect()
    }
}
```

### Application-Specific Rules

```rust
use crate::rules::is_app;

impl Rule for GitPush {
    fn is_match(&self, cmd: &Command) -> bool {
        is_app(cmd, &["git"])
            && cmd.script.contains("push")
            && cmd.output.contains("no upstream")
    }
}
```

### Multiple Suggestions

```rust
impl Rule for MyRule {
    fn get_new_command(&self, cmd: &Command) -> Vec<String> {
        vec![
            format!("option1 {}", cmd.script),
            format!("option2 {}", cmd.script),
            format!("option3 {}", cmd.script),
        ]
    }
}
```

## Best Practices

### 1. Be Specific

Match precisely to avoid false positives:

```rust
// Good - specific pattern
fn is_match(&self, cmd: &Command) -> bool {
    cmd.output.contains("Permission denied")
        && !cmd.script.starts_with("sudo")
}

// Bad - too broad
fn is_match(&self, cmd: &Command) -> bool {
    cmd.output.contains("denied")
}
```

### 2. Handle Edge Cases

```rust
fn get_new_command(&self, cmd: &Command) -> Vec<String> {
    let parts = cmd.script_parts();

    // Check for empty or single-element commands
    if parts.len() < 2 {
        return vec![];
    }

    // ... rest of logic
}
```

### 3. Use Helper Functions

```rust
// Extract common logic
fn extract_file_path(output: &str) -> Option<&str> {
    // ...
}

impl Rule for MyRule {
    fn is_match(&self, cmd: &Command) -> bool {
        extract_file_path(&cmd.output).is_some()
    }
}
```

### 4. Document the Rule

```rust
/// Fixes permission denied errors by prepending sudo.
///
/// # Examples
///
/// ```text
/// $ apt install vim
/// E: Could not open lock file - Permission denied
///
/// $ oops
/// sudo apt install vim
/// ```
pub struct Sudo;
```

### 5. Consider Priority

```rust
impl Rule for MyRule {
    fn priority(&self) -> i32 {
        // High priority (runs early): 1-99
        // Normal priority: 1000 (default)
        // Low priority (runs late): 2000+
        100
    }
}
```

## Debugging Rules

### Enable Debug Mode

```bash
oops --debug
```

### Add Tracing

```rust
use tracing::debug;

impl Rule for MyRule {
    fn is_match(&self, cmd: &Command) -> bool {
        debug!("Checking MyRule for: {}", cmd.script);
        let result = /* ... */;
        debug!("MyRule match result: {}", result);
        result
    }
}
```

## Submitting Your Rule

1. Fork the repository
2. Create a branch: `git checkout -b feature/my-new-rule`
3. Add your rule with tests
4. Run `cargo fmt` and `cargo clippy`
5. Submit a pull request

Include in your PR:
- Description of what the rule fixes
- Example of the error and correction
- Any edge cases considered
