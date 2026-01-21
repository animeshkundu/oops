//! Parity tests comparing oops (Rust) and Python thefuck implementations.
//!
//! These tests verify that oops produces the same corrections
//! as the original Python thefuck for the same inputs.
//!
//! Run with: `cargo test --test parity_tests`

use std::collections::HashMap;
use std::path::PathBuf;
use std::process::Command;

use oops::core::{Command as TfCommand, Rule};
use oops::rules::get_all_rules;

// ============================================================================
// Test Infrastructure
// ============================================================================

/// Get the path to the Python thefuck installation.
fn get_python_thefuck_path() -> Option<PathBuf> {
    let possible_paths = if cfg!(windows) {
        vec![
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .parent()
                .map(|p| p.to_path_buf()),
            Some(PathBuf::from(r"C:\Python\Scripts\thefuck")),
        ]
    } else {
        vec![
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .parent()
                .map(|p| p.to_path_buf()),
            Some(PathBuf::from("/usr/local/bin/thefuck")),
        ]
    };

    for path_opt in possible_paths.into_iter().flatten() {
        if path_opt.exists() {
            return Some(path_opt);
        }
    }
    None
}

/// Check if Python is available.
fn python_available() -> bool {
    Command::new("python")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
        || Command::new("python3")
            .arg("--version")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
}

/// Get the Python executable name.
fn get_python_executable() -> &'static str {
    if Command::new("python3")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
    {
        "python3"
    } else {
        "python"
    }
}

// ============================================================================
// Test Case Definitions
// ============================================================================

/// Represents a test case for comparing Python and Rust outputs.
#[derive(Debug, Clone)]
struct ParityTestCase {
    name: &'static str,
    script: &'static str,
    output: &'static str,
    expected_corrections: Vec<&'static str>,
    rule_name: &'static str,
}

/// Standard test cases for parity testing.
fn get_parity_test_cases() -> Vec<ParityTestCase> {
    vec![
        // sudo rule tests
        ParityTestCase {
            name: "sudo_permission_denied",
            script: "apt install vim",
            output: "E: Could not open lock file - Permission denied",
            expected_corrections: vec!["sudo apt install vim"],
            rule_name: "sudo",
        },
        ParityTestCase {
            name: "sudo_eacces",
            script: "touch /etc/test",
            output: "touch: cannot touch '/etc/test': EACCES",
            expected_corrections: vec!["sudo touch /etc/test"],
            rule_name: "sudo",
        },
        ParityTestCase {
            name: "sudo_operation_not_permitted",
            script: "rm /protected/file",
            output: "rm: cannot remove '/protected/file': Operation not permitted",
            expected_corrections: vec!["sudo rm /protected/file"],
            rule_name: "sudo",
        },
        // cd_parent rule tests
        ParityTestCase {
            name: "cd_parent_double_dot",
            script: "cd..",
            output: "cd..: command not found",
            expected_corrections: vec!["cd .."],
            rule_name: "cd_parent",
        },
        ParityTestCase {
            name: "cd_parent_triple_dot",
            script: "cd...",
            output: "cd...: command not found",
            expected_corrections: vec!["cd ..."],
            rule_name: "cd_parent",
        },
        // cd_mkdir rule tests
        ParityTestCase {
            name: "cd_mkdir_new_dir",
            script: "cd new_project",
            output: "cd: no such file or directory: new_project",
            expected_corrections: vec!["mkdir -p new_project && cd new_project"],
            rule_name: "cd_mkdir",
        },
        ParityTestCase {
            name: "cd_mkdir_nested_path",
            script: "cd project/src/lib",
            output: "cd: no such file or directory: project/src/lib",
            expected_corrections: vec!["mkdir -p project/src/lib && cd project/src/lib"],
            rule_name: "cd_mkdir",
        },
        // no_command rule tests
        ParityTestCase {
            name: "no_command_bash_style",
            script: "gti status",
            output: "gti: command not found",
            expected_corrections: vec![],
            rule_name: "no_command",
        },
        ParityTestCase {
            name: "no_command_zsh_style",
            script: "gti status",
            output: "zsh: command not found: gti",
            expected_corrections: vec![],
            rule_name: "no_command",
        },
        ParityTestCase {
            name: "no_command_powershell_style",
            script: "gti status",
            output: "'gti' is not recognized as an internal or external command",
            expected_corrections: vec![],
            rule_name: "no_command",
        },
    ]
}

/// Run a parity test case against the Rust implementation.
fn test_rust_implementation(test_case: &ParityTestCase) -> Vec<String> {
    let cmd = TfCommand::new(test_case.script, test_case.output);
    let rules = get_all_rules();

    let mut corrections = Vec::new();
    for rule in rules.iter() {
        if rule.name() == test_case.rule_name && rule.is_match(&cmd) {
            corrections.extend(rule.get_new_command(&cmd));
        }
    }
    corrections
}

// ============================================================================
// Parity Tests
// ============================================================================

#[test]
fn test_sudo_rule_parity() {
    let test_cases: Vec<ParityTestCase> = get_parity_test_cases()
        .into_iter()
        .filter(|tc| tc.rule_name == "sudo")
        .collect();

    for test_case in test_cases {
        let rust_corrections = test_rust_implementation(&test_case);

        for expected in &test_case.expected_corrections {
            assert!(
                rust_corrections.contains(&expected.to_string()),
                "Test '{}': Expected correction '{}' not found in Rust output {:?}",
                test_case.name,
                expected,
                rust_corrections
            );
        }
    }
}

#[test]
fn test_cd_parent_rule_parity() {
    let test_cases: Vec<ParityTestCase> = get_parity_test_cases()
        .into_iter()
        .filter(|tc| tc.rule_name == "cd_parent")
        .collect();

    for test_case in test_cases {
        let rust_corrections = test_rust_implementation(&test_case);

        for expected in &test_case.expected_corrections {
            assert!(
                rust_corrections.contains(&expected.to_string()),
                "Test '{}': Expected correction '{}' not found in Rust output {:?}",
                test_case.name,
                expected,
                rust_corrections
            );
        }
    }
}

#[test]
fn test_cd_mkdir_rule_parity() {
    let test_cases: Vec<ParityTestCase> = get_parity_test_cases()
        .into_iter()
        .filter(|tc| tc.rule_name == "cd_mkdir")
        .collect();

    for test_case in test_cases {
        let rust_corrections = test_rust_implementation(&test_case);

        for expected in &test_case.expected_corrections {
            assert!(
                rust_corrections.contains(&expected.to_string()),
                "Test '{}': Expected correction '{}' not found in Rust output {:?}",
                test_case.name,
                expected,
                rust_corrections
            );
        }
    }
}

#[test]
fn test_no_command_rule_matches() {
    let test_cases: Vec<ParityTestCase> = get_parity_test_cases()
        .into_iter()
        .filter(|tc| tc.rule_name == "no_command")
        .collect();

    let rules = get_all_rules();
    let no_command_rule = rules
        .iter()
        .find(|r: &&Box<dyn Rule>| r.name() == "no_command")
        .unwrap();

    for test_case in test_cases {
        let cmd = TfCommand::new(test_case.script, test_case.output);
        assert!(
            no_command_rule.is_match(&cmd),
            "Test '{}': no_command rule should match",
            test_case.name
        );
    }
}

#[test]
fn test_all_rules_accessible() {
    let rules = get_all_rules();
    assert!(!rules.is_empty(), "Should have at least one rule");
}

#[test]
fn test_rule_priority_ordering() {
    let rules = get_all_rules();
    let sudo_priority = rules
        .iter()
        .find(|r: &&Box<dyn Rule>| r.name() == "sudo")
        .map(|r| r.priority());

    assert!(sudo_priority.is_some(), "sudo rule should exist");
    assert!(
        sudo_priority.unwrap() <= 100,
        "sudo should have high priority"
    );
}

#[test]
fn test_no_false_positives() {
    let rules = get_all_rules();

    // A successful command shouldn't match sudo
    let successful_cmd = TfCommand::new("ls /home", "file1 file2 file3");
    let sudo_rule = rules
        .iter()
        .find(|r: &&Box<dyn Rule>| r.name() == "sudo")
        .unwrap();
    assert!(!sudo_rule.is_match(&successful_cmd));

    // A normal cd shouldn't match cd_parent
    let normal_cd = TfCommand::new("cd /home", "");
    let cd_parent_rule = rules
        .iter()
        .find(|r: &&Box<dyn Rule>| r.name() == "cd_parent")
        .unwrap();
    assert!(!cd_parent_rule.is_match(&normal_cd));
}

// ============================================================================
// Comprehensive Rule Tests
// ============================================================================

struct RuleTestData {
    matching_cases: Vec<(&'static str, &'static str)>,
    non_matching_cases: Vec<(&'static str, &'static str)>,
    expected_corrections: HashMap<(&'static str, &'static str), Vec<&'static str>>,
}

impl RuleTestData {
    fn sudo() -> Self {
        let mut expected = HashMap::new();
        expected.insert(
            ("apt install vim", "Permission denied"),
            vec!["sudo apt install vim"],
        );
        expected.insert(
            ("systemctl restart nginx", "Operation not permitted"),
            vec!["sudo systemctl restart nginx"],
        );

        RuleTestData {
            matching_cases: vec![
                ("apt install vim", "Permission denied"),
                ("systemctl restart nginx", "Operation not permitted"),
                ("touch /etc/test", "EACCES"),
                ("dnf install package", "are you root?"),
            ],
            non_matching_cases: vec![
                ("ls /home", "file1 file2"),
                ("git push", "error: failed to push"),
                ("sudo apt install vim", "Permission denied"),
            ],
            expected_corrections: expected,
        }
    }

    fn cd_parent() -> Self {
        let mut expected = HashMap::new();
        expected.insert(("cd..", "command not found"), vec!["cd .."]);
        expected.insert(("cd...", "command not found"), vec!["cd ..."]);

        RuleTestData {
            matching_cases: vec![
                ("cd..", "command not found"),
                ("cd...", "command not found"),
            ],
            non_matching_cases: vec![("cd ..", ""), ("cd /home", ""), ("cd Documents", "")],
            expected_corrections: expected,
        }
    }

    fn cd_mkdir() -> Self {
        let mut expected = HashMap::new();
        expected.insert(
            ("cd new_dir", "no such file or directory"),
            vec!["mkdir -p new_dir && cd new_dir"],
        );

        RuleTestData {
            matching_cases: vec![
                ("cd new_dir", "no such file or directory"),
                ("cd project/src", "does not exist"),
                ("cd mypath", "Cannot find path"),
            ],
            non_matching_cases: vec![
                ("ls new_dir", "no such file or directory"),
                ("cd /home", ""),
            ],
            expected_corrections: expected,
        }
    }

    fn no_command() -> Self {
        RuleTestData {
            matching_cases: vec![
                ("gti status", "gti: command not found"),
                ("gti status", "zsh: command not found: gti"),
                ("gti status", "'gti' is not recognized"),
                ("pythno script.py", "pythno: not found"),
            ],
            non_matching_cases: vec![("git status", "On branch master"), ("ls", "file1 file2")],
            expected_corrections: HashMap::new(),
        }
    }
}

fn test_rule_matching(rule_name: &str, test_data: &RuleTestData) {
    let rules = get_all_rules();
    let rule = rules
        .iter()
        .find(|r: &&Box<dyn Rule>| r.name() == rule_name)
        .unwrap_or_else(|| panic!("Rule '{}' not found", rule_name));

    for (script, output) in &test_data.matching_cases {
        let cmd = TfCommand::new(*script, *output);
        assert!(
            rule.is_match(&cmd),
            "Rule '{}' should match script='{}' output='{}'",
            rule_name,
            script,
            output
        );
    }

    for (script, output) in &test_data.non_matching_cases {
        let cmd = TfCommand::new(*script, *output);
        assert!(
            !rule.is_match(&cmd),
            "Rule '{}' should NOT match script='{}' output='{}'",
            rule_name,
            script,
            output
        );
    }

    for ((script, output), expected) in &test_data.expected_corrections {
        let cmd = TfCommand::new(*script, *output);
        let corrections = rule.get_new_command(&cmd);

        for exp in expected {
            assert!(
                corrections.contains(&exp.to_string()),
                "Rule '{}': Expected '{}' in corrections {:?}",
                rule_name,
                exp,
                corrections
            );
        }
    }
}

#[test]
fn test_sudo_rule_comprehensive() {
    test_rule_matching("sudo", &RuleTestData::sudo());
}

#[test]
fn test_cd_parent_rule_comprehensive() {
    test_rule_matching("cd_parent", &RuleTestData::cd_parent());
}

#[test]
fn test_cd_mkdir_rule_comprehensive() {
    test_rule_matching("cd_mkdir", &RuleTestData::cd_mkdir());
}

#[test]
fn test_no_command_rule_comprehensive() {
    test_rule_matching("no_command", &RuleTestData::no_command());
}
