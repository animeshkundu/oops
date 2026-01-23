# Parity Checker GitHub API Enhancement

**Date**: 2026-01-23  
**Agent**: Claude (rust-expert specialization)  
**Context**: Enhanced parity checker to fetch rules dynamically from GitHub API and use library's get_all_rules()

## Summary

Upgraded the parity checker (`check_parity`) binary to:
1. **Dynamically fetch thefuck rules** from GitHub API (no static list)
2. **Use `get_all_rules()`** from oops library instead of scanning source files
3. **Provide robust comparison** with categorized missing rules
4. **Add minimal dependencies** (ureq for HTTP, checked with gh-advisory-database)

## Key Changes

### 1. Dynamic GitHub API Fetching

**Before**: Used a static list of 159 hardcoded rule names, with optional local clone scanning.

**After**: Fetches rules directly from GitHub API at runtime:
```rust
fn fetch_thefuck_rules_from_github(verbose: bool) -> Result<Vec<TheFuckRule>> {
    let url = format!(
        "{}/repos/{}/contents/{}",
        GITHUB_API_BASE, THEFUCK_REPO, THEFUCK_RULES_PATH
    );

    let response = ureq::get(&url)
        .set("Accept", "application/vnd.github+json")
        .set("User-Agent", "oops-parity-checker")
        .timeout(std::time::Duration::from_secs(10))
        .call()?;

    let files: Vec<GitHubFile> = response.into_json()?;
    // Filter for .py files, exclude __init__.py
    // ...
}
```

**Benefits**:
- No manual maintenance of rule lists
- Always up-to-date with thefuck repository
- Can detect new rules immediately
- Includes metadata (SHA, GitHub URL) for each rule

### 2. Library Integration with get_all_rules()

**Before**: Parsed Rust source files to extract rule names using regex.

**After**: Uses the official `get_all_rules()` function from the library:
```rust
fn get_oops_rules_from_library(verbose: bool) -> Result<Vec<OopsRule>> {
    let all_rules = get_all_rules();
    let rules: Vec<OopsRule> = all_rules
        .iter()
        .map(|rule| OopsRule {
            name: rule.name().to_string(),
            priority: rule.priority(),
            requires_output: rule.requires_output(),
            enabled_by_default: rule.enabled_by_default(),
        })
        .collect();
    Ok(rules)
}
```

**Benefits**:
- Uses official API, no parsing heuristics
- Gets accurate metadata (priority, requires_output, etc.)
- Automatically includes all registered rules
- More maintainable and reliable

### 3. Enhanced Reporting

**New Features**:
- **Categorized missing rules**: Groups by Git, Docker, Node.js, Package Managers, etc.
- **Metadata display**: Shows priority and requirements in verbose mode
- **Better organization**: Alphabetically sorted within categories
- **Direct GitHub links**: Each missing rule links to its source

**Example Output**:
```
📊 Summary:
   • thefuck rules: 169
   • oops rules:    175
   • Coverage:      98.2%
   • Missing:       3 rules

❌ Missing Rules (3):

   Build Tools (1 rules):
      • cargo

   Miscellaneous (2 rules):
      • history
      • test
```

### 4. Minimal Dependencies

Added only one new dependency:
```toml
ureq = { version = "2.10", default-features = false, features = ["json", "tls"] }
```

**Why ureq?**
- Synchronous HTTP client (simpler than async)
- Minimal footprint (no tokio runtime)
- Native TLS support
- Built-in JSON deserialization
- **Security**: Checked with gh-advisory-database (no vulnerabilities)

## Technical Details

### Data Structures

```rust
/// GitHub API response for directory listing
#[derive(Debug, Clone, Serialize, Deserialize)]
struct GitHubFile {
    name: String,
    path: String,
    sha: String,
    #[serde(rename = "type")]
    file_type: String,
    html_url: String,
}

/// A rule from oops with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
struct OopsRule {
    name: String,
    priority: i32,
    requires_output: bool,
    enabled_by_default: bool,
}

/// Enhanced parity report
#[derive(Debug, Clone, Serialize, Deserialize)]
struct ParityReport {
    thefuck_rules: Vec<TheFuckRule>,
    oops_rules: Vec<OopsRule>,
    missing_rules: Vec<TheFuckRule>,
    total_thefuck_rules: usize,
    total_oops_rules: usize,
    coverage_percentage: f64,
    name_mappings: HashMap<String, String>,
}
```

### Categorization Algorithm

The `categorize_rule()` function groups rules by prefix:
- `git_*` → Git
- `docker_*` → Docker
- `npm_*`, `yarn_*`, `npx_*` → Node.js
- `brew_*` → Homebrew
- `apt_*`, `pacman`, `dnf_*` → Package Managers
- `cargo`, `mvn_*`, `gradle_*` → Build Tools
- `aws_*`, `az_*`, `heroku_*` → Cloud Services
- And 7 more categories...

### Command Line Interface

**Simplified Usage**:
```bash
# Default output
cargo run --bin check_parity

# JSON output
cargo run --bin check_parity -- --output json

# Verbose mode (shows all rules with metadata)
cargo run --bin check_parity -- --verbose
```

**Removed**:
- `--days` flag (no longer needed without local clone scanning)

## Testing

### Updated Tests

All tests in `tests/check_parity_test.rs` updated and passing:

1. `test_check_parity_runs` - Basic execution
2. `test_check_parity_json_output` - JSON format validation
3. `test_check_parity_finds_rules` - Statistics reporting
4. `test_check_parity_verbose_output` - Verbose mode
5. `test_uses_get_all_rules_function` - Verifies library integration
6. `test_fetches_from_github` - Verifies GitHub API usage
7. `test_categorizes_missing_rules` - Verifies categorization

```bash
$ cargo test --test check_parity_test
running 7 tests
test test_categorizes_missing_rules ... ok
test test_check_parity_finds_rules ... ok
test test_check_parity_json_output ... ok
test test_check_parity_runs ... ok
test test_check_parity_verbose_output ... ok
test test_fetches_from_github ... ok
test test_uses_get_all_rules_function ... ok

test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

### Code Quality

- ✅ All tests pass (`cargo test`)
- ✅ Formatted (`cargo fmt`)
- ✅ No clippy warnings (`cargo clippy -- -D warnings`)
- ✅ Compiles without warnings

## Current Parity Status

As of this update:
- **thefuck rules**: 169 (fetched from GitHub)
- **oops rules**: 175 (from get_all_rules())
- **Coverage**: 98.2%
- **Missing rules**: 3
  1. `cargo` - Cargo command corrections
  2. `history` - Shell history-based corrections
  3. `test` - Test command corrections

**Note**: oops has 6 more rules than thefuck because some rules are split for better granularity (e.g., git rules by error type).

## Migration Path

### For Future Rule Additions

When adding a new rule to oops:
1. Implement the rule in `src/rules/`
2. Register it in `get_all_rules()` in `src/rules/mod.rs`
3. Run `cargo run --bin check_parity` to verify detection
4. The rule will automatically appear in the report

No manual list updates needed!

## Performance Considerations

- **Startup time**: ~500ms (includes HTTP request to GitHub)
- **Network dependency**: Requires internet access (cached by OS/DNS)
- **Rate limiting**: GitHub API allows 60 req/hour unauthenticated (sufficient)
- **Fallback**: Could add caching or static fallback if needed

## Security

- ✅ ureq dependency scanned with gh-advisory-database
- ✅ Uses TLS for GitHub API requests
- ✅ No credentials required (public API)
- ✅ User-Agent header identifies tool

## Future Enhancements

1. **Rate Limit Handling**: Add GitHub token support for higher rate limits
2. **Caching**: Cache GitHub response for offline/CI usage
3. **Historical Tracking**: Store results over time to show coverage trends
4. **PR Integration**: Comment on PRs with coverage impact
5. **Rule Difficulty Scoring**: Analyze thefuck rules to estimate porting effort
6. **Automated Issue Creation**: File GitHub issues for missing high-priority rules

## Handoff Context

### What Works
- Dynamic fetching from GitHub API
- Library integration via get_all_rules()
- Robust categorization and reporting
- Comprehensive test coverage
- Clean, maintainable code

### What to Watch
- GitHub API rate limits (60/hour unauthenticated)
- Network failures (no fallback currently)
- Changes to GitHub API format

### Next Steps
1. Consider adding GitHub token support for CI usage
2. Implement the 3 missing rules (cargo, history, test)
3. Add caching layer for offline development
4. Integrate into CI pipeline to track coverage

## References

- GitHub API: https://docs.github.com/en/rest/repos/contents
- thefuck repository: https://github.com/nvbn/thefuck
- ureq documentation: https://docs.rs/ureq/
- Previous handoff: docs/handoffs/2026-01-23-parity-checker-implementation.md
