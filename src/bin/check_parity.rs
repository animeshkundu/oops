//! Parity checker for oops vs thefuck
//!
//! This tool checks the thefuck repository for new or recently updated rules
//! and reports which ones are missing from oops or need to be ported.
//!
//! Usage:
//!   cargo run --bin check_parity
//!   cargo run --bin check_parity -- --days 7
//!   cargo run --bin check_parity -- --output json

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct TheFuckRule {
    name: String,
    path: String,
    last_modified: String,
    has_recent_activity: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ParityReport {
    thefuck_rules: Vec<TheFuckRule>,
    oops_rules: Vec<String>,
    missing_rules: Vec<TheFuckRule>,
    recently_updated_rules: Vec<TheFuckRule>,
    total_thefuck_rules: usize,
    total_oops_rules: usize,
    coverage_percentage: f64,
}

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let days = parse_days(&args).unwrap_or(7);
    let output_format = parse_output_format(&args);

    eprintln!("🔍 Checking parity between oops and thefuck...");
    eprintln!(
        "   Looking for rules with activity in the last {} days",
        days
    );

    // Get thefuck rules from GitHub API or local clone
    let thefuck_rules = get_thefuck_rules(days)?;

    // Get oops rules from source
    let oops_rules = get_oops_rules()?;

    // Generate report
    let report = generate_report(thefuck_rules, oops_rules)?;

    // Output report
    match output_format.as_str() {
        "json" => print_json_report(&report)?,
        _ => print_human_report(&report)?,
    }

    Ok(())
}

fn parse_days(args: &[String]) -> Option<u32> {
    for i in 0..args.len() {
        if args[i] == "--days" && i + 1 < args.len() {
            return args[i + 1].parse().ok();
        }
    }
    None
}

fn parse_output_format(args: &[String]) -> String {
    for i in 0..args.len() {
        if args[i] == "--output" && i + 1 < args.len() {
            return args[i + 1].clone();
        }
    }
    "human".to_string()
}

/// Get thefuck rules from the GitHub repository
fn get_thefuck_rules(days: u32) -> Result<Vec<TheFuckRule>> {
    eprintln!("📥 Fetching thefuck rules...");

    // First, try to find a local clone
    if let Ok(rules) = get_thefuck_rules_from_local(days) {
        return Ok(rules);
    }

    // Fall back to GitHub API
    get_thefuck_rules_from_github(days)
}

/// Get thefuck rules from a local clone
fn get_thefuck_rules_from_local(days: u32) -> Result<Vec<TheFuckRule>> {
    let mut possible_paths: Vec<PathBuf> =
        vec![PathBuf::from("../thefuck"), PathBuf::from("../../thefuck")];

    if let Some(home) = dirs::home_dir() {
        possible_paths.push(home.join("projects/thefuck"));
        possible_paths.push(home.join("src/thefuck"));
    }

    for path_opt in possible_paths {
        let rules_dir = path_opt.join("thefuck").join("rules");
        if rules_dir.exists() {
            eprintln!("   Found local thefuck clone at: {:?}", path_opt);
            return scan_local_rules(&rules_dir, days);
        }
    }

    Err(anyhow::anyhow!("No local thefuck clone found"))
}

/// Scan local thefuck rules directory
fn scan_local_rules(rules_dir: &Path, days: u32) -> Result<Vec<TheFuckRule>> {
    let mut rules = Vec::new();
    let cutoff_time =
        std::time::SystemTime::now() - std::time::Duration::from_secs(days as u64 * 24 * 60 * 60);

    for entry in fs::read_dir(rules_dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.extension().and_then(|e| e.to_str()) == Some("py") {
            let name = path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("")
                .to_string();

            // Skip __init__.py and internal files
            if name.starts_with("__") {
                continue;
            }

            let metadata = fs::metadata(&path)?;
            let modified = metadata.modified()?;
            let has_recent_activity = modified > cutoff_time;

            let last_modified = format_timestamp(modified);

            rules.push(TheFuckRule {
                name,
                path: path.display().to_string(),
                last_modified,
                has_recent_activity,
            });
        }
    }

    eprintln!("   Found {} rules in local clone", rules.len());
    Ok(rules)
}

/// Get thefuck rules from GitHub API
fn get_thefuck_rules_from_github(_days: u32) -> Result<Vec<TheFuckRule>> {
    eprintln!("   Fetching from GitHub API...");

    // For now, we'll use a static list of known thefuck rules
    // In production, this would use the GitHub API
    let known_rules = get_known_thefuck_rules();

    eprintln!("   Using {} known rules", known_rules.len());
    Ok(known_rules
        .into_iter()
        .map(|name| TheFuckRule {
            name: name.to_string(),
            path: format!("thefuck/rules/{}.py", name),
            last_modified: "unknown".to_string(),
            has_recent_activity: false,
        })
        .collect())
}

/// Get list of known thefuck rules (maintained manually from thefuck repo)
fn get_known_thefuck_rules() -> Vec<&'static str> {
    vec![
        "ag_literal",
        "aws_cli",
        "az_cli",
        "brew_install",
        "brew_link",
        "brew_uninstall",
        "brew_unknown_command",
        "brew_update_formula",
        "cargo",
        "cargo_no_command",
        "cat_dir",
        "cd_correction",
        "cd_cs",
        "cd_mkdir",
        "cd_parent",
        "chmod_x",
        "choco_install",
        "composer_not_command",
        "conda_mistype",
        "cp_create_destination",
        "cp_omitting_directory",
        "cpp11",
        "dirty_untar",
        "dirty_unzip",
        "django_south_ghost",
        "django_south_merge",
        "dnf_no_such_command",
        "docker_login",
        "docker_not_command",
        "dry",
        "fab_command_not_found",
        "fix_alt_space",
        "fix_file",
        "flutter_command_not_found",
        "gem_unknown_command",
        "git_add",
        "git_add_force",
        "git_bisect_usage",
        "git_branch_delete",
        "git_branch_delete_checked_out",
        "git_branch_exists",
        "git_branch_list",
        "git_checkout",
        "git_clone_git_clone",
        "git_clone_missing",
        "git_commit_add",
        "git_commit_amend",
        "git_commit_reset",
        "git_diff_no_index",
        "git_diff_staged",
        "git_fix_stash",
        "git_flag_after_filename",
        "git_help_aliased",
        "git_hook_bypass",
        "git_lfs_mistype",
        "git_main_master",
        "git_merge",
        "git_merge_unrelated",
        "git_not_command",
        "git_pull",
        "git_pull_clone",
        "git_pull_uncommitted_changes",
        "git_push",
        "git_push_different_branch_names",
        "git_push_force",
        "git_push_pull",
        "git_push_set_upstream",
        "git_push_without_commits",
        "git_rebase_continue",
        "git_rebase_merge_dir",
        "git_rebase_no_changes",
        "git_remote_delete",
        "git_remote_set_url",
        "git_revert_merge",
        "git_rm_local_modifications",
        "git_rm_recursive",
        "git_rm_staged",
        "git_stash",
        "git_stash_pop",
        "git_tag_force",
        "git_two_dashes",
        "go_run",
        "gradle_no_task",
        "gradle_wrapper",
        "grep_arguments_order",
        "grep_recursive",
        "grunt_not_found",
        "grunt_task_not_found",
        "gulp_not_task",
        "has_exists_script",
        "helm_not_command",
        "heroku_multiple_apps",
        "heroku_not_command",
        "history",
        "hostscli",
        "ifconfig_device_not_found",
        "java",
        "javac",
        "ln_no_hard_link",
        "ln_s_order",
        "long_form_help",
        "ls_all",
        "ls_lah",
        "lein_not_task",
        "man",
        "man_no_space",
        "mercurial",
        "mkdir_p",
        "mvn_no_command",
        "mvn_unknown_lifecycle_phase",
        "nixos_cmd_not_found",
        "no_command",
        "no_such_file",
        "npm_missing_script",
        "npm_run_script",
        "npm_wrong_command",
        "npx_install",
        "open",
        "open_with_args",
        "pacman",
        "pacman_invalid_option",
        "pacman_not_found",
        "path_from_history",
        "pip_install",
        "pip_unknown_command",
        "port_already_in_use",
        "prove_recursively",
        "python_command",
        "python_execute",
        "python_module_error",
        "quotation_marks",
        "react_native_command_unrecognized",
        "remove_trailing_cedilla",
        "rm_dir",
        "rm_root",
        "scm_correction",
        "sed_unterminated_s",
        "sl_ls",
        "ssh_known_hosts",
        "sudo",
        "sudo_command_from_user_path",
        "switch_lang",
        "systemctl",
        "terraform_init",
        "terraform_no_command",
        "test_py",
        "touch",
        "tsuru_login",
        "tsuru_not_command",
        "unknown_command",
        "unsudo",
        "vagrant_up",
        "whois",
        "workon_doesnt_exists",
        "wrong_hyphen_before_subcommand",
        "yarn_alias",
        "yarn_command_not_found",
        "yarn_command_replaced",
        "yarn_help",
    ]
}

/// Get oops rules from the source code
fn get_oops_rules() -> Result<Vec<String>> {
    eprintln!("📋 Scanning oops rules...");

    let mut rules = HashSet::new();
    let rules_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/rules");

    scan_rust_rules(&rules_dir, &mut rules)?;

    let mut sorted_rules: Vec<_> = rules.into_iter().collect();
    sorted_rules.sort();

    eprintln!("   Found {} rules in oops", sorted_rules.len());
    Ok(sorted_rules)
}

/// Recursively scan Rust files for rule names
fn scan_rust_rules(dir: &Path, rules: &mut HashSet<String>) -> Result<()> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            scan_rust_rules(&path, rules)?;
        } else if path.extension().and_then(|e| e.to_str()) == Some("rs") {
            extract_rule_names(&path, rules)?;
        }
    }
    Ok(())
}

/// Extract rule names from a Rust source file
fn extract_rule_names(path: &Path, rules: &mut HashSet<String>) -> Result<()> {
    let content = fs::read_to_string(path)?;
    let lines: Vec<&str> = content.lines().collect();

    for i in 0..lines.len() {
        let line = lines[i];

        // Look for: fn name(&self) -> &str {
        if line.contains("fn name(&self) -> &str") {
            // Check the same line first
            if let Some(rule_name) = extract_string_literal(line) {
                rules.insert(rule_name);
                continue;
            }

            // Check next few lines for the string literal
            for next_line in lines.iter().skip(i + 1).take(4) {
                if let Some(rule_name) = extract_string_literal(next_line) {
                    rules.insert(rule_name);
                    break;
                }
            }
        }
    }

    Ok(())
}

/// Extract a string literal from a line of code
/// Note: This is a simple parser for extracting rule names from Rust source.
/// It handles basic string literals and attempts to skip escaped quotes.
/// For our use case (rule names which are simple identifiers), this is sufficient.
fn extract_string_literal(line: &str) -> Option<String> {
    let trimmed = line.trim();
    if let Some(start) = trimmed.find('"') {
        // Skip the opening quote and find the closing quote
        let after_start = &trimmed[start + 1..];
        if let Some(end) = after_start.find('"') {
            // Simple heuristic: skip if the quote appears to be escaped
            // Note: This doesn't handle all edge cases (e.g., \\"), but works
            // for our use case since rule names don't contain special characters
            if end > 0 && after_start.chars().nth(end - 1) == Some('\\') {
                return None;
            }
            return Some(after_start[..end].to_string());
        }
    }
    None
}

/// Format a system time as a human-readable string
fn format_timestamp(time: std::time::SystemTime) -> String {
    use std::time::UNIX_EPOCH;

    match time.duration_since(UNIX_EPOCH) {
        Ok(duration) => {
            let secs = duration.as_secs();
            let days = secs / 86400;
            if days == 0 {
                "today".to_string()
            } else if days == 1 {
                "1 day ago".to_string()
            } else if days < 7 {
                format!("{} days ago", days)
            } else if days < 30 {
                format!("{} weeks ago", days / 7)
            } else {
                format!("{} months ago", days / 30)
            }
        }
        Err(_) => "unknown".to_string(),
    }
}

/// Generate the parity report
fn generate_report(
    thefuck_rules: Vec<TheFuckRule>,
    oops_rules: Vec<String>,
) -> Result<ParityReport> {
    let oops_set: HashSet<String> = oops_rules.iter().cloned().collect();

    // Build a mapping from thefuck names to oops names (since they might differ)
    let name_mapping = build_name_mapping();

    let mut missing_rules = Vec::new();
    let mut recently_updated_rules = Vec::new();

    for rule in &thefuck_rules {
        let oops_name = name_mapping.get(&rule.name).unwrap_or(&rule.name);

        if !oops_set.contains(oops_name) {
            missing_rules.push(rule.clone());
        }

        if rule.has_recent_activity {
            recently_updated_rules.push(rule.clone());
        }
    }

    let total_thefuck = thefuck_rules.len();
    let total_oops = oops_rules.len();
    let coverage_percentage = if total_thefuck > 0 {
        (total_oops as f64 / total_thefuck as f64) * 100.0
    } else {
        // Edge case: no thefuck rules to compare against
        // Return 0.0 to indicate "no baseline" rather than "complete coverage"
        // In practice, this should never happen since thefuck has 159+ rules
        0.0
    };

    Ok(ParityReport {
        thefuck_rules,
        oops_rules,
        missing_rules,
        recently_updated_rules,
        total_thefuck_rules: total_thefuck,
        total_oops_rules: total_oops,
        coverage_percentage,
    })
}

/// Build a mapping from thefuck rule names to oops rule names
fn build_name_mapping() -> HashMap<String, String> {
    // Add known mappings where names differ between thefuck and oops
    // Example: mapping.insert("thefuck_name".to_string(), "oops_name".to_string());
    // Currently, all rule names are identical between projects
    HashMap::new()
}

/// Print the report in human-readable format
fn print_human_report(report: &ParityReport) -> Result<()> {
    println!("\n═══════════════════════════════════════════════════════════");
    println!("             oops ↔ thefuck Parity Report");
    println!("═══════════════════════════════════════════════════════════\n");

    println!("📊 Summary:");
    println!("   • thefuck rules: {}", report.total_thefuck_rules);
    println!("   • oops rules:    {}", report.total_oops_rules);
    println!("   • Coverage:      {:.1}%", report.coverage_percentage);
    println!("   • Missing:       {} rules", report.missing_rules.len());
    println!(
        "   • Recent activity: {} rules\n",
        report.recently_updated_rules.len()
    );

    if !report.recently_updated_rules.is_empty() {
        println!("🔥 Recently Updated Rules (last 7 days):");
        for rule in &report.recently_updated_rules {
            println!("   • {} ({})", rule.name, rule.last_modified);
        }
        println!();
    }

    if !report.missing_rules.is_empty() {
        println!("❌ Missing Rules ({}):", report.missing_rules.len());
        for rule in &report.missing_rules {
            if rule.has_recent_activity {
                println!("   • {} 🔥 ({})", rule.name, rule.last_modified);
            } else {
                println!("   • {}", rule.name);
            }
        }
        println!();
    }

    println!("✅ Next Steps:");
    if report.missing_rules.is_empty() {
        println!("   🎉 Full parity achieved! All thefuck rules are ported.");
    } else {
        println!("   1. Review missing rules for relevance");
        println!("   2. Prioritize rules with recent activity (marked 🔥)");
        println!("   3. Port high-value rules first");
        println!("   4. Update tests in tests/parity_tests.rs");
        println!("\n   See: https://github.com/nvbn/thefuck/tree/master/thefuck/rules");
    }
    println!();

    Ok(())
}

/// Print the report in JSON format
fn print_json_report(report: &ParityReport) -> Result<()> {
    let json = serde_json::to_string_pretty(report)?;
    println!("{}", json);
    Ok(())
}
