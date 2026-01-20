//! Development tool rules (Go, Java, Maven, Gradle, Terraform, etc.)
//!
//! Contains rules for:
//! - Go: `go_run`, `go_unknown_command`
//! - Gradle: `gradle_no_task`, `gradle_wrapper`
//! - Java: `java`, `javac`
//! - Maven: `mvn_no_command`, `mvn_unknown_lifecycle_phase`
//! - PHP Composer: `composer_not_command`
//! - C++: `cpp11`
//! - Python Fabric: `fab_command_not_found`
//! - Node.js: `grunt_task_not_found`, `gulp_not_task`
//! - Clojure: `lein_not_task`
//! - Terraform: `terraform_init`, `terraform_no_command`

use crate::core::{is_app, Command, Rule};
use crate::utils::{get_close_matches, get_closest, replace_argument};
use regex::Regex;

// ============================================================================
// Go Rules
// ============================================================================

/// Rule to append `.go` extension when running `go run` without it.
///
/// Matches errors like:
/// - `go run foo` (should be `go run foo.go`)
///
/// # Example
///
/// ```text
/// > go run foo
/// error: go run: no go files listed
/// ```
#[derive(Debug, Clone, Copy, Default)]
pub struct GoRun;

impl Rule for GoRun {
    fn name(&self) -> &str {
        "go_run"
    }

    fn is_match(&self, command: &Command) -> bool {
        if !is_app(command, &["go"]) {
            return false;
        }

        command.script.starts_with("go run ") && !command.script.ends_with(".go")
    }

    fn get_new_command(&self, command: &Command) -> Vec<String> {
        vec![format!("{}.go", command.script)]
    }

    fn requires_output(&self) -> bool {
        false
    }
}

/// Rule to suggest correct Go subcommands when an unknown command is used.
///
/// Matches errors like:
/// - `go buidl` -> suggests `go build`
///
/// # Example
///
/// ```text
/// > go buidl
/// go buidl: unknown command
/// ```
#[derive(Debug, Clone, Copy, Default)]
pub struct GoUnknownCommand;

impl GoUnknownCommand {
    /// Common Go subcommands for fuzzy matching.
    const GO_COMMANDS: &'static [&'static str] = &[
        "bug", "build", "clean", "doc", "env", "fix", "fmt", "generate", "get", "help", "install",
        "list", "mod", "work", "run", "test", "tool", "version", "vet",
    ];
}

impl Rule for GoUnknownCommand {
    fn name(&self) -> &str {
        "go_unknown_command"
    }

    fn is_match(&self, command: &Command) -> bool {
        if !is_app(command, &["go"]) {
            return false;
        }

        command.output.contains("unknown command")
    }

    fn get_new_command(&self, command: &Command) -> Vec<String> {
        let parts = command.script_parts();
        if parts.len() < 2 {
            return vec![];
        }

        let broken = &parts[1];
        let go_commands: Vec<String> = Self::GO_COMMANDS.iter().map(|s| s.to_string()).collect();

        if let Some(closest) = get_closest(broken, &go_commands, 0.6, false) {
            return vec![replace_argument(&command.script, broken, &closest)];
        }

        vec![]
    }
}

// ============================================================================
// Gradle Rules
// ============================================================================

/// Rule to suggest correct Gradle task names when a task is not found.
///
/// Matches errors like:
/// - `Task 'complie' not found in root project`
///
/// # Example
///
/// ```text
/// > gradle complie
/// FAILURE: Build failed with an exception.
/// * What went wrong:
/// Task 'complie' not found in root project
/// ```
#[derive(Debug, Clone, Copy, Default)]
pub struct GradleNoTask;

impl GradleNoTask {
    /// Common Gradle tasks for fuzzy matching.
    const GRADLE_TASKS: &'static [&'static str] = &[
        "assemble",
        "build",
        "buildDependents",
        "buildNeeded",
        "classes",
        "clean",
        "jar",
        "testClasses",
        "test",
        "check",
        "javadoc",
        "dependencies",
        "dependencyInsight",
        "help",
        "projects",
        "properties",
        "tasks",
        "wrapper",
        "init",
        "compileJava",
        "compileTestJava",
        "processResources",
        "processTestResources",
        "run",
        "bootRun",
        "bootJar",
    ];

    /// Extract the wrong task name from Gradle output.
    fn get_wrong_task(output: &str) -> Option<String> {
        let re = Regex::new(r"Task '([^']*)' (?:is ambiguous|not found)").ok()?;
        let caps = re.captures(output)?;
        caps.get(1).map(|m| m.as_str().to_string())
    }
}

impl Rule for GradleNoTask {
    fn name(&self) -> &str {
        "gradle_no_task"
    }

    fn is_match(&self, command: &Command) -> bool {
        if !is_app(command, &["gradle", "gradlew", "gradlew.bat"]) {
            return false;
        }

        Self::get_wrong_task(&command.output).is_some()
    }

    fn get_new_command(&self, command: &Command) -> Vec<String> {
        let wrong_task = match Self::get_wrong_task(&command.output) {
            Some(t) => t,
            None => return vec![],
        };

        let gradle_tasks: Vec<String> = Self::GRADLE_TASKS.iter().map(|s| s.to_string()).collect();

        let matches = get_close_matches(&wrong_task, &gradle_tasks, 3, 0.6);
        matches
            .into_iter()
            .map(|fixed| replace_argument(&command.script, &wrong_task, &fixed))
            .collect()
    }
}

/// Rule to suggest `./gradlew` when `gradle` is not installed but wrapper exists.
///
/// Matches when:
/// - `gradle` command not found
/// - `gradlew` wrapper exists in current directory
///
/// # Example
///
/// ```text
/// > gradle build
/// gradle: command not found
/// ```
#[derive(Debug, Clone, Copy, Default)]
pub struct GradleWrapper;

impl Rule for GradleWrapper {
    fn name(&self) -> &str {
        "gradle_wrapper"
    }

    fn is_match(&self, command: &Command) -> bool {
        let parts = command.script_parts();
        if parts.is_empty() || parts[0] != "gradle" {
            return false;
        }

        // Check if gradle is not found and wrapper might exist
        let output_lower = command.output.to_lowercase();
        (output_lower.contains("not found")
            || output_lower.contains("not recognized")
            || output_lower.contains("is not recognized"))
            && (std::path::Path::new("gradlew").exists()
                || std::path::Path::new("gradlew.bat").exists())
    }

    fn get_new_command(&self, command: &Command) -> Vec<String> {
        let parts = command.script_parts();
        let args: String = if parts.len() > 1 {
            parts[1..].join(" ")
        } else {
            String::new()
        };

        // Use platform-appropriate wrapper
        #[cfg(windows)]
        let wrapper = "gradlew.bat";
        #[cfg(not(windows))]
        let wrapper = "./gradlew";

        if args.is_empty() {
            vec![wrapper.to_string()]
        } else {
            vec![format!("{} {}", wrapper, args)]
        }
    }
}

// ============================================================================
// Java Rules
// ============================================================================

/// Rule to remove `.java` extension when running `java` command.
///
/// Fixes the common mistake of running `java foo.java` instead of `java foo`.
///
/// # Example
///
/// ```text
/// > java foo.java
/// Error: Could not find or load main class foo.java
/// ```
#[derive(Debug, Clone, Copy, Default)]
pub struct Java;

impl Rule for Java {
    fn name(&self) -> &str {
        "java"
    }

    fn is_match(&self, command: &Command) -> bool {
        if !is_app(command, &["java"]) {
            return false;
        }

        command.script.ends_with(".java")
    }

    fn get_new_command(&self, command: &Command) -> Vec<String> {
        // Remove the .java extension
        let fixed = command
            .script
            .strip_suffix(".java")
            .unwrap_or(&command.script);
        vec![fixed.to_string()]
    }

    fn requires_output(&self) -> bool {
        false
    }
}

/// Rule to append `.java` extension when compiling with `javac`.
///
/// Fixes the mistake of running `javac foo` instead of `javac foo.java`.
///
/// # Example
///
/// ```text
/// > javac foo
/// error: Class names, 'foo', are only accepted if annotation
/// processing is explicitly requested
/// ```
#[derive(Debug, Clone, Copy, Default)]
pub struct Javac;

impl Rule for Javac {
    fn name(&self) -> &str {
        "javac"
    }

    fn is_match(&self, command: &Command) -> bool {
        if !is_app(command, &["javac"]) {
            return false;
        }

        !command.script.ends_with(".java")
    }

    fn get_new_command(&self, command: &Command) -> Vec<String> {
        vec![format!("{}.java", command.script)]
    }

    fn requires_output(&self) -> bool {
        false
    }
}

// ============================================================================
// Maven Rules
// ============================================================================

/// Rule to suggest common Maven goals when no goal is specified.
///
/// Matches errors like:
/// - `No goals have been specified for this build`
///
/// # Example
///
/// ```text
/// > mvn
/// [ERROR] No goals have been specified for this build.
/// ```
#[derive(Debug, Clone, Copy, Default)]
pub struct MvnNoCommand;

impl Rule for MvnNoCommand {
    fn name(&self) -> &str {
        "mvn_no_command"
    }

    fn is_match(&self, command: &Command) -> bool {
        if !is_app(command, &["mvn"]) {
            return false;
        }

        command
            .output
            .contains("No goals have been specified for this build")
    }

    fn get_new_command(&self, command: &Command) -> Vec<String> {
        vec![
            format!("{} clean package", command.script),
            format!("{} clean install", command.script),
        ]
    }
}

/// Rule to fix unknown Maven lifecycle phase errors.
///
/// Matches errors like:
/// - `Unknown lifecycle phase "comiple"`
///
/// # Example
///
/// ```text
/// > mvn comiple
/// [ERROR] Unknown lifecycle phase "comiple". You must specify a valid
/// lifecycle phase or a goal...
/// ```
#[derive(Debug, Clone, Copy, Default)]
pub struct MvnUnknownLifecyclePhase;

impl MvnUnknownLifecyclePhase {
    /// Extract the failed lifecycle phase from Maven output.
    fn get_failed_lifecycle(output: &str) -> Option<String> {
        let re = Regex::new(r#"\[ERROR\] Unknown lifecycle phase "([^"]+)""#).ok()?;
        let caps = re.captures(output)?;
        caps.get(1).map(|m| m.as_str().to_string())
    }

    /// Extract available lifecycle phases from Maven output.
    fn get_available_lifecycles(output: &str) -> Option<Vec<String>> {
        let re = Regex::new(r"Available lifecycle phases are: ([^>]+) -> \[Help 1\]").ok()?;
        let caps = re.captures(output)?;
        let phases_str = caps.get(1)?.as_str();
        Some(
            phases_str
                .split(", ")
                .map(|s| s.trim().to_string())
                .collect(),
        )
    }
}

impl Rule for MvnUnknownLifecyclePhase {
    fn name(&self) -> &str {
        "mvn_unknown_lifecycle_phase"
    }

    fn is_match(&self, command: &Command) -> bool {
        if !is_app(command, &["mvn"]) {
            return false;
        }

        Self::get_failed_lifecycle(&command.output).is_some()
            && Self::get_available_lifecycles(&command.output).is_some()
    }

    fn get_new_command(&self, command: &Command) -> Vec<String> {
        let failed = match Self::get_failed_lifecycle(&command.output) {
            Some(f) => f,
            None => return vec![],
        };

        let available = match Self::get_available_lifecycles(&command.output) {
            Some(a) => a,
            None => return vec![],
        };

        let matches = get_close_matches(&failed, &available, 3, 0.6);
        matches
            .into_iter()
            .map(|fixed| replace_argument(&command.script, &failed, &fixed))
            .collect()
    }
}

// ============================================================================
// PHP Composer Rule
// ============================================================================

/// Rule to fix PHP Composer command errors.
///
/// Matches errors like:
/// - `Command "instal" is not defined. Did you mean this?`
/// - Using `install` when `require` should be used
///
/// # Example
///
/// ```text
/// > composer instal
/// Command "instal" is not defined.
/// Did you mean this?
///     install
/// ```
#[derive(Debug, Clone, Copy, Default)]
pub struct ComposerNotCommand;

impl ComposerNotCommand {
    /// Extract the broken command from Composer output.
    fn get_broken_command(output: &str) -> Option<String> {
        let re = Regex::new(r#"Command "([^"]*)" is not defined"#).ok()?;
        let caps = re.captures(output)?;
        caps.get(1).map(|m| m.as_str().to_string())
    }

    /// Extract the suggested command from Composer output.
    fn get_suggested_command(output: &str) -> Option<String> {
        // Try "Did you mean this?" first
        let re1 = Regex::new(r"Did you mean this\?[^\n]*\n\s*([^\n]*)").ok()?;
        if let Some(caps) = re1.captures(output) {
            if let Some(m) = caps.get(1) {
                return Some(m.as_str().trim().to_string());
            }
        }

        // Try "Did you mean one of these?" next
        let re2 = Regex::new(r"Did you mean one of these\?[^\n]*\n\s*([^\n]*)").ok()?;
        if let Some(caps) = re2.captures(output) {
            if let Some(m) = caps.get(1) {
                return Some(m.as_str().trim().to_string());
            }
        }

        None
    }
}

impl Rule for ComposerNotCommand {
    fn name(&self) -> &str {
        "composer_not_command"
    }

    fn is_match(&self, command: &Command) -> bool {
        if !is_app(command, &["composer"]) {
            return false;
        }

        let output_lower = command.output.to_lowercase();

        // Check for "did you mean" suggestions
        if output_lower.contains("did you mean this?")
            || output_lower.contains("did you mean one of these?")
        {
            return true;
        }

        // Check for install -> require suggestion
        let parts = command.script_parts();
        if parts.iter().any(|p| p == "install") && output_lower.contains("composer require") {
            return true;
        }

        false
    }

    fn get_new_command(&self, command: &Command) -> Vec<String> {
        let parts = command.script_parts();
        let output_lower = command.output.to_lowercase();

        // Handle install -> require case
        if parts.iter().any(|p| p == "install") && output_lower.contains("composer require") {
            return vec![replace_argument(&command.script, "install", "require")];
        }

        // Handle "did you mean" case
        let broken = match Self::get_broken_command(&command.output) {
            Some(b) => b,
            None => return vec![],
        };

        let suggested = match Self::get_suggested_command(&command.output) {
            Some(s) => s,
            None => return vec![],
        };

        vec![replace_argument(&command.script, &broken, &suggested)]
    }
}

// ============================================================================
// C++ Rule
// ============================================================================

/// Rule to suggest `-std=c++11` flag for C++11 features.
///
/// Matches errors about C++11 features not being enabled.
///
/// # Example
///
/// ```text
/// > g++ main.cpp
/// error: This file requires compiler and library support for the
/// ISO C++ 2011 standard.
/// ```
#[derive(Debug, Clone, Copy, Default)]
pub struct Cpp11;

impl Rule for Cpp11 {
    fn name(&self) -> &str {
        "cpp11"
    }

    fn is_match(&self, command: &Command) -> bool {
        if !is_app(command, &["g++", "clang++", "c++"]) {
            return false;
        }

        command.output.contains(
            "This file requires compiler and library support for the ISO C++ 2011 standard",
        ) || command.output.contains("-Wc++11-extensions")
    }

    fn get_new_command(&self, command: &Command) -> Vec<String> {
        vec![format!("{} -std=c++11", command.script)]
    }
}

// ============================================================================
// Python Fabric Rule
// ============================================================================

/// Rule to fix Python Fabric command not found errors.
///
/// Matches errors like:
/// - `Warning: Command(s) not found:`
///
/// # Example
///
/// ```text
/// > fab deply
/// Warning: Command(s) not found:
///     deply
/// Available commands:
///     deploy
/// ```
#[derive(Debug, Clone, Copy, Default)]
pub struct FabCommandNotFound;

impl FabCommandNotFound {
    /// Extract commands between two markers in the output.
    fn get_between<'a>(content: &'a str, start: &str, end: Option<&str>) -> Vec<&'a str> {
        let mut result = Vec::new();
        let mut should_yield = false;

        for line in content.lines() {
            if line.contains(start) {
                should_yield = true;
                continue;
            }

            if let Some(end_marker) = end {
                if line.contains(end_marker) {
                    break;
                }
            }

            if should_yield && !line.is_empty() {
                if let Some(cmd) = line.trim().split_whitespace().next() {
                    result.push(cmd);
                }
            }
        }

        result
    }
}

impl Rule for FabCommandNotFound {
    fn name(&self) -> &str {
        "fab_command_not_found"
    }

    fn is_match(&self, command: &Command) -> bool {
        if !is_app(command, &["fab"]) {
            return false;
        }

        command.output.contains("Warning: Command(s) not found:")
    }

    fn get_new_command(&self, command: &Command) -> Vec<String> {
        let not_found = Self::get_between(
            &command.output,
            "Warning: Command(s) not found:",
            Some("Available commands:"),
        );

        let available = Self::get_between(&command.output, "Available commands:", None);

        if not_found.is_empty() || available.is_empty() {
            return vec![];
        }

        let available_strings: Vec<String> = available.iter().map(|s| s.to_string()).collect();

        let mut script = command.script.clone();
        for nf in not_found {
            if let Some(fix) = get_closest(nf, &available_strings, 0.6, false) {
                script = script.replace(&format!(" {}", nf), &format!(" {}", fix));
            }
        }

        if script != command.script {
            vec![script]
        } else {
            vec![]
        }
    }
}

// ============================================================================
// Node.js Grunt Rule
// ============================================================================

/// Rule to fix Grunt task not found errors.
///
/// Matches errors like:
/// - `Warning: Task "biuld" not found.`
///
/// # Example
///
/// ```text
/// > grunt biuld
/// Warning: Task "biuld" not found.
/// ```
#[derive(Debug, Clone, Copy, Default)]
pub struct GruntTaskNotFound;

impl GruntTaskNotFound {
    /// Common Grunt tasks for fuzzy matching.
    const GRUNT_TASKS: &'static [&'static str] = &[
        "build",
        "test",
        "default",
        "watch",
        "serve",
        "clean",
        "copy",
        "concat",
        "uglify",
        "cssmin",
        "htmlmin",
        "jshint",
        "eslint",
        "sass",
        "less",
        "compass",
        "coffee",
        "typescript",
        "imagemin",
        "connect",
        "concurrent",
        "newer",
    ];

    /// Extract the misspelled task from Grunt output.
    fn get_wrong_task(output: &str) -> Option<String> {
        let re = Regex::new(r#"Warning: Task "([^"]*)" not found"#).ok()?;
        let caps = re.captures(output)?;
        let task = caps.get(1)?.as_str();
        // Handle task:target format - just get the task name
        Some(task.split(':').next().unwrap_or(task).to_string())
    }
}

impl Rule for GruntTaskNotFound {
    fn name(&self) -> &str {
        "grunt_task_not_found"
    }

    fn is_match(&self, command: &Command) -> bool {
        if !is_app(command, &["grunt"]) {
            return false;
        }

        Self::get_wrong_task(&command.output).is_some()
    }

    fn get_new_command(&self, command: &Command) -> Vec<String> {
        let wrong_task = match Self::get_wrong_task(&command.output) {
            Some(t) => t,
            None => return vec![],
        };

        let grunt_tasks: Vec<String> = Self::GRUNT_TASKS.iter().map(|s| s.to_string()).collect();

        if let Some(fixed) = get_closest(&wrong_task, &grunt_tasks, 0.6, false) {
            let new_script = command
                .script
                .replace(&format!(" {}", wrong_task), &format!(" {}", fixed));
            if new_script != command.script {
                return vec![new_script];
            }
        }

        vec![]
    }
}

// ============================================================================
// Node.js Gulp Rule
// ============================================================================

/// Rule to fix Gulp task not found errors.
///
/// Matches errors like:
/// - `Task 'biuld' is not in your gulpfile`
///
/// # Example
///
/// ```text
/// > gulp biuld
/// [gulp] Task 'biuld' is not in your gulpfile
/// ```
#[derive(Debug, Clone, Copy, Default)]
pub struct GulpNotTask;

impl GulpNotTask {
    /// Common Gulp tasks for fuzzy matching.
    const GULP_TASKS: &'static [&'static str] = &[
        "build", "default", "watch", "serve", "clean", "test", "lint", "scripts", "styles",
        "images", "fonts", "html", "copy", "dev", "prod", "deploy",
    ];

    /// Extract the wrong task from Gulp output.
    fn get_wrong_task(output: &str) -> Option<String> {
        let re = Regex::new(r"Task '(\w+)' is not in your gulpfile").ok()?;
        let caps = re.captures(output)?;
        caps.get(1).map(|m| m.as_str().to_string())
    }
}

impl Rule for GulpNotTask {
    fn name(&self) -> &str {
        "gulp_not_task"
    }

    fn is_match(&self, command: &Command) -> bool {
        if !is_app(command, &["gulp"]) {
            return false;
        }

        command.output.contains("is not in your gulpfile")
    }

    fn get_new_command(&self, command: &Command) -> Vec<String> {
        let wrong_task = match Self::get_wrong_task(&command.output) {
            Some(t) => t,
            None => return vec![],
        };

        let gulp_tasks: Vec<String> = Self::GULP_TASKS.iter().map(|s| s.to_string()).collect();

        let matches = get_close_matches(&wrong_task, &gulp_tasks, 3, 0.6);
        matches
            .into_iter()
            .map(|fixed| replace_argument(&command.script, &wrong_task, &fixed))
            .collect()
    }
}

// ============================================================================
// Clojure Leiningen Rule
// ============================================================================

/// Rule to fix Leiningen task not found errors.
///
/// Matches errors like:
/// - `'repl' is not a task. See 'lein help'.`
///
/// # Example
///
/// ```text
/// > lein repls
/// 'repls' is not a task. See 'lein help'.
/// Did you mean this?
///          repl
/// ```
#[derive(Debug, Clone, Copy, Default)]
pub struct LeinNotTask;

impl LeinNotTask {
    /// Extract the broken task from Leiningen output.
    fn get_broken_task(output: &str) -> Option<String> {
        let re = Regex::new(r"'([^']*)' is not a task").ok()?;
        let caps = re.captures(output)?;
        caps.get(1).map(|m| m.as_str().to_string())
    }

    /// Extract suggested tasks from Leiningen output.
    fn get_suggested_tasks(output: &str) -> Vec<String> {
        let mut result = Vec::new();
        let mut in_suggestions = false;

        for line in output.lines() {
            if line.contains("Did you mean this?") {
                in_suggestions = true;
                continue;
            }

            if in_suggestions {
                let trimmed = line.trim();
                if trimmed.is_empty() {
                    break;
                }
                if let Some(task) = trimmed.split_whitespace().next() {
                    result.push(task.to_string());
                }
            }
        }

        result
    }
}

impl Rule for LeinNotTask {
    fn name(&self) -> &str {
        "lein_not_task"
    }

    fn is_match(&self, command: &Command) -> bool {
        if !is_app(command, &["lein"]) {
            return false;
        }

        command.output.contains("is not a task. See 'lein help'")
            && command.output.contains("Did you mean this?")
    }

    fn get_new_command(&self, command: &Command) -> Vec<String> {
        let broken = match Self::get_broken_task(&command.output) {
            Some(b) => b,
            None => return vec![],
        };

        let suggested = Self::get_suggested_tasks(&command.output);
        if suggested.is_empty() {
            return vec![];
        }

        suggested
            .into_iter()
            .map(|fixed| replace_argument(&command.script, &broken, &fixed))
            .collect()
    }
}

// ============================================================================
// Terraform Rules
// ============================================================================

/// Rule to suggest running `terraform init` first.
///
/// Matches errors like:
/// - `This module is not yet installed`
/// - `Initialization required`
///
/// # Example
///
/// ```text
/// > terraform plan
/// Error: Module not installed
/// This module is not yet installed.
/// ```
#[derive(Debug, Clone, Copy, Default)]
pub struct TerraformInit;

impl Rule for TerraformInit {
    fn name(&self) -> &str {
        "terraform_init"
    }

    fn is_match(&self, command: &Command) -> bool {
        if !is_app(command, &["terraform"]) {
            return false;
        }

        let output_lower = command.output.to_lowercase();
        output_lower.contains("this module is not yet installed")
            || output_lower.contains("initialization required")
    }

    fn get_new_command(&self, command: &Command) -> Vec<String> {
        // Run init first, then the original command
        vec![format!("terraform init && {}", command.script)]
    }
}

/// Rule to fix unknown Terraform command errors.
///
/// Matches errors like:
/// - `Terraform has no command named "plna". Did you mean "plan"?`
///
/// # Example
///
/// ```text
/// > terraform plna
/// Terraform has no command named "plna".
/// Did you mean "plan"?
/// ```
#[derive(Debug, Clone, Copy, Default)]
pub struct TerraformNoCommand;

impl TerraformNoCommand {
    /// Extract the mistaken command from Terraform output.
    fn get_mistake(output: &str) -> Option<String> {
        let re = Regex::new(r#"Terraform has no command named "([^"]+)""#).ok()?;
        let caps = re.captures(output)?;
        caps.get(1).map(|m| m.as_str().to_string())
    }

    /// Extract the suggested fix from Terraform output.
    fn get_fix(output: &str) -> Option<String> {
        let re = Regex::new(r#"Did you mean "([^"]+)"\?"#).ok()?;
        let caps = re.captures(output)?;
        caps.get(1).map(|m| m.as_str().to_string())
    }
}

impl Rule for TerraformNoCommand {
    fn name(&self) -> &str {
        "terraform_no_command"
    }

    fn is_match(&self, command: &Command) -> bool {
        if !is_app(command, &["terraform"]) {
            return false;
        }

        Self::get_mistake(&command.output).is_some() && Self::get_fix(&command.output).is_some()
    }

    fn get_new_command(&self, command: &Command) -> Vec<String> {
        let mistake = match Self::get_mistake(&command.output) {
            Some(m) => m,
            None => return vec![],
        };

        let fix = match Self::get_fix(&command.output) {
            Some(f) => f,
            None => return vec![],
        };

        vec![command.script.replace(&mistake, &fix)]
    }
}

// ============================================================================
// All Rules Function
// ============================================================================

/// Returns all development tool rules as boxed trait objects.
///
/// This function creates instances of all devtools rules
/// for registration with the rule system.
pub fn all_rules() -> Vec<Box<dyn Rule>> {
    vec![
        // Go rules
        Box::new(GoRun),
        Box::new(GoUnknownCommand),
        // Gradle rules
        Box::new(GradleNoTask),
        Box::new(GradleWrapper),
        // Java rules
        Box::new(Java),
        Box::new(Javac),
        // Maven rules
        Box::new(MvnNoCommand),
        Box::new(MvnUnknownLifecyclePhase),
        // PHP Composer
        Box::new(ComposerNotCommand),
        // C++
        Box::new(Cpp11),
        // Python Fabric
        Box::new(FabCommandNotFound),
        // Node.js
        Box::new(GruntTaskNotFound),
        Box::new(GulpNotTask),
        // Clojure
        Box::new(LeinNotTask),
        // Terraform
        Box::new(TerraformInit),
        Box::new(TerraformNoCommand),
    ]
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // ------------------------------------------------------------------------
    // Go Rules Tests
    // ------------------------------------------------------------------------

    mod go_run_tests {
        use super::*;

        #[test]
        fn test_name() {
            assert_eq!(GoRun.name(), "go_run");
        }

        #[test]
        fn test_matches_go_run_without_extension() {
            let cmd = Command::new("go run foo", "go run: no go files listed");
            assert!(GoRun.is_match(&cmd));
        }

        #[test]
        fn test_no_match_with_extension() {
            let cmd = Command::new("go run foo.go", "");
            assert!(!GoRun.is_match(&cmd));
        }

        #[test]
        fn test_no_match_other_go_command() {
            let cmd = Command::new("go build foo", "");
            assert!(!GoRun.is_match(&cmd));
        }

        #[test]
        fn test_get_new_command() {
            let cmd = Command::new("go run foo", "go run: no go files listed");
            let fixes = GoRun.get_new_command(&cmd);
            assert_eq!(fixes, vec!["go run foo.go"]);
        }

        #[test]
        fn test_requires_no_output() {
            assert!(!GoRun.requires_output());
        }
    }

    mod go_unknown_command_tests {
        use super::*;

        #[test]
        fn test_name() {
            assert_eq!(GoUnknownCommand.name(), "go_unknown_command");
        }

        #[test]
        fn test_matches_unknown_command() {
            let cmd = Command::new("go buidl", "go buidl: unknown command");
            assert!(GoUnknownCommand.is_match(&cmd));
        }

        #[test]
        fn test_no_match_valid_command() {
            let cmd = Command::new("go build", "");
            assert!(!GoUnknownCommand.is_match(&cmd));
        }

        #[test]
        fn test_no_match_other_tool() {
            let cmd = Command::new("npm buidl", "unknown command");
            assert!(!GoUnknownCommand.is_match(&cmd));
        }

        #[test]
        fn test_get_new_command() {
            let cmd = Command::new("go buidl", "go buidl: unknown command");
            let fixes = GoUnknownCommand.get_new_command(&cmd);
            assert_eq!(fixes, vec!["go build"]);
        }
    }

    // ------------------------------------------------------------------------
    // Gradle Rules Tests
    // ------------------------------------------------------------------------

    mod gradle_no_task_tests {
        use super::*;

        #[test]
        fn test_name() {
            assert_eq!(GradleNoTask.name(), "gradle_no_task");
        }

        #[test]
        fn test_matches_task_not_found() {
            let cmd = Command::new("gradle complie", "Task 'complie' not found in root project");
            assert!(GradleNoTask.is_match(&cmd));
        }

        #[test]
        fn test_matches_task_ambiguous() {
            let cmd = Command::new("gradle test", "Task 'test' is ambiguous in root project");
            assert!(GradleNoTask.is_match(&cmd));
        }

        #[test]
        fn test_no_match_valid_task() {
            let cmd = Command::new("gradle build", "BUILD SUCCESSFUL");
            assert!(!GradleNoTask.is_match(&cmd));
        }

        #[test]
        fn test_get_new_command() {
            let cmd = Command::new("gradle complie", "Task 'complie' not found in root project");
            let fixes = GradleNoTask.get_new_command(&cmd);
            // Should suggest compile-related tasks
            assert!(!fixes.is_empty());
        }
    }

    mod gradle_wrapper_tests {
        use super::*;

        #[test]
        fn test_name() {
            assert_eq!(GradleWrapper.name(), "gradle_wrapper");
        }

        #[test]
        fn test_no_match_gradle_found() {
            let cmd = Command::new("gradle build", "BUILD SUCCESSFUL");
            assert!(!GradleWrapper.is_match(&cmd));
        }

        #[test]
        fn test_no_match_other_tool() {
            let cmd = Command::new("maven build", "command not found");
            assert!(!GradleWrapper.is_match(&cmd));
        }
    }

    // ------------------------------------------------------------------------
    // Java Rules Tests
    // ------------------------------------------------------------------------

    mod java_tests {
        use super::*;

        #[test]
        fn test_name() {
            assert_eq!(Java.name(), "java");
        }

        #[test]
        fn test_matches_java_extension() {
            let cmd = Command::new(
                "java foo.java",
                "Error: Could not find or load main class foo.java",
            );
            assert!(Java.is_match(&cmd));
        }

        #[test]
        fn test_no_match_without_extension() {
            let cmd = Command::new("java foo", "");
            assert!(!Java.is_match(&cmd));
        }

        #[test]
        fn test_no_match_other_tool() {
            let cmd = Command::new("javac foo.java", "");
            assert!(!Java.is_match(&cmd));
        }

        #[test]
        fn test_get_new_command() {
            let cmd = Command::new(
                "java foo.java",
                "Error: Could not find or load main class foo.java",
            );
            let fixes = Java.get_new_command(&cmd);
            assert_eq!(fixes, vec!["java foo"]);
        }

        #[test]
        fn test_requires_no_output() {
            assert!(!Java.requires_output());
        }
    }

    mod javac_tests {
        use super::*;

        #[test]
        fn test_name() {
            assert_eq!(Javac.name(), "javac");
        }

        #[test]
        fn test_matches_without_extension() {
            let cmd = Command::new("javac foo", "error: Class names, 'foo', are only accepted");
            assert!(Javac.is_match(&cmd));
        }

        #[test]
        fn test_no_match_with_extension() {
            let cmd = Command::new("javac foo.java", "");
            assert!(!Javac.is_match(&cmd));
        }

        #[test]
        fn test_no_match_other_tool() {
            let cmd = Command::new("java foo", "");
            assert!(!Javac.is_match(&cmd));
        }

        #[test]
        fn test_get_new_command() {
            let cmd = Command::new("javac foo", "error: Class names, 'foo', are only accepted");
            let fixes = Javac.get_new_command(&cmd);
            assert_eq!(fixes, vec!["javac foo.java"]);
        }

        #[test]
        fn test_requires_no_output() {
            assert!(!Javac.requires_output());
        }
    }

    // ------------------------------------------------------------------------
    // Maven Rules Tests
    // ------------------------------------------------------------------------

    mod mvn_no_command_tests {
        use super::*;

        #[test]
        fn test_name() {
            assert_eq!(MvnNoCommand.name(), "mvn_no_command");
        }

        #[test]
        fn test_matches_no_goals() {
            let cmd = Command::new(
                "mvn",
                "[ERROR] No goals have been specified for this build.",
            );
            assert!(MvnNoCommand.is_match(&cmd));
        }

        #[test]
        fn test_no_match_with_goals() {
            let cmd = Command::new("mvn clean install", "BUILD SUCCESS");
            assert!(!MvnNoCommand.is_match(&cmd));
        }

        #[test]
        fn test_get_new_command() {
            let cmd = Command::new(
                "mvn",
                "[ERROR] No goals have been specified for this build.",
            );
            let fixes = MvnNoCommand.get_new_command(&cmd);
            assert_eq!(fixes.len(), 2);
            assert!(fixes.contains(&"mvn clean package".to_string()));
            assert!(fixes.contains(&"mvn clean install".to_string()));
        }
    }

    mod mvn_unknown_lifecycle_phase_tests {
        use super::*;

        #[test]
        fn test_name() {
            assert_eq!(
                MvnUnknownLifecyclePhase.name(),
                "mvn_unknown_lifecycle_phase"
            );
        }

        #[test]
        fn test_matches_unknown_phase() {
            let cmd = Command::new(
                "mvn comiple",
                r#"[ERROR] Unknown lifecycle phase "comiple". You must specify a valid lifecycle phase or a goal in the format...
Available lifecycle phases are: validate, initialize, compile -> [Help 1]"#,
            );
            assert!(MvnUnknownLifecyclePhase.is_match(&cmd));
        }

        #[test]
        fn test_no_match_valid_phase() {
            let cmd = Command::new("mvn compile", "BUILD SUCCESS");
            assert!(!MvnUnknownLifecyclePhase.is_match(&cmd));
        }

        #[test]
        fn test_get_new_command() {
            let cmd = Command::new(
                "mvn comiple",
                r#"[ERROR] Unknown lifecycle phase "comiple". You must specify a valid lifecycle phase or a goal...
Available lifecycle phases are: validate, initialize, compile -> [Help 1]"#,
            );
            let fixes = MvnUnknownLifecyclePhase.get_new_command(&cmd);
            assert!(!fixes.is_empty());
            assert!(fixes.iter().any(|f| f.contains("compile")));
        }
    }

    // ------------------------------------------------------------------------
    // Composer Tests
    // ------------------------------------------------------------------------

    mod composer_tests {
        use super::*;

        #[test]
        fn test_name() {
            assert_eq!(ComposerNotCommand.name(), "composer_not_command");
        }

        #[test]
        fn test_matches_did_you_mean() {
            let cmd = Command::new(
                "composer instal",
                r#"Command "instal" is not defined.
Did you mean this?
    install"#,
            );
            assert!(ComposerNotCommand.is_match(&cmd));
        }

        #[test]
        fn test_matches_install_to_require() {
            let cmd = Command::new("composer install package", "Did you mean composer require?");
            assert!(ComposerNotCommand.is_match(&cmd));
        }

        #[test]
        fn test_no_match_valid_command() {
            let cmd = Command::new("composer install", "Installing dependencies");
            assert!(!ComposerNotCommand.is_match(&cmd));
        }

        #[test]
        fn test_get_new_command_did_you_mean() {
            let cmd = Command::new(
                "composer instal",
                r#"Command "instal" is not defined.
Did you mean this?
    install"#,
            );
            let fixes = ComposerNotCommand.get_new_command(&cmd);
            assert_eq!(fixes, vec!["composer install"]);
        }
    }

    // ------------------------------------------------------------------------
    // C++ Tests
    // ------------------------------------------------------------------------

    mod cpp11_tests {
        use super::*;

        #[test]
        fn test_name() {
            assert_eq!(Cpp11.name(), "cpp11");
        }

        #[test]
        fn test_matches_cpp11_required() {
            let cmd = Command::new(
                "g++ main.cpp",
                "This file requires compiler and library support for the ISO C++ 2011 standard",
            );
            assert!(Cpp11.is_match(&cmd));
        }

        #[test]
        fn test_matches_cpp11_extensions() {
            let cmd = Command::new(
                "clang++ main.cpp",
                "warning: range-based for loop is a C++11 extension [-Wc++11-extensions]",
            );
            assert!(Cpp11.is_match(&cmd));
        }

        #[test]
        fn test_no_match_valid_compilation() {
            let cmd = Command::new("g++ main.cpp", "");
            assert!(!Cpp11.is_match(&cmd));
        }

        #[test]
        fn test_get_new_command() {
            let cmd = Command::new(
                "g++ main.cpp",
                "This file requires compiler and library support for the ISO C++ 2011 standard",
            );
            let fixes = Cpp11.get_new_command(&cmd);
            assert_eq!(fixes, vec!["g++ main.cpp -std=c++11"]);
        }
    }

    // ------------------------------------------------------------------------
    // Fabric Tests
    // ------------------------------------------------------------------------

    mod fab_tests {
        use super::*;

        #[test]
        fn test_name() {
            assert_eq!(FabCommandNotFound.name(), "fab_command_not_found");
        }

        #[test]
        fn test_matches_command_not_found() {
            let cmd = Command::new(
                "fab deply",
                r#"Warning: Command(s) not found:
    deply
Available commands:
    deploy"#,
            );
            assert!(FabCommandNotFound.is_match(&cmd));
        }

        #[test]
        fn test_no_match_valid_command() {
            let cmd = Command::new("fab deploy", "Running deploy...");
            assert!(!FabCommandNotFound.is_match(&cmd));
        }

        #[test]
        fn test_get_new_command() {
            let cmd = Command::new(
                "fab deply",
                r#"Warning: Command(s) not found:
    deply
Available commands:
    deploy"#,
            );
            let fixes = FabCommandNotFound.get_new_command(&cmd);
            assert_eq!(fixes, vec!["fab deploy"]);
        }
    }

    // ------------------------------------------------------------------------
    // Grunt Tests
    // ------------------------------------------------------------------------

    mod grunt_tests {
        use super::*;

        #[test]
        fn test_name() {
            assert_eq!(GruntTaskNotFound.name(), "grunt_task_not_found");
        }

        #[test]
        fn test_matches_task_not_found() {
            let cmd = Command::new("grunt biuld", r#"Warning: Task "biuld" not found."#);
            assert!(GruntTaskNotFound.is_match(&cmd));
        }

        #[test]
        fn test_no_match_valid_task() {
            let cmd = Command::new("grunt build", "Running build task...");
            assert!(!GruntTaskNotFound.is_match(&cmd));
        }

        #[test]
        fn test_get_new_command() {
            let cmd = Command::new("grunt biuld", r#"Warning: Task "biuld" not found."#);
            let fixes = GruntTaskNotFound.get_new_command(&cmd);
            assert_eq!(fixes, vec!["grunt build"]);
        }
    }

    // ------------------------------------------------------------------------
    // Gulp Tests
    // ------------------------------------------------------------------------

    mod gulp_tests {
        use super::*;

        #[test]
        fn test_name() {
            assert_eq!(GulpNotTask.name(), "gulp_not_task");
        }

        #[test]
        fn test_matches_task_not_found() {
            let cmd = Command::new("gulp biuld", "Task 'biuld' is not in your gulpfile");
            assert!(GulpNotTask.is_match(&cmd));
        }

        #[test]
        fn test_no_match_valid_task() {
            let cmd = Command::new("gulp build", "Starting 'build'...");
            assert!(!GulpNotTask.is_match(&cmd));
        }

        #[test]
        fn test_get_new_command() {
            let cmd = Command::new("gulp biuld", "Task 'biuld' is not in your gulpfile");
            let fixes = GulpNotTask.get_new_command(&cmd);
            assert_eq!(fixes, vec!["gulp build"]);
        }
    }

    // ------------------------------------------------------------------------
    // Leiningen Tests
    // ------------------------------------------------------------------------

    mod lein_tests {
        use super::*;

        #[test]
        fn test_name() {
            assert_eq!(LeinNotTask.name(), "lein_not_task");
        }

        #[test]
        fn test_matches_task_not_found() {
            let cmd = Command::new(
                "lein repls",
                r#"'repls' is not a task. See 'lein help'.
Did you mean this?
         repl"#,
            );
            assert!(LeinNotTask.is_match(&cmd));
        }

        #[test]
        fn test_no_match_valid_task() {
            let cmd = Command::new("lein repl", "nREPL server started");
            assert!(!LeinNotTask.is_match(&cmd));
        }

        #[test]
        fn test_no_match_without_suggestion() {
            let cmd = Command::new("lein xyz", "'xyz' is not a task. See 'lein help'.");
            assert!(!LeinNotTask.is_match(&cmd));
        }

        #[test]
        fn test_get_new_command() {
            let cmd = Command::new(
                "lein repls",
                r#"'repls' is not a task. See 'lein help'.
Did you mean this?
         repl"#,
            );
            let fixes = LeinNotTask.get_new_command(&cmd);
            assert_eq!(fixes, vec!["lein repl"]);
        }
    }

    // ------------------------------------------------------------------------
    // Terraform Tests
    // ------------------------------------------------------------------------

    mod terraform_init_tests {
        use super::*;

        #[test]
        fn test_name() {
            assert_eq!(TerraformInit.name(), "terraform_init");
        }

        #[test]
        fn test_matches_module_not_installed() {
            let cmd = Command::new("terraform plan", "This module is not yet installed");
            assert!(TerraformInit.is_match(&cmd));
        }

        #[test]
        fn test_matches_initialization_required() {
            let cmd = Command::new("terraform apply", "Error: Initialization required");
            assert!(TerraformInit.is_match(&cmd));
        }

        #[test]
        fn test_no_match_initialized() {
            let cmd = Command::new("terraform plan", "Plan: 5 to add, 0 to change");
            assert!(!TerraformInit.is_match(&cmd));
        }

        #[test]
        fn test_get_new_command() {
            let cmd = Command::new("terraform plan", "This module is not yet installed");
            let fixes = TerraformInit.get_new_command(&cmd);
            assert_eq!(fixes, vec!["terraform init && terraform plan"]);
        }
    }

    mod terraform_no_command_tests {
        use super::*;

        #[test]
        fn test_name() {
            assert_eq!(TerraformNoCommand.name(), "terraform_no_command");
        }

        #[test]
        fn test_matches_unknown_command() {
            let cmd = Command::new(
                "terraform plna",
                r#"Terraform has no command named "plna".
Did you mean "plan"?"#,
            );
            assert!(TerraformNoCommand.is_match(&cmd));
        }

        #[test]
        fn test_no_match_valid_command() {
            let cmd = Command::new("terraform plan", "Plan: 5 to add");
            assert!(!TerraformNoCommand.is_match(&cmd));
        }

        #[test]
        fn test_no_match_without_suggestion() {
            let cmd = Command::new("terraform xyz", r#"Terraform has no command named "xyz"."#);
            assert!(!TerraformNoCommand.is_match(&cmd));
        }

        #[test]
        fn test_get_new_command() {
            let cmd = Command::new(
                "terraform plna",
                r#"Terraform has no command named "plna".
Did you mean "plan"?"#,
            );
            let fixes = TerraformNoCommand.get_new_command(&cmd);
            assert_eq!(fixes, vec!["terraform plan"]);
        }
    }

    // ------------------------------------------------------------------------
    // All Rules Tests
    // ------------------------------------------------------------------------

    #[test]
    fn test_all_rules_not_empty() {
        let rules = all_rules();
        assert!(!rules.is_empty());
    }

    #[test]
    fn test_all_rules_have_names() {
        let rules = all_rules();
        for rule in rules {
            assert!(!rule.name().is_empty());
        }
    }

    #[test]
    fn test_all_rules_count() {
        let rules = all_rules();
        assert_eq!(rules.len(), 16, "Expected 16 devtools rules");
    }

    #[test]
    fn test_no_duplicate_rule_names() {
        let rules = all_rules();
        let mut names: Vec<&str> = rules.iter().map(|r| r.name()).collect();
        let original_len = names.len();
        names.sort();
        names.dedup();
        assert_eq!(names.len(), original_len, "Rule names should be unique");
    }
}
