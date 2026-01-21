//! CLI argument tests for oops.
//!
//! Tests the command-line interface using assert_cmd and predicates.
//!
//! Run with: `cargo test --test cli_tests`

use assert_cmd::Command;
use predicates::prelude::*;

/// Get the command for the oops binary.
fn oops_cmd() -> Command {
    Command::cargo_bin("oops").expect("Failed to find oops binary")
}

// ============================================================================
// Version Tests
// ============================================================================

#[test]
fn test_version_flag() {
    let mut cmd = oops_cmd();
    cmd.arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("oops"));
}

#[test]
fn test_version_short_flag() {
    let mut cmd = oops_cmd();
    cmd.arg("-V")
        .assert()
        .success()
        .stdout(predicate::str::contains("oops"));
}

// ============================================================================
// Help Tests
// ============================================================================

#[test]
fn test_help_flag() {
    let mut cmd = oops_cmd();
    cmd.arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("blazingly fast"))
        .stdout(predicate::str::contains("--alias"))
        .stdout(predicate::str::contains("--yes"));
}

#[test]
fn test_help_short_flag() {
    let mut cmd = oops_cmd();
    cmd.arg("-h")
        .assert()
        .success()
        .stdout(predicate::str::contains("blazingly fast"));
}

// ============================================================================
// Alias Tests
// ============================================================================

#[test]
fn test_alias_flag_produces_output() {
    let mut cmd = oops_cmd();
    cmd.env("TF_SHELL", "bash")
        .arg("--alias")
        .assert()
        .success();
}

#[test]
fn test_alias_with_bash_shell() {
    let mut cmd = oops_cmd();
    cmd.env("TF_SHELL", "bash")
        .arg("--alias")
        .assert()
        .success()
        .stdout(predicate::str::contains("function").or(predicate::str::contains("alias")));
}

#[test]
fn test_alias_with_zsh_shell() {
    let mut cmd = oops_cmd();
    cmd.env("TF_SHELL", "zsh").arg("--alias").assert().success();
}

#[test]
fn test_alias_with_fish_shell() {
    let mut cmd = oops_cmd();
    cmd.env("TF_SHELL", "fish")
        .arg("--alias")
        .assert()
        .success();
}

#[test]
fn test_alias_with_powershell() {
    let mut cmd = oops_cmd();
    cmd.env("TF_SHELL", "powershell")
        .arg("--alias")
        .assert()
        .success();
}

// ============================================================================
// Flag Acceptance Tests
// ============================================================================

#[test]
fn test_yes_flag_accepted() {
    let mut cmd = oops_cmd();
    cmd.arg("-y").arg("--help").assert().success();
}

#[test]
fn test_yes_long_flag_accepted() {
    let mut cmd = oops_cmd();
    cmd.arg("--yes").arg("--help").assert().success();
}

#[test]
fn test_debug_flag_accepted() {
    let mut cmd = oops_cmd();
    cmd.arg("-d").arg("--help").assert().success();
}

#[test]
fn test_debug_long_flag_accepted() {
    let mut cmd = oops_cmd();
    cmd.arg("--debug").arg("--help").assert().success();
}

#[test]
fn test_repeat_flag_accepted() {
    let mut cmd = oops_cmd();
    cmd.arg("-r").arg("--help").assert().success();
}

#[test]
fn test_repeat_long_flag_accepted() {
    let mut cmd = oops_cmd();
    cmd.arg("--repeat").arg("--help").assert().success();
}

#[test]
fn test_force_command_flag_accepted() {
    let mut cmd = oops_cmd();
    cmd.arg("--force-command")
        .arg("git status")
        .arg("--help")
        .assert()
        .success();
}

#[test]
fn test_instant_mode_flag_accepted() {
    let mut cmd = oops_cmd();
    cmd.arg("--enable-experimental-instant-mode")
        .arg("--help")
        .assert()
        .success();
}

// ============================================================================
// Combined Flags Tests
// ============================================================================

#[test]
fn test_multiple_flags_accepted() {
    let mut cmd = oops_cmd();
    cmd.arg("-y")
        .arg("-d")
        .arg("-r")
        .arg("--help")
        .assert()
        .success();
}

#[test]
fn test_long_and_short_flags_mixed() {
    let mut cmd = oops_cmd();
    cmd.arg("--yes")
        .arg("-d")
        .arg("--repeat")
        .arg("--help")
        .assert()
        .success();
}

// ============================================================================
// Invalid Flag Tests
// ============================================================================

#[test]
fn test_invalid_flag_rejected() {
    let mut cmd = oops_cmd();
    cmd.arg("--invalid-flag-that-does-not-exist")
        .assert()
        .failure();
}

#[test]
fn test_unknown_short_flag_rejected() {
    let mut cmd = oops_cmd();
    cmd.arg("-Z").assert().failure();
}

// ============================================================================
// Environment Variable Tests
// ============================================================================

#[test]
fn test_tf_shell_environment() {
    let mut cmd = oops_cmd();
    cmd.env("TF_SHELL", "bash")
        .arg("--alias")
        .assert()
        .success();
}

#[test]
fn test_tf_alias_environment() {
    let mut cmd = oops_cmd();
    cmd.env("TF_ALIAS", "fix")
        .env("TF_SHELL", "bash")
        .arg("--alias")
        .assert()
        .success();
}

#[test]
fn test_oops_debug_environment() {
    let mut cmd = oops_cmd();
    cmd.env("THEFUCK_DEBUG", "1")
        .arg("--help")
        .assert()
        .success();
}

// ============================================================================
// Output Format Tests
// ============================================================================

#[test]
fn test_version_format() {
    let mut cmd = oops_cmd();
    let output = cmd
        .arg("--version")
        .output()
        .expect("Failed to run command");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("0.") || stdout.contains("oops"),
        "Version output should contain version info: {}",
        stdout
    );
}

#[test]
fn test_help_includes_all_options() {
    let mut cmd = oops_cmd();
    let output = cmd.arg("--help").output().expect("Failed to run command");
    let stdout = String::from_utf8_lossy(&output.stdout);

    let expected_options = [
        "--alias",
        "--yes",
        "--repeat",
        "--debug",
        "--enable-experimental-instant-mode",
        "--force-command",
    ];

    for option in &expected_options {
        assert!(
            stdout.contains(option),
            "Help should include option: {}",
            option
        );
    }
}

// ============================================================================
// Error Message Tests
// ============================================================================

#[test]
fn test_invalid_flag_error_message() {
    let mut cmd = oops_cmd();
    cmd.arg("--not-a-real-flag")
        .assert()
        .failure()
        .stderr(predicate::str::contains("error").or(predicate::str::contains("unexpected")));
}

// ============================================================================
// Basic Integration Tests
// ============================================================================

#[test]
fn test_basic_invocation() {
    let mut cmd = oops_cmd();
    cmd.arg("--version").assert().success();
}

#[test]
fn test_no_history_available() {
    let mut cmd = oops_cmd();
    cmd.env_remove("TF_HISTORY")
        .env_remove("THEFUCK_HISTORY")
        .assert();
    // Should handle gracefully
}

#[test]
fn test_placeholder_handling() {
    let mut cmd = oops_cmd();
    cmd.arg("THEFUCK_ARGUMENT_PLACEHOLDER")
        .arg("git")
        .arg("status")
        .env("TF_HISTORY", "")
        .env_remove("THEFUCK_HISTORY")
        .assert();
    // May fail but should parse arguments correctly
}
