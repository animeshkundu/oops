//! Tests enforcing script placement conventions.
//!
//! Ensures scripts live under /scripts and test scripts under /scripts/tests.

use std::fs;
use std::path::{Path, PathBuf};

fn is_script(path: &Path) -> bool {
    matches!(
        path.extension().and_then(|ext| ext.to_str()).map(str::to_lowercase),
        Some(ext) if ext == "sh" || ext == "ps1"
    )
}

fn is_test_script(path: &Path) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .map(|name| {
            let name = name.to_lowercase();
            name.starts_with("test-") || name.starts_with("test_")
        })
        .unwrap_or(false)
}

fn should_skip_dir(path: &Path) -> bool {
    matches!(
        path.file_name().and_then(|name| name.to_str()),
        Some(".git" | "target")
    )
}

#[test]
fn test_no_scripts_at_repo_root() {
    let root_path = Path::new(env!("CARGO_MANIFEST_DIR"));
    let entries = fs::read_dir(root_path).expect("Failed to read repository root directory");

    let mut violations = Vec::new();
    for entry in entries {
        let entry = entry.expect("Failed to read directory entry");
        let path = entry.path();
        if path.is_file() && is_script(&path) {
            if let Some(name) = path.file_name().and_then(|name| name.to_str()) {
                violations.push(name.to_string());
            }
        }
    }

    assert!(
        violations.is_empty(),
        "Script files must live under /scripts (no root-level scripts):\n  - {}",
        violations.join("\n  - ")
    );
}

#[test]
fn test_test_scripts_live_in_scripts_tests() {
    let root_path = Path::new(env!("CARGO_MANIFEST_DIR"));
    let scripts_tests = Path::new("scripts").join("tests");
    let mut pending = vec![root_path.to_path_buf()];
    let mut violations: Vec<PathBuf> = Vec::new();

    while let Some(dir) = pending.pop() {
        for entry in fs::read_dir(&dir).expect("Failed to read directory") {
            let entry = entry.expect("Failed to read directory entry");
            let path = entry.path();
            if path.is_dir() {
                if should_skip_dir(&path) {
                    continue;
                }
                pending.push(path);
                continue;
            }

            if is_script(&path) && is_test_script(&path) {
                let relative = path
                    .strip_prefix(root_path)
                    .expect("Failed to strip repository root prefix");
                if !relative.starts_with(&scripts_tests) {
                    violations.push(relative.to_path_buf());
                }
            }
        }
    }

    assert!(
        violations.is_empty(),
        "Test scripts must live under scripts/tests:\n  - {}",
        violations
            .iter()
            .map(|path| path.to_string_lossy())
            .collect::<Vec<_>>()
            .join("\n  - ")
    );
}
