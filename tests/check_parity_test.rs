//! Tests for the parity checker tool
//!
//! Note: These tests require network access to fetch rules from GitHub API.
//! Tests that require network are marked with #[ignore] to avoid CI failures
//! when network is unavailable. Run with `cargo test -- --ignored` to include them.

use std::process::Command;

/// Test that check_parity binary runs successfully.
/// Note: This test requires network access to GitHub API.
#[test]
#[ignore = "requires network access to GitHub API"]
fn test_check_parity_runs() {
    let output = Command::new("cargo")
        .args(&["run", "--bin", "check_parity"])
        .output()
        .expect("Failed to execute check_parity");

    assert!(
        output.status.success(),
        "check_parity should run successfully"
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Checking parity between oops and thefuck"));
}

/// Test that check_parity can output JSON format.
/// Note: This test requires network access to GitHub API.
#[test]
#[ignore = "requires network access to GitHub API"]
fn test_check_parity_json_output() {
    let output = Command::new("cargo")
        .args(&["run", "--bin", "check_parity", "--", "--output", "json"])
        .output()
        .expect("Failed to execute check_parity");

    assert!(
        output.status.success(),
        "check_parity with JSON output should run successfully"
    );

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Parse as JSON to ensure it's valid
    let json: serde_json::Value =
        serde_json::from_str(&stdout).expect("Output should be valid JSON");

    // Check expected fields
    assert!(json.get("total_thefuck_rules").is_some());
    assert!(json.get("total_oops_rules").is_some());
    assert!(json.get("coverage_percentage").is_some());
    assert!(json.get("missing_rules").is_some());
    assert!(json.get("oops_rules").is_some());
    assert!(json.get("thefuck_rules").is_some());
    assert!(json.get("name_mappings").is_some());

    // Verify we have actual data
    let thefuck_count = json["total_thefuck_rules"]
        .as_u64()
        .expect("total_thefuck_rules should be a number");
    assert!(thefuck_count > 0, "Should have found thefuck rules");

    let oops_count = json["total_oops_rules"]
        .as_u64()
        .expect("total_oops_rules should be a number");
    assert!(oops_count > 0, "Should have found oops rules");
}

/// Test that check_parity finds and reports rules.
/// Note: This test requires network access to GitHub API.
#[test]
#[ignore = "requires network access to GitHub API"]
fn test_check_parity_finds_rules() {
    let output = Command::new("cargo")
        .args(&["run", "--bin", "check_parity"])
        .output()
        .expect("Failed to execute check_parity");

    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Should report some statistics
    assert!(stdout.contains("thefuck rules:"));
    assert!(stdout.contains("oops rules:"));
    assert!(stdout.contains("Coverage:"));
    assert!(stdout.contains("Missing:"));
}

/// Test verbose output mode.
/// Note: This test requires network access to GitHub API.
#[test]
#[ignore = "requires network access to GitHub API"]
fn test_check_parity_verbose_output() {
    let output = Command::new("cargo")
        .args(&["run", "--bin", "check_parity", "--", "--verbose"])
        .output()
        .expect("Failed to execute check_parity");

    assert!(output.status.success());

    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);

    // Verbose mode should show additional information
    assert!(stderr.contains("Fetching thefuck rules from GitHub API"));
    assert!(stderr.contains("Loading oops rules from library"));
    assert!(stdout.contains("Implemented Rules"));
}

/// Test that we use get_all_rules() from library.
/// Note: This test requires network access to GitHub API.
#[test]
#[ignore = "requires network access to GitHub API"]
fn test_uses_get_all_rules_function() {
    // This test verifies that we're using get_all_rules() from the library
    // rather than scanning source files
    let output = Command::new("cargo")
        .args(&["run", "--bin", "check_parity", "--", "--verbose"])
        .output()
        .expect("Failed to execute check_parity");

    assert!(output.status.success());

    let stderr = String::from_utf8_lossy(&output.stderr);

    // Should find rules from library, not by scanning
    assert!(
        stderr.contains("Loading oops rules from library"),
        "Should use get_all_rules() from library"
    );
}

/// Test that we fetch from GitHub API.
/// Note: This test requires network access to GitHub API.
#[test]
#[ignore = "requires network access to GitHub API"]
fn test_fetches_from_github() {
    // This test verifies that we fetch thefuck rules from GitHub API
    let output = Command::new("cargo")
        .args(&["run", "--bin", "check_parity", "--", "--verbose"])
        .output()
        .expect("Failed to execute check_parity");

    assert!(output.status.success());

    let stderr = String::from_utf8_lossy(&output.stderr);

    // Should fetch from GitHub API
    assert!(
        stderr.contains("Fetching thefuck rules from GitHub API"),
        "Should fetch rules from GitHub"
    );
    assert!(
        stderr.contains("Found") && stderr.contains("thefuck rules"),
        "Should report number of thefuck rules found"
    );
}

/// Test that missing rules are categorized.
/// Note: This test requires network access to GitHub API.
#[test]
#[ignore = "requires network access to GitHub API"]
fn test_categorizes_missing_rules() {
    let output = Command::new("cargo")
        .args(&["run", "--bin", "check_parity"])
        .output()
        .expect("Failed to execute check_parity");

    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);

    // If there are missing rules, they should be categorized
    if stdout.contains("Missing Rules") {
        // Should have at least one category or say "Full parity achieved"
        assert!(
            stdout.contains("(") && stdout.contains("rules):")
                || stdout.contains("Full parity achieved"),
            "Missing rules should be categorized or show full parity"
        );
    }
}
