//! Repository structure enforcement tests.
//!
//! These tests ensure that the repository maintains a clean structure:
//! - Documentation files (*.md) must be under /docs except README.md
//! - Scripts (*.sh, *.ps1, *.cmd, *.bat) must not be at repository root
//!   (they should be under /scripts or /.github instead)
//!
//! Run with: `cargo test --test structure_tests`

use std::fs;
use std::path::{Path, PathBuf};

/// Get the repository root directory (parent of Cargo.toml).
fn repo_root() -> PathBuf {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    PathBuf::from(manifest_dir)
}

/// Check if a path is allowed to be at the repository root.
fn is_allowed_at_root(path: &Path) -> bool {
    // Allow README.md at root
    if let Some(name) = path.file_name() {
        if name == "README.md" {
            return true;
        }
    }
    false
}

/// Find all files with the given extensions at the repository root (non-recursive).
fn find_root_files_with_extensions(extensions: &[&str]) -> Vec<PathBuf> {
    let root = repo_root();
    let mut found_files = Vec::new();

    if let Ok(entries) = fs::read_dir(&root) {
        for entry in entries.flatten() {
            let path = entry.path();

            // Skip directories and hidden files
            let file_name = match path.file_name() {
                Some(name) => name.to_string_lossy(),
                None => continue,
            };
            if path.is_dir() || file_name.starts_with('.') {
                continue;
            }

            // Check if file has one of the target extensions
            if let Some(ext) = path.extension() {
                let ext_str = ext.to_string_lossy().to_lowercase();
                if extensions.contains(&ext_str.as_str()) && !is_allowed_at_root(&path) {
                    found_files.push(path);
                }
            }
        }
    }

    found_files
}

#[test]
fn test_no_markdown_files_at_root() {
    let markdown_files = find_root_files_with_extensions(&["md"]);

    assert!(
        markdown_files.is_empty(),
        "Found markdown files at repository root that should be under /docs:\n{}",
        markdown_files
            .iter()
            .map(|p| format!(
                "  - {}",
                p.file_name()
                    .map(|n| n.to_string_lossy())
                    .unwrap_or_default()
            ))
            .collect::<Vec<_>>()
            .join("\n")
    );
}

#[test]
fn test_no_script_files_at_root() {
    let script_files = find_root_files_with_extensions(&["sh", "ps1", "cmd", "bat"]);

    assert!(
        script_files.is_empty(),
        "Found script files at repository root that should be under /scripts or /.github:\n{}",
        script_files
            .iter()
            .map(|p| format!(
                "  - {}",
                p.file_name()
                    .map(|n| n.to_string_lossy())
                    .unwrap_or_default()
            ))
            .collect::<Vec<_>>()
            .join("\n")
    );
}

#[test]
fn test_docs_directory_exists() {
    let docs_dir = repo_root().join("docs");
    assert!(
        docs_dir.exists() && docs_dir.is_dir(),
        "The /docs directory must exist"
    );
}

#[test]
fn test_scripts_directory_exists() {
    let scripts_dir = repo_root().join("scripts");
    assert!(
        scripts_dir.exists() && scripts_dir.is_dir(),
        "The /scripts directory must exist"
    );
}

#[test]
fn test_readme_exists_at_root() {
    let readme = repo_root().join("README.md");
    assert!(
        readme.exists() && readme.is_file(),
        "README.md must exist at repository root"
    );
}
