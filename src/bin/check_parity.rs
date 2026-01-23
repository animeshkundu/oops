//! Parity checker for oops vs thefuck
//!
//! This tool dynamically fetches rules from the thefuck GitHub repository
//! and compares them with oops rules to identify missing coverage.
//!
//! Usage:
//!   cargo run --bin check_parity
//!   cargo run --bin check_parity -- --output json
//!   cargo run --bin check_parity -- --verbose

use anyhow::{Context, Result};
use oops::rules::get_all_rules;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

const GITHUB_API_BASE: &str = "https://api.github.com";
const THEFUCK_REPO: &str = "nvbn/thefuck";
const THEFUCK_RULES_PATH: &str = "thefuck/rules";

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

/// A rule from thefuck repository
#[derive(Debug, Clone, Serialize, Deserialize)]
struct TheFuckRule {
    name: String,
    path: String,
    sha: String,
    url: String,
}

/// A rule from oops with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
struct OopsRule {
    name: String,
    priority: i32,
    requires_output: bool,
    enabled_by_default: bool,
}

/// The complete parity report
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

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let output_format = parse_output_format(&args);
    let verbose = args.contains(&"--verbose".to_string());

    if !verbose {
        eprintln!("🔍 Checking parity between oops and thefuck...");
    }

    // Fetch thefuck rules from GitHub
    let thefuck_rules = fetch_thefuck_rules_from_github(verbose)?;

    // Get oops rules using get_all_rules()
    let oops_rules = get_oops_rules_from_library(verbose)?;

    // Generate comparison report
    let report = generate_report(thefuck_rules, oops_rules)?;

    // Output report
    match output_format.as_str() {
        "json" => print_json_report(&report)?,
        _ => print_human_report(&report, verbose)?,
    }

    Ok(())
}

fn parse_output_format(args: &[String]) -> String {
    for i in 0..args.len() {
        if args[i] == "--output" && i + 1 < args.len() {
            return args[i + 1].clone();
        }
    }
    "human".to_string()
}

/// Fetch thefuck rules from GitHub API
fn fetch_thefuck_rules_from_github(verbose: bool) -> Result<Vec<TheFuckRule>> {
    if verbose {
        eprintln!("📥 Fetching thefuck rules from GitHub API...");
    }

    let url = format!(
        "{}/repos/{}/contents/{}",
        GITHUB_API_BASE, THEFUCK_REPO, THEFUCK_RULES_PATH
    );

    let response = ureq::get(&url)
        .set("Accept", "application/vnd.github+json")
        .set("User-Agent", "oops-parity-checker")
        .timeout(std::time::Duration::from_secs(10))
        .call()
        .context("Failed to fetch thefuck rules from GitHub API. Check your internet connection and GitHub API rate limits (60 requests per hour per IP for unauthenticated requests)")?;

    let files: Vec<GitHubFile> = response
        .into_json()
        .context("Failed to parse GitHub API response")?;

    let rules: Vec<TheFuckRule> = files
        .into_iter()
        .filter(|f| f.file_type == "file" && f.name.ends_with(".py") && f.name != "__init__.py")
        .map(|f| {
            let name = f.name.trim_end_matches(".py").to_string();
            TheFuckRule {
                name,
                path: f.path,
                sha: f.sha,
                url: f.html_url,
            }
        })
        .collect();

    if verbose {
        eprintln!("   Found {} thefuck rules", rules.len());
    }

    Ok(rules)
}

/// Get oops rules using get_all_rules()
fn get_oops_rules_from_library(verbose: bool) -> Result<Vec<OopsRule>> {
    if verbose {
        eprintln!("📋 Loading oops rules from library...");
    }

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

    if verbose {
        eprintln!("   Found {} oops rules", rules.len());
    }

    Ok(rules)
}

/// Generate the parity report with detailed comparison
fn generate_report(
    thefuck_rules: Vec<TheFuckRule>,
    oops_rules: Vec<OopsRule>,
) -> Result<ParityReport> {
    let oops_name_set: HashSet<String> = oops_rules.iter().map(|r| r.name.clone()).collect();

    // Build name mapping for rules that differ between thefuck and oops
    let name_mapping = build_name_mapping();

    let mut missing_rules = Vec::new();

    for rule in &thefuck_rules {
        if is_rule_missing(&rule.name, &name_mapping, &oops_name_set) {
            missing_rules.push(rule.clone());
        }
    }

    let total_thefuck = thefuck_rules.len();
    let total_oops = oops_rules.len();
    let coverage_percentage = if total_thefuck > 0 {
        ((total_thefuck - missing_rules.len()) as f64 / total_thefuck as f64) * 100.0
    } else {
        0.0
    };

    Ok(ParityReport {
        thefuck_rules,
        oops_rules,
        missing_rules,
        total_thefuck_rules: total_thefuck,
        total_oops_rules: total_oops,
        coverage_percentage,
        name_mappings: name_mapping,
    })
}

/// Check if a thefuck rule is missing from oops
///
/// This checks both the original rule name and any mapped name (for cases
/// where thefuck and oops use different names for the same functionality).
///
/// A rule is considered present if either:
/// - The mapped name (if one exists) is found in oops, OR
/// - The original name (if no mapping or as fallback) is found in oops
fn is_rule_missing(
    rule_name: &str,
    name_mapping: &HashMap<String, String>,
    oops_rules: &HashSet<String>,
) -> bool {
    // Check if there's a mapping for this rule
    if let Some(mapped_name) = name_mapping.get(rule_name) {
        // If mapped, check only the mapped name
        !oops_rules.contains(mapped_name)
    } else {
        // No mapping, check the original name
        !oops_rules.contains(rule_name)
    }
}

/// Build a mapping from thefuck rule names to oops rule names
/// for cases where the naming differs between projects
fn build_name_mapping() -> HashMap<String, String> {
    // Add known mappings where names differ
    // Format: mapping.insert("thefuck_name".to_string(), "oops_name".to_string());

    // Example: apt_get -> apt (if we consolidate)
    // Most rules maintain the same names between projects

    HashMap::new()
}

/// Print the report in human-readable format
fn print_human_report(report: &ParityReport, verbose: bool) -> Result<()> {
    println!("\n═══════════════════════════════════════════════════════════");
    println!("             oops ↔ thefuck Parity Report");
    println!("═══════════════════════════════════════════════════════════\n");

    println!("📊 Summary:");
    println!("   • thefuck rules: {}", report.total_thefuck_rules);
    println!("   • oops rules:    {}", report.total_oops_rules);
    println!("   • Coverage:      {:.1}%", report.coverage_percentage);
    println!("   • Missing:       {} rules\n", report.missing_rules.len());

    if verbose && !report.oops_rules.is_empty() {
        println!("✅ Implemented Rules ({}):", report.oops_rules.len());
        for rule in &report.oops_rules {
            println!(
                "   • {} (priority: {}, requires_output: {})",
                rule.name, rule.priority, rule.requires_output
            );
        }
        println!();
    }

    if !report.missing_rules.is_empty() {
        println!("❌ Missing Rules ({}):", report.missing_rules.len());

        // Group missing rules by category
        let mut categorized: HashMap<&str, Vec<&TheFuckRule>> = HashMap::new();
        for rule in &report.missing_rules {
            let category = categorize_rule(&rule.name);
            categorized.entry(category).or_default().push(rule);
        }

        // Sort categories by name
        let mut categories: Vec<_> = categorized.keys().copied().collect();
        categories.sort();

        for category in categories {
            if let Some(rules) = categorized.get(category) {
                println!("\n   {} ({} rules):", category, rules.len());
                let mut sorted_rules = rules.clone();
                sorted_rules.sort_by(|a, b| a.name.cmp(&b.name));
                for rule in sorted_rules {
                    if verbose {
                        println!("      • {} ({})", rule.name, rule.url);
                    } else {
                        println!("      • {}", rule.name);
                    }
                }
            }
        }
        println!();
    }

    println!("✅ Next Steps:");
    if report.missing_rules.is_empty() {
        println!("   🎉 Full parity achieved! All thefuck rules are ported.");
    } else {
        println!("   1. Review missing rules for relevance to Rust ecosystem");
        println!("   2. Prioritize frequently-used commands (git, docker, npm, etc.)");
        println!("   3. Implement high-value rules with tests");
        println!("   4. Update tests in tests/parity_tests.rs");
        println!("\n   Reference: https://github.com/nvbn/thefuck/tree/master/thefuck/rules");
    }
    println!();

    Ok(())
}

/// Categorize a rule by its name prefix
fn categorize_rule(name: &str) -> &'static str {
    if name.starts_with("git_") {
        "Git"
    } else if name.starts_with("docker_") {
        "Docker"
    } else if name.starts_with("npm_") || name.starts_with("yarn_") || name.starts_with("npx_") {
        "Node.js"
    } else if name.starts_with("brew_") {
        "Homebrew"
    } else if name.starts_with("apt_") || name.starts_with("pacman") || name.starts_with("dnf_") {
        "Package Managers"
    } else if name.starts_with("cargo") || name.starts_with("mvn_") || name.starts_with("gradle_") {
        "Build Tools"
    } else if name.starts_with("aws_") || name.starts_with("az_") || name.starts_with("heroku_") {
        "Cloud Services"
    } else if name.starts_with("python_") || name.starts_with("java") || name.starts_with("go_") {
        "Programming Languages"
    } else if name.starts_with("cd_")
        || name.starts_with("ls_")
        || name.starts_with("cp_")
        || name.starts_with("rm_")
        || name.starts_with("mkdir_")
    {
        "Shell/System"
    } else if name.starts_with("grep_") || name.starts_with("sed_") || name.starts_with("adb_") {
        "CLI Utilities"
    } else if name.starts_with("terraform_")
        || name.starts_with("kubectl_")
        || name.starts_with("helm_")
    {
        "Infrastructure"
    } else if name.starts_with("django_")
        || name.starts_with("react_")
        || name.starts_with("rails_")
    {
        "Web Frameworks"
    } else {
        "Miscellaneous"
    }
}

/// Print the report in JSON format
fn print_json_report(report: &ParityReport) -> Result<()> {
    let json = serde_json::to_string_pretty(report)?;
    println!("{}", json);
    Ok(())
}
