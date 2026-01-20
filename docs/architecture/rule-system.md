# Rule System Architecture

The rule system is the core of oops. It determines how commands are matched and corrected.

## Rule Trait

Every rule implements the `Rule` trait:

```rust
pub trait Rule: Send + Sync {
    /// Unique identifier for this rule
    fn name(&self) -> &str;

    /// Check if this rule applies to the command
    fn is_match(&self, cmd: &Command) -> bool;

    /// Generate corrected command(s)
    fn get_new_command(&self, cmd: &Command) -> Vec<String>;

    /// Priority (lower = higher priority, default: 1000)
    fn priority(&self) -> i32 { 1000 }

    /// Whether this rule is enabled by default
    fn enabled_by_default(&self) -> bool { true }

    /// Whether this rule needs command output to match
    fn requires_output(&self) -> bool { true }

    /// Optional side effect after correction runs
    fn side_effect(&self, _: &Command, _: &str) -> Result<()> { Ok(()) }
}
```

## Rule Evaluation Flow

```
┌─────────────────┐
│ Failed Command  │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│ Filter by       │  rules enabled in config
│ Configuration   │  exclude_rules setting
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│ Check           │  rule.requires_output()
│ Output Needs    │  skip if output empty and required
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│ Match Rules     │  rule.is_match(&cmd)
│ (in priority    │
│  order)         │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│ Generate        │  rule.get_new_command(&cmd)
│ Corrections     │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│ Sort & Dedup    │  by priority, remove duplicates
│ Results         │
└─────────────────┘
```

## Rule Categories

### High Priority Rules (< 100)

Fast, simple fixes that should run first:

- **Sudo** (priority: 50) - Add `sudo` prefix for permission errors
- **CD rules** (priority: 100) - Directory navigation fixes

### Standard Rules (1000)

Most rules use the default priority:

- Git rules
- Package manager rules
- Development tool rules

### Low Priority Rules (> 1000)

Rules that do more expensive work:

- **NoCommand** (priority: 3000) - Fuzzy executable matching
- Pattern-based rules that scan history

## Rule Implementation Patterns

### Simple Pattern Matching

```rust
impl Rule for Sudo {
    fn is_match(&self, cmd: &Command) -> bool {
        cmd.output.contains("Permission denied")
            || cmd.output.contains("operation not permitted")
    }

    fn get_new_command(&self, cmd: &Command) -> Vec<String> {
        vec![format!("sudo {}", cmd.script)]
    }
}
```

### Regex Matching

```rust
impl Rule for GitPush {
    fn is_match(&self, cmd: &Command) -> bool {
        lazy_static! {
            static ref RE: Regex = Regex::new(
                r"git push --set-upstream origin (\S+)"
            ).unwrap();
        }
        RE.is_match(&cmd.output)
    }
}
```

### Fuzzy Matching

```rust
impl Rule for GitCheckout {
    fn get_new_command(&self, cmd: &Command) -> Vec<String> {
        let branches = get_branches();
        let typo = extract_branch_name(&cmd.script);

        get_close_matches(&typo, &branches, 3, 0.6)
            .into_iter()
            .map(|b| format!("git checkout {}", b))
            .collect()
    }
}
```

### Application-Specific Rules

```rust
impl Rule for GitNotCommand {
    fn is_match(&self, cmd: &Command) -> bool {
        is_app(cmd, &["git"])
            && cmd.output.contains("is not a git command")
    }
}
```

## Rule Helpers

### `is_app()` Function

Check if command starts with specific application:

```rust
pub fn is_app(cmd: &Command, apps: &[&str]) -> bool {
    cmd.script_parts()
        .first()
        .map(|first| apps.iter().any(|app| first == *app))
        .unwrap_or(false)
}
```

### `for_app()` Wrapper

Limit a rule to specific applications:

```rust
pub struct ForAppRule<R: Rule> {
    inner: R,
    apps: &'static [&'static str],
}

impl<R: Rule> Rule for ForAppRule<R> {
    fn is_match(&self, cmd: &Command) -> bool {
        is_app(cmd, self.apps) && self.inner.is_match(cmd)
    }
}
```

### Fuzzy Matching Utilities

```rust
/// Get close matches for a string
pub fn get_close_matches(
    word: &str,
    possibilities: &[String],
    n: usize,
    cutoff: f64,
) -> Vec<String>;

/// Get the single closest match
pub fn get_closest(
    word: &str,
    possibilities: &[String],
) -> Option<String>;
```

## Adding Rules

1. Create a struct implementing `Rule`
2. Add to appropriate module in `src/rules/`
3. Register in `src/rules/mod.rs`:

```rust
pub fn get_all_rules() -> Vec<Box<dyn Rule>> {
    vec![
        // High priority
        Box::new(Sudo),
        // ...

        // Your new rule
        Box::new(MyNewRule),
    ]
}
```

## Testing Rules

Every rule should have tests:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_matches() {
        let cmd = Command::new("script", "output");
        assert!(MyRule.is_match(&cmd));
    }

    #[test]
    fn test_no_match() {
        let cmd = Command::new("other", "different");
        assert!(!MyRule.is_match(&cmd));
    }

    #[test]
    fn test_correction() {
        let cmd = Command::new("script", "output");
        let fixes = MyRule.get_new_command(&cmd);
        assert_eq!(fixes[0], "expected");
    }
}
```
