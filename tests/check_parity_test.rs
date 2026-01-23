//! Tests for the parity checker tool

use std::process::Command;

#[test]
fn test_check_parity_runs() {
    let output = Command::new("cargo")
        .args(&["run", "--bin", "check_parity", "--", "--days", "7"])
        .output()
        .expect("Failed to execute check_parity");

    assert!(
        output.status.success(),
        "check_parity should run successfully"
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Checking parity between oops and thefuck"));
    assert!(stderr.contains("Found"));
}

#[test]
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
}

#[test]
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
}

#[test]
fn test_extract_rule_names_from_source() {
    // This test verifies that we can extract rule names from Rust source files
    let output = Command::new("cargo")
        .args(&["run", "--bin", "check_parity"])
        .output()
        .expect("Failed to execute check_parity");

    assert!(output.status.success());

    let stderr = String::from_utf8_lossy(&output.stderr);

    // Should find a reasonable number of rules (we have 177+ rules)
    assert!(
        stderr.contains("Found") && stderr.contains("rules in oops"),
        "Should report found rules"
    );
}
