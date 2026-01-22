//! Cloud and network rules (AWS, Azure, Heroku, SSH, Tsuru, etc.)
//!
//! This module contains rules for cloud services and network-related commands:
//!
//! - [`AwsCli`] - AWS CLI command fixes
//! - [`AzCli`] - Azure CLI command fixes
//! - [`HerokuMultipleApps`] - Fix heroku multiple apps error
//! - [`HerokuNotCommand`] - Fix unknown heroku commands
//! - [`SshKnownHosts`] - Handle SSH known_hosts issues
//! - [`Whois`] - Fix whois command errors
//! - [`PortAlreadyInUse`] - Suggest killing process on port
//! - [`TsuruLogin`] - Tsuru login suggestions
//! - [`TsuruNotCommand`] - Tsuru command fixes
//! - [`HostsCli`] - Hosts CLI fixes

use crate::core::{is_app, Command, Rule};
use crate::shells::detect_shell;
use crate::utils::{get_close_matches, replace_argument};
use regex::Regex;

// =============================================================================
// AWS CLI Rule
// =============================================================================

/// Rule that fixes AWS CLI command errors.
///
/// AWS CLI provides helpful suggestions when a command is invalid.
/// This rule extracts those suggestions and offers them as corrections.
///
/// # Example
///
/// ```
/// use oops::rules::cloud::AwsCli;
/// use oops::core::{Command, Rule};
///
/// let rule = AwsCli;
/// let cmd = Command::new(
///     "aws dynamdb describe-table",
///     "usage: aws [options] <command> <subcommand>\nInvalid choice: 'dynamdb', maybe you meant:\n\n\t* dynamodb"
/// );
/// assert!(rule.is_match(&cmd));
/// ```
#[derive(Debug, Clone, Copy, Default)]
pub struct AwsCli;

impl AwsCli {
    pub fn new() -> Self {
        Self
    }
}

impl Rule for AwsCli {
    fn name(&self) -> &str {
        "aws_cli"
    }

    fn is_match(&self, cmd: &Command) -> bool {
        if !is_app(cmd, &["aws"]) {
            return false;
        }
        cmd.output.contains("usage:") && cmd.output.contains("maybe you meant:")
    }

    fn get_new_command(&self, cmd: &Command) -> Vec<String> {
        // Pattern to extract the invalid choice: (?<=Invalid choice: ')(.*)(?=', maybe you meant:)
        let invalid_choice_re = Regex::new(r"Invalid choice: '([^']*)', maybe you meant:").unwrap();
        // Pattern to extract options: ^\s*\*\s(.*)
        let options_re = Regex::new(r"(?m)^\s*\*\s+(.+)$").unwrap();

        let mistake = match invalid_choice_re.captures(&cmd.output) {
            Some(caps) => caps.get(1).map(|m| m.as_str().to_string()),
            None => None,
        };

        let mistake = match mistake {
            Some(m) => m,
            None => return vec![],
        };

        let options: Vec<String> = options_re
            .captures_iter(&cmd.output)
            .filter_map(|caps| caps.get(1).map(|m| m.as_str().trim().to_string()))
            .collect();

        options
            .into_iter()
            .map(|opt| replace_argument(&cmd.script, &mistake, &opt))
            .collect()
    }

    fn priority(&self) -> i32 {
        1000
    }

    fn requires_output(&self) -> bool {
        true
    }
}

// =============================================================================
// Azure CLI Rule
// =============================================================================

/// Rule that fixes Azure CLI command errors.
///
/// Azure CLI provides suggestions when a command is not found in a command group.
/// This rule extracts those suggestions and offers them as corrections.
///
/// # Example
///
/// ```
/// use oops::rules::cloud::AzCli;
/// use oops::core::{Command, Rule};
///
/// let rule = AzCli;
/// let cmd = Command::new(
///     "az resoure list",
///     "az: 'resoure' is not in the 'az' command group.\nThe most similar choice to 'resoure' is:\n    resource"
/// );
/// assert!(rule.is_match(&cmd));
/// ```
#[derive(Debug, Clone, Copy, Default)]
pub struct AzCli;

impl AzCli {
    pub fn new() -> Self {
        Self
    }
}

impl Rule for AzCli {
    fn name(&self) -> &str {
        "az_cli"
    }

    fn is_match(&self, cmd: &Command) -> bool {
        if !is_app(cmd, &["az"]) {
            return false;
        }
        cmd.output.contains("is not in the") && cmd.output.contains("command group")
    }

    fn get_new_command(&self, cmd: &Command) -> Vec<String> {
        // Pattern to extract the invalid command
        let invalid_choice_re = Regex::new(r"'([^']*)' is not in the '.*' command group").unwrap();
        // Pattern to extract the suggestion
        let options_re =
            Regex::new(r"(?m)^The most similar choice to '[^']*' is:\n\s*(.+)$").unwrap();

        let mistake = match invalid_choice_re.captures(&cmd.output) {
            Some(caps) => caps.get(1).map(|m| m.as_str().to_string()),
            None => None,
        };

        let mistake = match mistake {
            Some(m) => m,
            None => return vec![],
        };

        let options: Vec<String> = options_re
            .captures_iter(&cmd.output)
            .filter_map(|caps| caps.get(1).map(|m| m.as_str().trim().to_string()))
            .collect();

        options
            .into_iter()
            .map(|opt| replace_argument(&cmd.script, &mistake, &opt))
            .collect()
    }

    fn priority(&self) -> i32 {
        1000
    }

    fn requires_output(&self) -> bool {
        true
    }
}

// =============================================================================
// Heroku Multiple Apps Rule
// =============================================================================

/// Rule that fixes Heroku commands when multiple apps are available.
///
/// When a Heroku command fails because multiple apps are configured,
/// this rule extracts the available apps and suggests the command with
/// the `--app` flag.
///
/// # Example
///
/// ```
/// use oops::rules::cloud::HerokuMultipleApps;
/// use oops::core::{Command, Rule};
///
/// let rule = HerokuMultipleApps;
/// let cmd = Command::new(
///     "heroku logs",
///     "Multiple apps in folder and target app is not specified.\n\nSpecify app with --app APP.\n\nAvailable apps:\nmy-app-staging (git remote: staging)\nmy-app-production (git remote: production)\n\nhttps://devcenter.heroku.com/articles/multiple-environments"
/// );
/// assert!(rule.is_match(&cmd));
/// ```
#[derive(Debug, Clone, Copy, Default)]
pub struct HerokuMultipleApps;

impl HerokuMultipleApps {
    pub fn new() -> Self {
        Self
    }
}

impl Rule for HerokuMultipleApps {
    fn name(&self) -> &str {
        "heroku_multiple_apps"
    }

    fn is_match(&self, cmd: &Command) -> bool {
        if !is_app(cmd, &["heroku"]) {
            return false;
        }
        cmd.output
            .contains("https://devcenter.heroku.com/articles/multiple-environments")
    }

    fn get_new_command(&self, cmd: &Command) -> Vec<String> {
        // Pattern to extract app names: (\S+) \([^)]*\)
        // Using \S+ to match non-whitespace characters (including newlines won't be matched)
        let apps_re = Regex::new(r"(\S+) \([^)]*\)").unwrap();

        let apps: Vec<String> = apps_re
            .captures_iter(&cmd.output)
            .filter_map(|caps| caps.get(1).map(|m| m.as_str().to_string()))
            .collect();

        apps.into_iter()
            .map(|app| format!("{} --app {}", cmd.script, app))
            .collect()
    }

    fn priority(&self) -> i32 {
        1000
    }

    fn requires_output(&self) -> bool {
        true
    }
}

// =============================================================================
// Heroku Not Command Rule
// =============================================================================

/// Rule that fixes unknown Heroku commands.
///
/// When a Heroku command is not recognized, Heroku suggests the correct command.
/// This rule extracts that suggestion.
///
/// # Example
///
/// ```
/// use oops::rules::cloud::HerokuNotCommand;
/// use oops::core::{Command, Rule};
///
/// let rule = HerokuNotCommand;
/// let cmd = Command::new(
///     "heroku lgs",
///     "lgs is not a heroku command.\nRun heroku _ to run heroku logs."
/// );
/// assert!(rule.is_match(&cmd));
/// ```
#[derive(Debug, Clone, Copy, Default)]
pub struct HerokuNotCommand;

impl HerokuNotCommand {
    pub fn new() -> Self {
        Self
    }
}

impl Rule for HerokuNotCommand {
    fn name(&self) -> &str {
        "heroku_not_command"
    }

    fn is_match(&self, cmd: &Command) -> bool {
        if !is_app(cmd, &["heroku"]) {
            return false;
        }
        cmd.output.contains("Run heroku _ to run")
    }

    fn get_new_command(&self, cmd: &Command) -> Vec<String> {
        // Pattern to extract the suggested command: Run heroku _ to run ([^.]+)
        let suggestion_re = Regex::new(r"Run heroku _ to run ([^.]+)").unwrap();

        if let Some(caps) = suggestion_re.captures(&cmd.output) {
            if let Some(suggestion) = caps.get(1) {
                return vec![format!("heroku {}", suggestion.as_str().trim())];
            }
        }
        vec![]
    }

    fn priority(&self) -> i32 {
        1000
    }

    fn requires_output(&self) -> bool {
        true
    }
}

// =============================================================================
// SSH Known Hosts Rule
// =============================================================================

/// Rule that handles SSH known_hosts verification failures.
///
/// When SSH detects that a remote host's identification has changed,
/// this rule suggests re-running the command after the offending key
/// has been handled.
///
/// Note: In the Python version, this rule has a side_effect that removes
/// the offending line from known_hosts. In this Rust version, we maintain
/// the same behavior.
///
/// # Example
///
/// ```
/// use oops::rules::cloud::SshKnownHosts;
/// use oops::core::{Command, Rule};
///
/// let rule = SshKnownHosts;
/// let cmd = Command::new(
///     "ssh user@host",
///     "WARNING: REMOTE HOST IDENTIFICATION HAS CHANGED!\nOffending key in /home/user/.ssh/known_hosts:5"
/// );
/// assert!(rule.is_match(&cmd));
/// ```
#[derive(Debug, Clone, Copy, Default)]
pub struct SshKnownHosts;

impl SshKnownHosts {
    pub fn new() -> Self {
        Self
    }
}

impl Rule for SshKnownHosts {
    fn name(&self) -> &str {
        "ssh_known_hosts"
    }

    fn is_match(&self, cmd: &Command) -> bool {
        if !is_app(cmd, &["ssh", "scp"]) {
            return false;
        }

        let patterns = [
            r"WARNING: REMOTE HOST IDENTIFICATION HAS CHANGED!",
            r"WARNING: POSSIBLE DNS SPOOFING DETECTED!",
            r"Warning: the \S+ host key for '[^']+' differs from the key for the IP address '[^']+'",
        ];

        patterns.iter().any(|pattern| {
            if let Ok(re) = Regex::new(pattern) {
                re.is_match(&cmd.output)
            } else {
                false
            }
        })
    }

    fn get_new_command(&self, cmd: &Command) -> Vec<String> {
        // The Python version returns the same command, and relies on side_effect
        // to remove the offending key. We do the same here.
        vec![cmd.script.clone()]
    }

    fn side_effect(&self, old_cmd: &Command, _new_script: &str) -> anyhow::Result<()> {
        // Pattern to find offending key entries: Offending key in ([^:]+):(\d+)
        let offending_re =
            Regex::new(r"(?:Offending (?:key for IP|\S+ key)|Matching host key) in ([^:]+):(\d+)")
                .unwrap();

        for caps in offending_re.captures_iter(&old_cmd.output) {
            let filepath = caps.get(1).map(|m| m.as_str()).unwrap_or("");
            let lineno_str = caps.get(2).map(|m| m.as_str()).unwrap_or("0");
            let lineno: usize = lineno_str.parse().unwrap_or(0);

            if lineno == 0 || filepath.is_empty() {
                continue;
            }

            // Read the file, remove the offending line, write it back
            if let Ok(content) = std::fs::read_to_string(filepath) {
                let lines: Vec<&str> = content.lines().collect();
                if lineno <= lines.len() {
                    let new_lines: Vec<&str> = lines
                        .iter()
                        .enumerate()
                        .filter(|(i, _)| *i != lineno - 1) // lineno is 1-based
                        .map(|(_, line)| *line)
                        .collect();
                    let new_content = new_lines.join("\n");
                    // Add trailing newline if original had one
                    let new_content = if content.ends_with('\n') {
                        format!("{}\n", new_content)
                    } else {
                        new_content
                    };
                    std::fs::write(filepath, new_content)?;
                }
            }
        }

        Ok(())
    }

    fn priority(&self) -> i32 {
        1000
    }

    fn requires_output(&self) -> bool {
        true
    }
}

// =============================================================================
// Whois Rule
// =============================================================================

/// Rule that fixes whois command errors.
///
/// Common whois mistakes:
/// - `whois https://example.com/` -> `whois example.com` (remove protocol/path)
/// - `whois sub.example.com` -> `whois example.com` (remove subdomain)
///
/// # Example
///
/// ```
/// use oops::rules::cloud::Whois;
/// use oops::core::{Command, Rule};
///
/// let rule = Whois;
/// let cmd = Command::new("whois https://example.com/page", "");
/// assert!(rule.is_match(&cmd));
/// ```
#[derive(Debug, Clone, Copy, Default)]
pub struct Whois;

impl Whois {
    pub fn new() -> Self {
        Self
    }
}

impl Rule for Whois {
    fn name(&self) -> &str {
        "whois"
    }

    fn is_match(&self, cmd: &Command) -> bool {
        // Match any whois command with at least one argument
        if !is_app(cmd, &["whois"]) {
            return false;
        }
        cmd.script_parts().len() >= 2
    }

    fn get_new_command(&self, cmd: &Command) -> Vec<String> {
        let parts = cmd.script_parts();
        if parts.len() < 2 {
            return vec![];
        }

        let url = &parts[1];

        // If there's a slash, extract the hostname from URL
        if cmd.script.contains('/') {
            // Parse as URL to extract netloc
            if let Ok(parsed) = url::Url::parse(url) {
                if let Some(host) = parsed.host_str() {
                    return vec![format!("whois {}", host)];
                }
            }
            // Fallback: try to extract hostname manually
            let without_protocol = url
                .trim_start_matches("http://")
                .trim_start_matches("https://")
                .trim_start_matches("ftp://");
            let hostname = without_protocol
                .split('/')
                .next()
                .unwrap_or(without_protocol);
            if !hostname.is_empty() {
                return vec![format!("whois {}", hostname)];
            }
        } else if cmd.script.contains('.') {
            // No slash but has dot - try removing subdomains
            let domain_parts: Vec<&str> = url.split('.').collect();
            if domain_parts.len() > 1 {
                // Generate suggestions removing left-most subdomain(s)
                return (1..domain_parts.len())
                    .map(|n| format!("whois {}", domain_parts[n..].join(".")))
                    .collect();
            }
        }

        vec![]
    }

    fn priority(&self) -> i32 {
        1000
    }

    fn requires_output(&self) -> bool {
        // This rule doesn't need output - it matches based on command structure
        false
    }
}

// =============================================================================
// Port Already In Use Rule
// =============================================================================

/// Rule that suggests killing a process when a port is already in use.
///
/// When a server fails to start because a port is occupied, this rule
/// suggests killing the occupying process and retrying the command.
///
/// Note: This rule requires `lsof` to be available on the system.
///
/// # Example
///
/// ```
/// use oops::rules::cloud::PortAlreadyInUse;
/// use oops::core::{Command, Rule};
///
/// let rule = PortAlreadyInUse;
/// let cmd = Command::new(
///     "python -m http.server 8000",
///     "OSError: [Errno 98] Address already in use\nbind on address ('', 8000)"
/// );
/// // Note: is_match will return false if lsof isn't available or no process is found
/// ```
#[derive(Debug, Clone, Copy, Default)]
pub struct PortAlreadyInUse;

impl PortAlreadyInUse {
    pub fn new() -> Self {
        Self
    }

    /// Extract the port number from the command output.
    fn get_used_port(output: &str) -> Option<u16> {
        let patterns = [
            r"bind on address \('.*', (?P<port>\d+)\)",
            r"Unable to bind [^ ]*:(?P<port>\d+)",
            r"can't listen on port (?P<port>\d+)",
            r"listen EADDRINUSE [^ ]*:(?P<port>\d+)",
            r"Address already in use.*:(?P<port>\d+)",
            r"port (?P<port>\d+) already in use",
        ];

        for pattern in &patterns {
            if let Ok(re) = Regex::new(pattern) {
                if let Some(caps) = re.captures(output) {
                    if let Some(port_match) = caps.name("port") {
                        if let Ok(port) = port_match.as_str().parse() {
                            return Some(port);
                        }
                    }
                }
            }
        }
        None
    }

    /// Get the PID of the process using the given port using lsof.
    fn get_pid_by_port(port: u16) -> Option<u32> {
        use std::process::Command as ProcessCommand;

        // Check if lsof is available
        crate::utils::which("lsof".to_string())?;

        let output = ProcessCommand::new("lsof")
            .args(["-i", &format!(":{}", port)])
            .output()
            .ok()?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let lines: Vec<&str> = stdout.lines().collect();

        // Skip header line
        if lines.len() > 1 {
            let fields: Vec<&str> = lines[1].split_whitespace().collect();
            if fields.len() > 1 {
                return fields[1].parse().ok();
            }
        }
        None
    }
}

impl Rule for PortAlreadyInUse {
    fn name(&self) -> &str {
        "port_already_in_use"
    }

    fn enabled_by_default(&self) -> bool {
        // Only enable if lsof is available
        crate::utils::which("lsof".to_string()).is_some()
    }

    fn is_match(&self, cmd: &Command) -> bool {
        if let Some(port) = Self::get_used_port(&cmd.output) {
            Self::get_pid_by_port(port).is_some()
        } else {
            false
        }
    }

    fn get_new_command(&self, cmd: &Command) -> Vec<String> {
        let shell = detect_shell();

        if let Some(port) = Self::get_used_port(&cmd.output) {
            if let Some(pid) = Self::get_pid_by_port(port) {
                let kill_cmd = format!("kill {}", pid);
                return vec![shell.and_(&[&kill_cmd, &cmd.script])];
            }
        }
        vec![]
    }

    fn priority(&self) -> i32 {
        1000
    }

    fn requires_output(&self) -> bool {
        true
    }
}

// =============================================================================
// Tsuru Login Rule
// =============================================================================

/// Rule that suggests logging in to Tsuru when authentication fails.
///
/// # Example
///
/// ```
/// use oops::rules::cloud::TsuruLogin;
/// use oops::core::{Command, Rule};
///
/// let rule = TsuruLogin;
/// let cmd = Command::new(
///     "tsuru app-list",
///     "Error: you're not authenticated or session has expired."
/// );
/// // Note: is_match requires both error messages
/// ```
#[derive(Debug, Clone, Copy, Default)]
pub struct TsuruLogin;

impl TsuruLogin {
    pub fn new() -> Self {
        Self
    }
}

impl Rule for TsuruLogin {
    fn name(&self) -> &str {
        "tsuru_login"
    }

    fn is_match(&self, cmd: &Command) -> bool {
        if !is_app(cmd, &["tsuru"]) {
            return false;
        }
        cmd.output.contains("not authenticated") && cmd.output.contains("session has expired")
    }

    fn get_new_command(&self, cmd: &Command) -> Vec<String> {
        let shell = detect_shell();
        vec![shell.and_(&["tsuru login", &cmd.script])]
    }

    fn priority(&self) -> i32 {
        1000
    }

    fn requires_output(&self) -> bool {
        true
    }
}

// =============================================================================
// Tsuru Not Command Rule
// =============================================================================

/// Rule that fixes unknown Tsuru commands.
///
/// When a Tsuru command is not recognized, Tsuru suggests similar commands.
/// This rule extracts those suggestions.
///
/// # Example
///
/// ```
/// use oops::rules::cloud::TsuruNotCommand;
/// use oops::core::{Command, Rule};
///
/// let rule = TsuruNotCommand;
/// let cmd = Command::new(
///     "tsuru ap-list",
///     "tsuru: \"ap-list\" is not a tsuru command. See \"tsuru help\".\nDid you mean?\n\tapp-list"
/// );
/// assert!(rule.is_match(&cmd));
/// ```
#[derive(Debug, Clone, Copy, Default)]
pub struct TsuruNotCommand;

impl TsuruNotCommand {
    pub fn new() -> Self {
        Self
    }
}

impl Rule for TsuruNotCommand {
    fn name(&self) -> &str {
        "tsuru_not_command"
    }

    fn is_match(&self, cmd: &Command) -> bool {
        if !is_app(cmd, &["tsuru"]) {
            return false;
        }
        cmd.output.contains("is not a tsuru command")
            && cmd.output.contains("Did you mean?")
            && cmd.output.contains("\n\t")
    }

    fn get_new_command(&self, cmd: &Command) -> Vec<String> {
        // Extract the broken command: tsuru: "([^"]*)" is not a tsuru command
        let broken_cmd_re = Regex::new(r#"tsuru: "([^"]*)" is not a tsuru command"#).unwrap();

        let broken_cmd = match broken_cmd_re.captures(&cmd.output) {
            Some(caps) => caps.get(1).map(|m| m.as_str().to_string()),
            None => None,
        };

        let broken_cmd = match broken_cmd {
            Some(c) => c,
            None => return vec![],
        };

        // Extract suggestions from "Did you mean?\n\t<suggestion>"
        // Pattern: lines starting with \t after "Did you mean?"
        let suggestions: Vec<String> = if let Some(pos) = cmd.output.find("Did you mean?") {
            cmd.output[pos..]
                .lines()
                .skip(1) // Skip "Did you mean?" line
                .filter(|line| line.starts_with('\t'))
                .map(|line| line.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect()
        } else {
            vec![]
        };

        suggestions
            .into_iter()
            .map(|suggestion| replace_argument(&cmd.script, &broken_cmd, &suggestion))
            .collect()
    }

    fn priority(&self) -> i32 {
        1000
    }

    fn requires_output(&self) -> bool {
        true
    }
}

// =============================================================================
// Hosts CLI Rule
// =============================================================================

/// Rule that fixes hostscli command errors.
///
/// Common errors:
/// - Unknown command - suggests similar commands
/// - Website import error - suggests `hostscli websites`
///
/// # Example
///
/// ```
/// use oops::rules::cloud::HostsCli;
/// use oops::core::{Command, Rule};
///
/// let rule = HostsCli;
/// let cmd = Command::new(
///     "hostscli blok facebook.com",
///     "Error: No such command \"blok\""
/// );
/// assert!(rule.is_match(&cmd));
/// ```
#[derive(Debug, Clone, Copy, Default)]
pub struct HostsCli;

impl HostsCli {
    pub fn new() -> Self {
        Self
    }

    /// Available hostscli commands.
    const COMMANDS: &'static [&'static str] =
        &["block", "unblock", "websites", "block_all", "unblock_all"];
}

impl Rule for HostsCli {
    fn name(&self) -> &str {
        "hostscli"
    }

    fn is_match(&self, cmd: &Command) -> bool {
        if !is_app(cmd, &["hostscli"]) {
            return false;
        }
        cmd.output.contains("Error: No such command")
            || cmd.output.contains("hostscli.errors.WebsiteImportError")
    }

    fn get_new_command(&self, cmd: &Command) -> Vec<String> {
        // Website import error - suggest listing websites
        if cmd.output.contains("hostscli.errors.WebsiteImportError") {
            return vec!["hostscli websites".to_string()];
        }

        // Extract misspelled command from error: Error: No such command "..."
        let error_re = Regex::new(r#"Error: No such command "([^"]*)""#).unwrap();

        if let Some(caps) = error_re.captures(&cmd.output) {
            if let Some(misspelled) = caps.get(1).map(|m| m.as_str()) {
                let commands: Vec<String> = Self::COMMANDS.iter().map(|s| s.to_string()).collect();
                let matches = get_close_matches(misspelled, &commands, 3, 0.6);

                if !matches.is_empty() {
                    return matches
                        .into_iter()
                        .map(|correct| replace_argument(&cmd.script, misspelled, &correct))
                        .collect();
                }
            }
        }

        vec![]
    }

    fn priority(&self) -> i32 {
        1000
    }

    fn requires_output(&self) -> bool {
        true
    }
}

// =============================================================================
// Module Functions
// =============================================================================

/// Returns all cloud and network rules as boxed trait objects.
pub fn all_rules() -> Vec<Box<dyn Rule>> {
    vec![
        Box::new(AwsCli::new()),
        Box::new(AzCli::new()),
        Box::new(HerokuMultipleApps::new()),
        Box::new(HerokuNotCommand::new()),
        Box::new(SshKnownHosts::new()),
        Box::new(Whois::new()),
        Box::new(PortAlreadyInUse::new()),
        Box::new(TsuruLogin::new()),
        Box::new(TsuruNotCommand::new()),
        Box::new(HostsCli::new()),
    ]
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    mod aws_cli {
        use super::*;

        #[test]
        fn test_name() {
            let rule = AwsCli::new();
            assert_eq!(rule.name(), "aws_cli");
        }

        #[test]
        fn test_matches_invalid_choice() {
            let rule = AwsCli::new();
            let cmd = Command::new(
                "aws dynamdb describe-table",
                "usage: aws [options] <command>\nInvalid choice: 'dynamdb', maybe you meant:\n\n\t* dynamodb",
            );
            assert!(rule.is_match(&cmd));
        }

        #[test]
        fn test_no_match_valid_command() {
            let rule = AwsCli::new();
            let cmd = Command::new("aws dynamodb describe-table", "Table details...");
            assert!(!rule.is_match(&cmd));
        }

        #[test]
        fn test_no_match_other_command() {
            let rule = AwsCli::new();
            let cmd = Command::new("gcloud compute instances list", "usage: maybe you meant:");
            assert!(!rule.is_match(&cmd));
        }

        #[test]
        fn test_get_new_command() {
            let rule = AwsCli::new();
            let cmd = Command::new(
                "aws dynamdb describe-table",
                "usage: aws [options] <command>\nInvalid choice: 'dynamdb', maybe you meant:\n\n\t* dynamodb",
            );
            let fixes = rule.get_new_command(&cmd);
            assert!(!fixes.is_empty());
            assert!(fixes[0].contains("dynamodb"));
        }
    }

    mod az_cli {
        use super::*;

        #[test]
        fn test_name() {
            let rule = AzCli::new();
            assert_eq!(rule.name(), "az_cli");
        }

        #[test]
        fn test_matches_not_in_command_group() {
            let rule = AzCli::new();
            let cmd = Command::new(
                "az resoure list",
                "az: 'resoure' is not in the 'az' command group.\nThe most similar choice to 'resoure' is:\n    resource",
            );
            assert!(rule.is_match(&cmd));
        }

        #[test]
        fn test_no_match_valid_command() {
            let rule = AzCli::new();
            let cmd = Command::new("az resource list", "Resources listed");
            assert!(!rule.is_match(&cmd));
        }

        #[test]
        fn test_get_new_command() {
            let rule = AzCli::new();
            let cmd = Command::new(
                "az resoure list",
                "az: 'resoure' is not in the 'az' command group.\nThe most similar choice to 'resoure' is:\n    resource",
            );
            let fixes = rule.get_new_command(&cmd);
            assert!(!fixes.is_empty());
            assert!(fixes[0].contains("resource"));
        }
    }

    mod heroku_multiple_apps {
        use super::*;

        #[test]
        fn test_name() {
            let rule = HerokuMultipleApps::new();
            assert_eq!(rule.name(), "heroku_multiple_apps");
        }

        #[test]
        fn test_matches_multiple_apps() {
            let rule = HerokuMultipleApps::new();
            let cmd = Command::new(
                "heroku logs",
                "my-app-staging (staging)\nmy-app-production (production)\nhttps://devcenter.heroku.com/articles/multiple-environments",
            );
            assert!(rule.is_match(&cmd));
        }

        #[test]
        fn test_no_match_single_app() {
            let rule = HerokuMultipleApps::new();
            let cmd = Command::new("heroku logs", "Log output...");
            assert!(!rule.is_match(&cmd));
        }

        #[test]
        fn test_get_new_command() {
            let rule = HerokuMultipleApps::new();
            let cmd = Command::new(
                "heroku logs",
                "my-app-staging (staging)\nmy-app-production (production)\nhttps://devcenter.heroku.com/articles/multiple-environments",
            );
            let fixes = rule.get_new_command(&cmd);
            assert_eq!(fixes.len(), 2);
            assert!(
                fixes[0].contains("--app my-app-staging"),
                "Expected '--app my-app-staging' in fix[0], got: {:?}",
                fixes
            );
            assert!(
                fixes[1].contains("--app my-app-production"),
                "Expected '--app my-app-production' in fix[1], got: {:?}",
                fixes
            );
        }
    }

    mod heroku_not_command {
        use super::*;

        #[test]
        fn test_name() {
            let rule = HerokuNotCommand::new();
            assert_eq!(rule.name(), "heroku_not_command");
        }

        #[test]
        fn test_matches_not_command() {
            let rule = HerokuNotCommand::new();
            let cmd = Command::new("heroku lgs", "Run heroku _ to run heroku logs.");
            assert!(rule.is_match(&cmd));
        }

        #[test]
        fn test_no_match_valid_command() {
            let rule = HerokuNotCommand::new();
            let cmd = Command::new("heroku logs", "Log output...");
            assert!(!rule.is_match(&cmd));
        }

        #[test]
        fn test_get_new_command() {
            let rule = HerokuNotCommand::new();
            let cmd = Command::new("heroku lgs", "Run heroku _ to run heroku logs.");
            let fixes = rule.get_new_command(&cmd);
            assert_eq!(fixes, vec!["heroku heroku logs"]);
        }
    }

    mod ssh_known_hosts {
        use super::*;

        #[test]
        fn test_name() {
            let rule = SshKnownHosts::new();
            assert_eq!(rule.name(), "ssh_known_hosts");
        }

        #[test]
        fn test_matches_host_changed() {
            let rule = SshKnownHosts::new();
            let cmd = Command::new(
                "ssh user@host",
                "WARNING: REMOTE HOST IDENTIFICATION HAS CHANGED!",
            );
            assert!(rule.is_match(&cmd));
        }

        #[test]
        fn test_matches_dns_spoofing() {
            let rule = SshKnownHosts::new();
            let cmd = Command::new("ssh user@host", "WARNING: POSSIBLE DNS SPOOFING DETECTED!");
            assert!(rule.is_match(&cmd));
        }

        #[test]
        fn test_matches_scp() {
            let rule = SshKnownHosts::new();
            let cmd = Command::new(
                "scp file user@host:/path",
                "WARNING: REMOTE HOST IDENTIFICATION HAS CHANGED!",
            );
            assert!(rule.is_match(&cmd));
        }

        #[test]
        fn test_no_match_successful() {
            let rule = SshKnownHosts::new();
            let cmd = Command::new("ssh user@host", "Welcome to host");
            assert!(!rule.is_match(&cmd));
        }

        #[test]
        fn test_get_new_command() {
            let rule = SshKnownHosts::new();
            let cmd = Command::new(
                "ssh user@host",
                "WARNING: REMOTE HOST IDENTIFICATION HAS CHANGED!",
            );
            let fixes = rule.get_new_command(&cmd);
            assert_eq!(fixes, vec!["ssh user@host"]);
        }
    }

    mod whois {
        use super::*;

        #[test]
        fn test_name() {
            let rule = Whois::new();
            assert_eq!(rule.name(), "whois");
        }

        #[test]
        fn test_matches_url_with_slash() {
            let rule = Whois::new();
            let cmd = Command::new("whois https://example.com/page", "");
            assert!(rule.is_match(&cmd));
        }

        #[test]
        fn test_matches_domain_with_subdomain() {
            let rule = Whois::new();
            let cmd = Command::new("whois www.example.com", "");
            assert!(rule.is_match(&cmd));
        }

        #[test]
        fn test_no_match_no_args() {
            let rule = Whois::new();
            let cmd = Command::new("whois", "");
            assert!(!rule.is_match(&cmd));
        }

        #[test]
        fn test_get_new_command_url() {
            let rule = Whois::new();
            let cmd = Command::new("whois https://example.com/page", "");
            let fixes = rule.get_new_command(&cmd);
            assert!(!fixes.is_empty());
            assert_eq!(fixes[0], "whois example.com");
        }

        #[test]
        fn test_get_new_command_subdomain() {
            let rule = Whois::new();
            let cmd = Command::new("whois www.example.com", "");
            let fixes = rule.get_new_command(&cmd);
            assert!(!fixes.is_empty());
            assert!(fixes.contains(&"whois example.com".to_string()));
        }

        #[test]
        fn test_requires_output() {
            let rule = Whois::new();
            assert!(!rule.requires_output());
        }
    }

    mod port_already_in_use {
        use super::*;

        #[test]
        fn test_name() {
            let rule = PortAlreadyInUse::new();
            assert_eq!(rule.name(), "port_already_in_use");
        }

        #[test]
        fn test_get_used_port_bind() {
            let output = "bind on address ('', 8000)";
            assert_eq!(PortAlreadyInUse::get_used_port(output), Some(8000));
        }

        #[test]
        fn test_get_used_port_unable_to_bind() {
            let output = "Unable to bind 0.0.0.0:3000";
            assert_eq!(PortAlreadyInUse::get_used_port(output), Some(3000));
        }

        #[test]
        fn test_get_used_port_eaddrinuse() {
            let output = "listen EADDRINUSE 0.0.0.0:8080";
            assert_eq!(PortAlreadyInUse::get_used_port(output), Some(8080));
        }

        #[test]
        fn test_get_used_port_none() {
            let output = "Some other error";
            assert_eq!(PortAlreadyInUse::get_used_port(output), None);
        }
    }

    mod tsuru_login {
        use super::*;

        #[test]
        fn test_name() {
            let rule = TsuruLogin::new();
            assert_eq!(rule.name(), "tsuru_login");
        }

        #[test]
        fn test_matches_not_authenticated() {
            let rule = TsuruLogin::new();
            let cmd = Command::new(
                "tsuru app-list",
                "Error: you're not authenticated or session has expired.",
            );
            assert!(rule.is_match(&cmd));
        }

        #[test]
        fn test_no_match_authenticated() {
            let rule = TsuruLogin::new();
            let cmd = Command::new("tsuru app-list", "Apps listed...");
            assert!(!rule.is_match(&cmd));
        }

        #[test]
        fn test_get_new_command() {
            let rule = TsuruLogin::new();
            let cmd = Command::new(
                "tsuru app-list",
                "Error: you're not authenticated or session has expired.",
            );
            let fixes = rule.get_new_command(&cmd);
            assert!(!fixes.is_empty());
            assert!(fixes[0].contains("tsuru login"));
            assert!(fixes[0].contains("tsuru app-list"));
        }
    }

    mod tsuru_not_command {
        use super::*;

        #[test]
        fn test_name() {
            let rule = TsuruNotCommand::new();
            assert_eq!(rule.name(), "tsuru_not_command");
        }

        #[test]
        fn test_matches_not_command() {
            let rule = TsuruNotCommand::new();
            let cmd = Command::new(
                "tsuru ap-list",
                "tsuru: \"ap-list\" is not a tsuru command. See \"tsuru help\".\nDid you mean?\n\tapp-list",
            );
            assert!(rule.is_match(&cmd));
        }

        #[test]
        fn test_no_match_valid_command() {
            let rule = TsuruNotCommand::new();
            let cmd = Command::new("tsuru app-list", "Apps...");
            assert!(!rule.is_match(&cmd));
        }

        #[test]
        fn test_get_new_command() {
            let rule = TsuruNotCommand::new();
            let cmd = Command::new(
                "tsuru ap-list",
                "tsuru: \"ap-list\" is not a tsuru command. See \"tsuru help\".\nDid you mean?\n\tapp-list",
            );
            let fixes = rule.get_new_command(&cmd);
            assert!(!fixes.is_empty());
            assert!(fixes[0].contains("app-list"));
        }
    }

    mod hostscli {
        use super::*;

        #[test]
        fn test_name() {
            let rule = HostsCli::new();
            assert_eq!(rule.name(), "hostscli");
        }

        #[test]
        fn test_matches_no_such_command() {
            let rule = HostsCli::new();
            let cmd = Command::new(
                "hostscli blok facebook.com",
                "Error: No such command \"blok\"",
            );
            assert!(rule.is_match(&cmd));
        }

        #[test]
        fn test_matches_website_import_error() {
            let rule = HostsCli::new();
            let cmd = Command::new(
                "hostscli block_all somefile",
                "hostscli.errors.WebsiteImportError: ...",
            );
            assert!(rule.is_match(&cmd));
        }

        #[test]
        fn test_no_match_valid_command() {
            let rule = HostsCli::new();
            let cmd = Command::new("hostscli block facebook.com", "Blocked");
            assert!(!rule.is_match(&cmd));
        }

        #[test]
        fn test_get_new_command_typo() {
            let rule = HostsCli::new();
            let cmd = Command::new(
                "hostscli blok facebook.com",
                "Error: No such command \"blok\"",
            );
            let fixes = rule.get_new_command(&cmd);
            assert!(!fixes.is_empty());
            assert!(fixes[0].contains("block"));
        }

        #[test]
        fn test_get_new_command_website_error() {
            let rule = HostsCli::new();
            let cmd = Command::new(
                "hostscli block_all somefile",
                "hostscli.errors.WebsiteImportError: ...",
            );
            let fixes = rule.get_new_command(&cmd);
            assert_eq!(fixes, vec!["hostscli websites"]);
        }
    }

    mod integration {
        use super::*;

        #[test]
        fn test_all_rules_not_empty() {
            let rules = all_rules();
            assert_eq!(rules.len(), 10);
        }

        #[test]
        fn test_all_rules_have_unique_names() {
            let rules = all_rules();
            let mut names: Vec<&str> = rules.iter().map(|r| r.name()).collect();
            let original_len = names.len();
            names.sort();
            names.dedup();
            assert_eq!(names.len(), original_len, "Rule names should be unique");
        }

        #[test]
        fn test_all_rules_have_names() {
            let rules = all_rules();
            for rule in rules {
                assert!(!rule.name().is_empty());
            }
        }
    }
}
