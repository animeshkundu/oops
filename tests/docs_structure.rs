/// Test to enforce documentation placement policy.
/// Only README.md, CLAUDE.md, and AGENT.md may exist at repository root.
/// All other markdown files should be under /docs.
use std::fs;
use std::path::Path;

#[test]
fn test_only_allowed_markdown_files_at_root() {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let root_path = Path::new(manifest_dir);

    // Allowed markdown files at root (case-insensitive)
    let allowed_files = ["readme.md", "claude.md", "agent.md"];

    // Read all entries in the repository root
    let entries = fs::read_dir(root_path)
        .expect("Failed to read repository root directory");

    let mut violations = Vec::new();

    for entry in entries {
        let entry = entry.expect("Failed to read directory entry");
        let path = entry.path();

        // Only check files (not directories)
        if path.is_file() {
            if let Some(filename) = path.file_name() {
                let filename_str = filename.to_string_lossy().to_string();
                let filename_lower = filename_str.to_lowercase();

                // Check if it's a markdown file
                if filename_lower.ends_with(".md") {
                    // Check if it's in the allowed list (case-insensitive)
                    if !allowed_files.contains(&filename_lower.as_str()) {
                        violations.push(filename_str);
                    }
                }
            }
        }
    }

    assert!(
        violations.is_empty(),
        "Found markdown files at repository root that should be in /docs:\n  - {}\n\n\
         Only README.md, CLAUDE.md, and AGENT.md are allowed at root.\n\
         All other markdown files should be under /docs.",
        violations.join("\n  - ")
    );
}
