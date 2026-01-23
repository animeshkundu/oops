# Contributing Rules to oops

This guide explains how to contribute new correction rules to oops, including how to identify rules that need to be ported from thefuck.

## Finding Rules to Port

### Using the Parity Checker

oops includes a built-in tool to check which thefuck rules are missing:

```bash
# Check parity with default settings
cargo run --bin check_parity

# Check for rules updated in the last 30 days
cargo run --bin check_parity -- --days 30

# Get JSON output for automation
cargo run --bin check_parity -- --output json
```

The parity checker will show:
- Total rules in thefuck vs oops
- Coverage percentage
- List of missing rules
- Recently updated rules (if you have a local thefuck clone)

**Example Output:**
```
═══════════════════════════════════════════════════════════
             oops ↔ thefuck Parity Report
═══════════════════════════════════════════════════════════

📊 Summary:
   • thefuck rules: 159
   • oops rules:    177
   • Coverage:      111.3%
   • Missing:       12 rules
   • Recent activity: 0 rules

❌ Missing Rules (12):
   • cargo
   • flutter_command_not_found
   • git_push_set_upstream
   ...
```

### Prioritizing Rules to Port

1. **Recently Updated** (marked with 🔥): These often indicate bug fixes or new features
2. **High Value Tools**: Popular CLI tools (git, docker, npm, kubectl)
3. **Frequently Encountered Errors**: Permission errors, typos, common mistakes
4. **Cross-Platform**: Rules that work on Linux, macOS, and Windows

### Understanding thefuck Rules

Before porting, review the original Python rule:

1. Visit: https://github.com/nvbn/thefuck/tree/master/thefuck/rules
2. Find the rule file (e.g., `git_push_set_upstream.py`)
3. Understand:
   - What error patterns it matches
   - What correction it generates
   - Any special logic or edge cases

## Porting a Rule to Rust

### Step 1: Determine Rule Category

Place your rule in the appropriate module:

| Category | Path | Examples |
|----------|------|----------|
| Git | `src/rules/git/*.rs` | git_push, git_checkout |
| Package managers | `src/rules/package_managers/*.rs` | apt, brew, npm, pip |
| Cloud/Infrastructure | `src/rules/cloud.rs` | aws, kubectl, terraform |
| Containers | `src/rules/docker.rs` | docker, docker-compose |
| Dev tools | `src/rules/devtools.rs` | go, maven, gradle |
| Frameworks | `src/rules/frameworks.rs` | rails, django, react-native |
| System | `src/rules/system.rs` | chmod, mkdir, rm, cp |
| Shell utils | `src/rules/shell_utils.rs` | grep, sed, awk |
| Miscellaneous | `src/rules/misc.rs` | Everything else |

### Step 2: Implement the Rule

See the full guide in [creating-rules.md](creating-rules.md) for detailed implementation instructions.

**Quick Example:**

```rust
use crate::core::{is_app, Command, Rule};

#[derive(Debug, Clone, Copy, Default)]
pub struct GitPushSetUpstream;

impl GitPushSetUpstream {
    pub const fn new() -> Self { Self }
}

impl Rule for GitPushSetUpstream {
    fn name(&self) -> &str { "git_push_set_upstream" }
    
    fn is_match(&self, cmd: &Command) -> bool {
        is_app(cmd, &["git", "git.exe"]) 
            && cmd.output.contains("--set-upstream")
    }
    
    fn get_new_command(&self, cmd: &Command) -> Vec<String> {
        // Extract branch name and generate correction
        // Implementation details...
        vec![]
    }
}
```

### Step 3: Add Tests

Minimum 6 tests required:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::Command;
    
    #[test]
    fn test_matches_error() {
        let cmd = Command::new(
            "git push",
            "fatal: The current branch has no upstream branch.\n\
             To push the current branch and set the remote as upstream, use\n\
             git push --set-upstream origin main"
        );
        assert!(GitPushSetUpstream::new().is_match(&cmd));
    }
    
    #[test]
    fn test_not_matches_different_tool() {
        let cmd = Command::new("hg push", "unknown command");
        assert!(!GitPushSetUpstream::new().is_match(&cmd));
    }
    
    #[test]
    fn test_generates_correction() {
        let cmd = Command::new(
            "git push",
            "To push the current branch and set the remote as upstream, use\n\
             git push --set-upstream origin main"
        );
        let fixes = GitPushSetUpstream::new().get_new_command(&cmd);
        assert!(fixes.iter().any(|f| f.contains("--set-upstream")));
    }
    
    // Add 3 more tests for edge cases...
}
```

### Step 4: Register the Rule

Add to `src/rules/mod.rs`:

```rust
// In the appropriate module
pub mod git_push_set_upstream;

// In get_all_rules()
rules.extend(git::all_rules());  // If it's a git rule
// or add directly if standalone
```

### Step 5: Quality Checks

```bash
# Run tests
cargo test

# Check formatting
cargo fmt

# Run linter
cargo clippy -- -D warnings

# Build
cargo build --release
```

### Step 6: Verify Parity

Run the parity checker again to confirm your rule is detected:

```bash
cargo run --bin check_parity
```

The missing count should decrease by 1.

## Porting Guidelines

### Do's ✅

- Use REAL error messages from actual command output
- Test on multiple platforms (Linux, macOS, Windows if applicable)
- Handle edge cases (empty command, special characters, etc.)
- Use `is_app()` for cross-platform tool detection
- Write descriptive test names
- Document any deviation from thefuck's behavior

### Don'ts ❌

- Don't hard-code error messages that change between versions
- Don't skip Windows compatibility (use `.exe` suffix)
- Don't assume Unix-only shell syntax
- Don't copy Python code directly (translate concepts, not code)
- Don't modify unrelated rules
- Don't skip tests

## Translation Tips

### Python → Rust Common Patterns

| Python | Rust |
|--------|------|
| `cmd.script.startswith('git')` | `is_app(cmd, &["git", "git.exe"])` |
| `'error' in cmd.output.lower()` | `cmd.output.to_lowercase().contains("error")` |
| `re.search(r'pattern', cmd.output)` | `Regex::new(r"pattern")?.is_match(&cmd.output)` |
| `difflib.get_close_matches()` | `get_close_matches(typo, options, 3, 0.6)` |
| `cmd.script.split()` | `cmd.script_parts()` |

### Handling thefuck-specific Features

Some thefuck features don't have direct equivalents:

- **Dynamic rules**: oops rules are compiled in, not loaded from files
- **Python expressions in config**: Use explicit rule enables/disables
- **`for_app()` decorator**: Use `is_app()` in `is_match()`
- **`get_new_command()` returning generator**: Return `Vec<String>`

## Testing Your Port

### Manual Testing

```bash
# Build your changes
cargo build

# Try triggering the error
git push  # or whatever command

# Run oops
./target/debug/oops

# Should show your correction
```

### Automated Testing

```bash
# Run all tests
cargo test

# Run tests for specific rule
cargo test git_push_set_upstream

# Run with output
cargo test -- --nocapture
```

## Submitting Your Contribution

1. Create a feature branch: `git checkout -b feature/add-git-push-set-upstream`
2. Commit your changes: `git commit -m "feat(rules): add git_push_set_upstream"`
3. Push to your fork: `git push -u origin feature/add-git-push-set-upstream`
4. Open a pull request with:
   - Description of the rule
   - Example error it fixes
   - Test coverage summary
   - Link to original thefuck rule (if porting)

## Need Help?

- Check existing rules in `src/rules/` for examples
- Review [creating-rules.md](creating-rules.md) for detailed API docs
- Ask questions in GitHub issues
- Reference thefuck implementation for logic

## Tracking Progress

Use the parity checker regularly to track progress:

```bash
# Weekly check
./scripts/check-parity.sh > parity-report.txt

# Track over time
git log --all --grep="feat(rules)" --oneline
```

Remember: Quality over quantity. A well-tested, cross-platform rule is better than a quick port that only works in one environment.
