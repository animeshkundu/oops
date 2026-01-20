//! Sudo rule for permission denied errors.
//!
//! This rule matches commands that fail due to permission errors and
//! suggests running them with `sudo`.

use crate::core::{Command, Rule};

/// Patterns that indicate a permission error.
const PERMISSION_PATTERNS: &[&str] = &[
    "Permission denied",
    "EACCES",
    "permission denied",
    "Operation not permitted",
    "you cannot perform this operation unless you are root",
    "must be root",
    "need to be root",
    "needs to be run as root",
    "requires superuser privileges",
    "requires root",
    "Access denied",
    "access denied",
    "must have root privileges",
    "This operation requires root",
    "Unable to write",
    "Cannot open",
    "Read-only file system",
    // Linux/Unix specific
    "only root can",
    "must be superuser",
    "you need root privileges",
    "insufficient permissions",
    // Package manager specific messages
    "are you root?",
    "Please run as root",
    "not allowed to perform this operation",
];

/// Commands that should not be prefixed with sudo.
const EXCLUDED_COMMANDS: &[&str] = &[
    "sudo",  // Already has sudo
    "su",    // Switching user
    "pkexec", // PolicyKit
    "doas",  // OpenBSD/Alpine sudo alternative
    "runas", // Windows equivalent
];

/// Rule that suggests adding `sudo` to commands that fail with permission errors.
///
/// # Example
///
/// ```
/// use oops::rules::sudo::Sudo;
/// use oops::core::{Command, Rule};
///
/// let rule = Sudo;
/// let cmd = Command::new("apt install vim", "E: Could not open lock file - Permission denied");
/// assert!(rule.is_match(&cmd));
/// assert_eq!(rule.get_new_command(&cmd), vec!["sudo apt install vim"]);
/// ```
#[derive(Debug, Clone, Copy, Default)]
pub struct Sudo;

impl Rule for Sudo {
    fn name(&self) -> &str {
        "sudo"
    }

    fn priority(&self) -> i32 {
        // High priority - permission errors are common and the fix is simple
        50
    }

    fn is_match(&self, cmd: &Command) -> bool {
        // Check if command already starts with sudo or equivalent
        let parts = cmd.script_parts();
        if let Some(first) = parts.first() {
            let first_lower = first.to_lowercase();
            // Extract just the command name (handle paths)
            let cmd_name = first_lower
                .rsplit(['/', '\\'])
                .next()
                .unwrap_or(&first_lower)
                .trim_end_matches(".exe");

            if EXCLUDED_COMMANDS.contains(&cmd_name) {
                return false;
            }
        }

        // Check if output contains any permission error pattern
        PERMISSION_PATTERNS
            .iter()
            .any(|pattern| cmd.output.contains(pattern))
    }

    fn get_new_command(&self, cmd: &Command) -> Vec<String> {
        // Check for env vars that need to be preserved
        let script = &cmd.script;

        // If the command uses env vars that need preservation, use sudo -E
        let needs_env = script.contains("$") || script.contains("${");

        if needs_env {
            vec![format!("sudo -E {}", script)]
        } else {
            vec![format!("sudo {}", script)]
        }
    }

    fn enabled_by_default(&self) -> bool {
        true
    }

    fn requires_output(&self) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sudo_name() {
        let rule = Sudo;
        assert_eq!(rule.name(), "sudo");
    }

    #[test]
    fn test_sudo_priority() {
        let rule = Sudo;
        assert_eq!(rule.priority(), 50);
    }

    #[test]
    fn test_matches_permission_denied() {
        let rule = Sudo;
        let cmd = Command::new("apt install vim", "E: Could not open lock file - Permission denied");
        assert!(rule.is_match(&cmd));
    }

    #[test]
    fn test_matches_eacces() {
        let rule = Sudo;
        let cmd = Command::new("touch /etc/test", "touch: cannot touch '/etc/test': EACCES");
        assert!(rule.is_match(&cmd));
    }

    #[test]
    fn test_matches_operation_not_permitted() {
        let rule = Sudo;
        let cmd = Command::new(
            "rm /protected/file",
            "rm: cannot remove '/protected/file': Operation not permitted",
        );
        assert!(rule.is_match(&cmd));
    }

    #[test]
    fn test_matches_must_be_root() {
        let rule = Sudo;
        let cmd = Command::new("systemctl restart nginx", "Error: you must be root to run this command");
        assert!(rule.is_match(&cmd));
    }

    #[test]
    fn test_matches_are_you_root() {
        let rule = Sudo;
        let cmd = Command::new("dnf install package", "Error: This command has to be run under the root user - are you root?");
        assert!(rule.is_match(&cmd));
    }

    #[test]
    fn test_no_match_success() {
        let rule = Sudo;
        let cmd = Command::new("ls /home", "file1  file2  file3");
        assert!(!rule.is_match(&cmd));
    }

    #[test]
    fn test_no_match_different_error() {
        let rule = Sudo;
        let cmd = Command::new("git push", "error: failed to push some refs");
        assert!(!rule.is_match(&cmd));
    }

    #[test]
    fn test_no_match_already_sudo() {
        let rule = Sudo;
        let cmd = Command::new("sudo apt install vim", "Permission denied");
        assert!(!rule.is_match(&cmd));
    }

    #[test]
    fn test_no_match_su_command() {
        let rule = Sudo;
        let cmd = Command::new("su - root", "Permission denied");
        assert!(!rule.is_match(&cmd));
    }

    #[test]
    fn test_no_match_doas() {
        let rule = Sudo;
        let cmd = Command::new("doas apt install vim", "Permission denied");
        assert!(!rule.is_match(&cmd));
    }

    #[test]
    fn test_get_new_command_simple() {
        let rule = Sudo;
        let cmd = Command::new("apt install vim", "Permission denied");
        let fixes = rule.get_new_command(&cmd);
        assert_eq!(fixes, vec!["sudo apt install vim"]);
    }

    #[test]
    fn test_get_new_command_with_args() {
        let rule = Sudo;
        let cmd = Command::new("systemctl restart nginx.service", "Permission denied");
        let fixes = rule.get_new_command(&cmd);
        assert_eq!(fixes, vec!["sudo systemctl restart nginx.service"]);
    }

    #[test]
    fn test_get_new_command_preserves_env() {
        let rule = Sudo;
        let cmd = Command::new("install -m 755 $HOME/.local/bin/tool", "Permission denied");
        let fixes = rule.get_new_command(&cmd);
        assert_eq!(fixes, vec!["sudo -E install -m 755 $HOME/.local/bin/tool"]);
    }

    #[test]
    fn test_get_new_command_preserves_env_braces() {
        let rule = Sudo;
        let cmd = Command::new("echo ${PATH} > /etc/profile.d/path.sh", "Permission denied");
        let fixes = rule.get_new_command(&cmd);
        assert_eq!(
            fixes,
            vec!["sudo -E echo ${PATH} > /etc/profile.d/path.sh"]
        );
    }

    #[test]
    fn test_enabled_by_default() {
        let rule = Sudo;
        assert!(rule.enabled_by_default());
    }

    #[test]
    fn test_requires_output() {
        let rule = Sudo;
        assert!(rule.requires_output());
    }

    #[test]
    fn test_case_insensitive_permission() {
        let rule = Sudo;
        // Lowercase "permission denied" should also match
        let cmd = Command::new("cat /etc/shadow", "cat: /etc/shadow: permission denied");
        assert!(rule.is_match(&cmd));
    }

    #[test]
    fn test_access_denied() {
        let rule = Sudo;
        let cmd = Command::new("docker ps", "Got permission denied while trying to connect to the Docker daemon");
        assert!(rule.is_match(&cmd));
    }
}
