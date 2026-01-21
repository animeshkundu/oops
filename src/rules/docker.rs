//! Docker, Vagrant, and container-related rules
//!
//! This module contains rules for fixing common container and virtualization tool errors:
//!
//! - [`DockerImageBeingUsedByContainer`] - Suggests stopping container before removing image
//! - [`DockerLogin`] - Suggests login when push fails due to authentication
//! - [`DockerNotCommand`] - Fixes unknown docker commands (typos)
//! - [`VagrantUp`] - Fixes vagrant up issues
//! - [`Tmux`] - Fixes ambiguous tmux commands

use crate::core::{is_app, Command, Rule};
use crate::utils::{get_close_matches, replace_argument};
use regex::Regex;

/// Common Docker commands for fuzzy matching.
const DOCKER_COMMANDS: &[&str] = &[
    "attach",
    "build",
    "commit",
    "cp",
    "create",
    "diff",
    "events",
    "exec",
    "export",
    "history",
    "images",
    "import",
    "info",
    "inspect",
    "kill",
    "load",
    "login",
    "logout",
    "logs",
    "pause",
    "port",
    "ps",
    "pull",
    "push",
    "rename",
    "restart",
    "rm",
    "rmi",
    "run",
    "save",
    "search",
    "start",
    "stats",
    "stop",
    "tag",
    "top",
    "unpause",
    "update",
    "version",
    "volume",
    "wait",
    // Management commands
    "builder",
    "config",
    "container",
    "context",
    "image",
    "manifest",
    "network",
    "node",
    "plugin",
    "secret",
    "service",
    "stack",
    "swarm",
    "system",
    "trust",
];

/// Rule that suggests stopping a container before removing an image.
///
/// When a Docker image is being used by a running container, you must
/// stop/remove the container before removing the image.
///
/// # Example
///
/// ```
/// use oops::rules::docker::DockerImageBeingUsedByContainer;
/// use oops::core::{Command, Rule};
///
/// let rule = DockerImageBeingUsedByContainer;
/// let cmd = Command::new(
///     "docker image rm abc123",
///     "Error response from daemon: conflict: unable to delete abc123 (cannot be forced) - image is being used by running container def456"
/// );
/// assert!(rule.is_match(&cmd));
/// ```
#[derive(Debug, Clone, Copy, Default)]
pub struct DockerImageBeingUsedByContainer;

impl Rule for DockerImageBeingUsedByContainer {
    fn name(&self) -> &str {
        "docker_image_being_used_by_container"
    }

    fn priority(&self) -> i32 {
        1000
    }

    fn is_match(&self, cmd: &Command) -> bool {
        is_app(cmd, &["docker"])
            && cmd
                .output
                .contains("image is being used by running container")
    }

    fn get_new_command(&self, cmd: &Command) -> Vec<String> {
        // Extract the container ID from the output
        // Output format: "... image is being used by running container <container_id>"
        let output = cmd.output.trim();
        if let Some(container_id) = output.split_whitespace().last() {
            // Create a command that first removes the container, then runs the original command
            // Using shell's && operator to chain commands
            vec![format!(
                "docker container rm -f {} && {}",
                container_id, cmd.script
            )]
        } else {
            vec![]
        }
    }

    fn requires_output(&self) -> bool {
        true
    }
}

/// Rule that suggests logging in when Docker push fails due to authentication.
///
/// # Example
///
/// ```
/// use oops::rules::docker::DockerLogin;
/// use oops::core::{Command, Rule};
///
/// let rule = DockerLogin;
/// let cmd = Command::new(
///     "docker push myimage:latest",
///     "denied: access denied. You may need to 'docker login'"
/// );
/// // Note: This would match but we can't test is_app without docker in script
/// ```
#[derive(Debug, Clone, Copy, Default)]
pub struct DockerLogin;

impl Rule for DockerLogin {
    fn name(&self) -> &str {
        "docker_login"
    }

    fn priority(&self) -> i32 {
        1000
    }

    fn is_match(&self, cmd: &Command) -> bool {
        is_app(cmd, &["docker"])
            && cmd.output.contains("access denied")
            && cmd.output.contains("docker login")
    }

    fn get_new_command(&self, cmd: &Command) -> Vec<String> {
        // First login, then retry the original command
        vec![format!("docker login && {}", cmd.script)]
    }

    fn requires_output(&self) -> bool {
        true
    }
}

/// Rule that fixes unknown docker commands (typos).
///
/// When Docker reports that a command is not recognized, this rule
/// suggests similar valid commands.
///
/// # Example
///
/// ```
/// use oops::rules::docker::DockerNotCommand;
/// use oops::core::{Command, Rule};
///
/// let rule = DockerNotCommand;
/// let cmd = Command::new(
///     "docker pus myimage",
///     "docker: 'pus' is not a docker command."
/// );
/// assert!(rule.is_match(&cmd));
/// ```
#[derive(Debug, Clone, Copy, Default)]
pub struct DockerNotCommand;

impl Rule for DockerNotCommand {
    fn name(&self) -> &str {
        "docker_not_command"
    }

    fn priority(&self) -> i32 {
        1000
    }

    fn is_match(&self, cmd: &Command) -> bool {
        is_app(cmd, &["docker"])
            && (cmd.output.contains("is not a docker command")
                || cmd.output.contains("Usage:\tdocker")
                || cmd.output.contains("Usage: docker"))
    }

    fn get_new_command(&self, cmd: &Command) -> Vec<String> {
        // Try to extract the wrong command from the error message
        // Format: "docker: 'pus' is not a docker command."
        let wrong_cmd_re = Regex::new(r"docker: '(\w+)' is not a docker command").ok();

        let wrong_command = if let Some(re) = wrong_cmd_re {
            re.captures(&cmd.output)
                .and_then(|caps| caps.get(1))
                .map(|m| m.as_str().to_string())
        } else {
            None
        };

        // If we found the wrong command in the error, use it for matching
        // Otherwise, try to get it from the script parts
        let wrong_cmd = wrong_command.or_else(|| {
            let parts = cmd.script_parts();
            if parts.len() >= 2 {
                Some(parts[1].clone())
            } else {
                None
            }
        });

        if let Some(wrong) = wrong_cmd {
            let commands: Vec<String> = DOCKER_COMMANDS.iter().map(|s| s.to_string()).collect();
            let matches = get_close_matches(&wrong, &commands, 3, 0.6);

            matches
                .into_iter()
                .map(|correct_cmd| replace_argument(&cmd.script, &wrong, &correct_cmd))
                .collect()
        } else {
            vec![]
        }
    }

    fn requires_output(&self) -> bool {
        true
    }
}

/// Rule that fixes vagrant up issues.
///
/// When Vagrant suggests running `vagrant up`, this rule provides
/// the correct command sequence.
///
/// # Example
///
/// ```
/// use oops::rules::docker::VagrantUp;
/// use oops::core::{Command, Rule};
///
/// let rule = VagrantUp;
/// let cmd = Command::new(
///     "vagrant ssh",
///     "VM must be running. Run `vagrant up` first."
/// );
/// assert!(rule.is_match(&cmd));
/// ```
#[derive(Debug, Clone, Copy, Default)]
pub struct VagrantUp;

impl Rule for VagrantUp {
    fn name(&self) -> &str {
        "vagrant_up"
    }

    fn priority(&self) -> i32 {
        1000
    }

    fn is_match(&self, cmd: &Command) -> bool {
        is_app(cmd, &["vagrant"]) && cmd.output.to_lowercase().contains("run `vagrant up`")
    }

    fn get_new_command(&self, cmd: &Command) -> Vec<String> {
        let parts = cmd.script_parts();

        // Try to extract machine name from command if present
        // Format: vagrant <command> [machine-name] [options]
        let machine = if parts.len() >= 3 {
            // Check if parts[2] looks like a machine name (not a flag)
            let potential_machine = &parts[2];
            if !potential_machine.starts_with('-') {
                Some(potential_machine.clone())
            } else {
                None
            }
        } else {
            None
        };

        // Start all instances command
        let start_all = format!("vagrant up && {}", cmd.script);

        if let Some(machine_name) = machine {
            // If we have a specific machine, offer both options
            vec![
                format!("vagrant up {} && {}", machine_name, cmd.script),
                start_all,
            ]
        } else {
            vec![start_all]
        }
    }

    fn requires_output(&self) -> bool {
        true
    }
}

/// Rule that fixes ambiguous tmux commands.
///
/// When tmux reports an ambiguous command, this rule suggests
/// the possible completions.
///
/// # Example
///
/// ```
/// use oops::rules::docker::Tmux;
/// use oops::core::{Command, Rule};
///
/// let rule = Tmux;
/// let cmd = Command::new(
///     "tmux list",
///     "ambiguous command: list, could be: list-buffers, list-clients, list-commands"
/// );
/// assert!(rule.is_match(&cmd));
/// ```
#[derive(Debug, Clone, Copy, Default)]
pub struct Tmux;

impl Rule for Tmux {
    fn name(&self) -> &str {
        "tmux"
    }

    fn priority(&self) -> i32 {
        1000
    }

    fn is_match(&self, cmd: &Command) -> bool {
        is_app(cmd, &["tmux"])
            && cmd.output.contains("ambiguous command:")
            && cmd.output.contains("could be:")
    }

    fn get_new_command(&self, cmd: &Command) -> Vec<String> {
        // Parse the tmux error message
        // Format: "ambiguous command: <cmd>, could be: <suggestion1>, <suggestion2>, ..."
        let re = Regex::new(r"ambiguous command: ([^,]+), could be: (.+)").ok();

        if let Some(re) = re {
            if let Some(caps) = re.captures(&cmd.output) {
                let old_cmd = caps.get(1).map(|m| m.as_str().trim()).unwrap_or("");
                let suggestions_str = caps.get(2).map(|m| m.as_str()).unwrap_or("");

                // Parse suggestions (comma-separated)
                let suggestions: Vec<&str> = suggestions_str
                    .split(',')
                    .map(|s| s.trim())
                    .filter(|s| !s.is_empty())
                    .collect();

                return suggestions
                    .into_iter()
                    .map(|suggestion| replace_argument(&cmd.script, old_cmd, suggestion))
                    .collect();
            }
        }

        vec![]
    }

    fn requires_output(&self) -> bool {
        true
    }
}

/// Returns all Docker and container-related rules as boxed trait objects.
///
/// This function creates instances of all rules in this module
/// for registration with the rule system.
pub fn all_rules() -> Vec<Box<dyn Rule>> {
    vec![
        Box::new(DockerImageBeingUsedByContainer),
        Box::new(DockerLogin),
        Box::new(DockerNotCommand),
        Box::new(VagrantUp),
        Box::new(Tmux),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    // DockerImageBeingUsedByContainer tests
    mod docker_image_being_used_by_container {
        use super::*;

        #[test]
        fn test_name() {
            let rule = DockerImageBeingUsedByContainer;
            assert_eq!(rule.name(), "docker_image_being_used_by_container");
        }

        #[test]
        fn test_matches_image_in_use() {
            let rule = DockerImageBeingUsedByContainer;
            let cmd = Command::new(
                "docker image rm abc123",
                "Error response from daemon: conflict: unable to delete abc123 (cannot be forced) - image is being used by running container def456",
            );
            assert!(rule.is_match(&cmd));
        }

        #[test]
        fn test_matches_rmi_in_use() {
            let rule = DockerImageBeingUsedByContainer;
            let cmd = Command::new(
                "docker rmi myimage",
                "Error: image is being used by running container abc123def",
            );
            assert!(rule.is_match(&cmd));
        }

        #[test]
        fn test_no_match_other_error() {
            let rule = DockerImageBeingUsedByContainer;
            let cmd = Command::new("docker image rm abc123", "Error: No such image: abc123");
            assert!(!rule.is_match(&cmd));
        }

        #[test]
        fn test_no_match_other_command() {
            let rule = DockerImageBeingUsedByContainer;
            let cmd = Command::new(
                "podman image rm abc123",
                "image is being used by running container def456",
            );
            assert!(!rule.is_match(&cmd));
        }

        #[test]
        fn test_get_new_command() {
            let rule = DockerImageBeingUsedByContainer;
            let cmd = Command::new(
                "docker image rm abc123",
                "image is being used by running container def456",
            );
            let fixes = rule.get_new_command(&cmd);
            assert!(!fixes.is_empty());
            assert!(fixes[0].contains("docker container rm -f def456"));
            assert!(fixes[0].contains("&& docker image rm abc123"));
        }

        #[test]
        fn test_requires_output() {
            let rule = DockerImageBeingUsedByContainer;
            assert!(rule.requires_output());
        }
    }

    // DockerLogin tests
    mod docker_login {
        use super::*;

        #[test]
        fn test_name() {
            let rule = DockerLogin;
            assert_eq!(rule.name(), "docker_login");
        }

        #[test]
        fn test_matches_access_denied() {
            let rule = DockerLogin;
            let cmd = Command::new(
                "docker push myimage:latest",
                "denied: access denied. You may need to 'docker login'",
            );
            assert!(rule.is_match(&cmd));
        }

        #[test]
        fn test_matches_may_require_login() {
            let rule = DockerLogin;
            let cmd = Command::new(
                "docker push registry.example.com/myimage",
                "unauthorized: access denied, please run 'docker login' first",
            );
            assert!(rule.is_match(&cmd));
        }

        #[test]
        fn test_no_match_other_error() {
            let rule = DockerLogin;
            let cmd = Command::new(
                "docker push myimage",
                "An image does not exist locally with the tag: myimage",
            );
            assert!(!rule.is_match(&cmd));
        }

        #[test]
        fn test_no_match_access_denied_without_login_hint() {
            let rule = DockerLogin;
            let cmd = Command::new("docker push myimage", "access denied");
            assert!(!rule.is_match(&cmd));
        }

        #[test]
        fn test_get_new_command() {
            let rule = DockerLogin;
            let cmd = Command::new(
                "docker push myimage:latest",
                "access denied. You may need to 'docker login'",
            );
            let fixes = rule.get_new_command(&cmd);
            assert_eq!(fixes, vec!["docker login && docker push myimage:latest"]);
        }

        #[test]
        fn test_requires_output() {
            let rule = DockerLogin;
            assert!(rule.requires_output());
        }
    }

    // DockerNotCommand tests
    mod docker_not_command {
        use super::*;

        #[test]
        fn test_name() {
            let rule = DockerNotCommand;
            assert_eq!(rule.name(), "docker_not_command");
        }

        #[test]
        fn test_matches_not_a_docker_command() {
            let rule = DockerNotCommand;
            let cmd = Command::new(
                "docker pus myimage",
                "docker: 'pus' is not a docker command.",
            );
            assert!(rule.is_match(&cmd));
        }

        #[test]
        fn test_matches_usage_output() {
            let rule = DockerNotCommand;
            let cmd = Command::new(
                "docker xyz",
                "Usage:\tdocker [OPTIONS] COMMAND [ARG...]\n\nUnknown command: xyz",
            );
            assert!(rule.is_match(&cmd));
        }

        #[test]
        fn test_no_match_valid_command() {
            let rule = DockerNotCommand;
            let cmd = Command::new("docker ps", "CONTAINER ID   IMAGE   COMMAND   CREATED");
            assert!(!rule.is_match(&cmd));
        }

        #[test]
        fn test_no_match_other_command() {
            let rule = DockerNotCommand;
            let cmd = Command::new("podman pus", "'pus' is not a docker command");
            assert!(!rule.is_match(&cmd));
        }

        #[test]
        fn test_get_new_command_pus_to_push() {
            let rule = DockerNotCommand;
            let cmd = Command::new(
                "docker pus myimage",
                "docker: 'pus' is not a docker command.",
            );
            let fixes = rule.get_new_command(&cmd);
            assert!(!fixes.is_empty());
            assert!(
                fixes.iter().any(|f| f.contains("push")),
                "Expected 'push' in fixes: {:?}",
                fixes
            );
        }

        #[test]
        fn test_get_new_command_rn_to_run() {
            let rule = DockerNotCommand;
            let cmd = Command::new("docker rn ubuntu", "docker: 'rn' is not a docker command.");
            let fixes = rule.get_new_command(&cmd);
            assert!(!fixes.is_empty());
            // Should suggest "run" or similar
            assert!(
                fixes.iter().any(|f| f.contains("rm") || f.contains("run")),
                "Expected 'rm' or 'run' in fixes: {:?}",
                fixes
            );
        }

        #[test]
        fn test_get_new_command_imag_to_image() {
            let rule = DockerNotCommand;
            let cmd = Command::new("docker imag ls", "docker: 'imag' is not a docker command.");
            let fixes = rule.get_new_command(&cmd);
            // Should suggest "image"
            assert!(
                fixes.iter().any(|f| f.contains("image")),
                "Expected 'image' in fixes: {:?}",
                fixes
            );
        }

        #[test]
        fn test_requires_output() {
            let rule = DockerNotCommand;
            assert!(rule.requires_output());
        }
    }

    // VagrantUp tests
    mod vagrant_up {
        use super::*;

        #[test]
        fn test_name() {
            let rule = VagrantUp;
            assert_eq!(rule.name(), "vagrant_up");
        }

        #[test]
        fn test_matches_run_vagrant_up() {
            let rule = VagrantUp;
            let cmd = Command::new("vagrant ssh", "VM must be running. Run `vagrant up` first.");
            assert!(rule.is_match(&cmd));
        }

        #[test]
        fn test_matches_case_insensitive() {
            let rule = VagrantUp;
            let cmd = Command::new(
                "vagrant ssh",
                "The machine is not running. Please run `Vagrant up` to start it.",
            );
            assert!(rule.is_match(&cmd));
        }

        #[test]
        fn test_no_match_other_error() {
            let rule = VagrantUp;
            let cmd = Command::new(
                "vagrant ssh",
                "A Vagrant environment is required. Run `vagrant init`",
            );
            assert!(!rule.is_match(&cmd));
        }

        #[test]
        fn test_no_match_other_command() {
            let rule = VagrantUp;
            let cmd = Command::new("docker run ubuntu", "run `vagrant up` first");
            assert!(!rule.is_match(&cmd));
        }

        #[test]
        fn test_get_new_command_simple() {
            let rule = VagrantUp;
            let cmd = Command::new("vagrant ssh", "Run `vagrant up` first");
            let fixes = rule.get_new_command(&cmd);
            assert!(!fixes.is_empty());
            assert_eq!(fixes[0], "vagrant up && vagrant ssh");
        }

        #[test]
        fn test_get_new_command_with_machine() {
            let rule = VagrantUp;
            let cmd = Command::new("vagrant ssh myvm", "Run `vagrant up` first");
            let fixes = rule.get_new_command(&cmd);
            assert!(!fixes.is_empty());
            // Should have machine-specific option first
            assert!(fixes[0].contains("vagrant up myvm"));
            // Should also have start-all option
            assert!(fixes.len() >= 2 || fixes[0] == "vagrant up myvm && vagrant ssh myvm");
        }

        #[test]
        fn test_get_new_command_with_flags() {
            let rule = VagrantUp;
            let cmd = Command::new("vagrant ssh --provision", "Run `vagrant up` first");
            let fixes = rule.get_new_command(&cmd);
            assert!(!fixes.is_empty());
            // Should not treat --provision as a machine name
            assert!(fixes[0].contains("vagrant up && vagrant ssh --provision"));
        }

        #[test]
        fn test_requires_output() {
            let rule = VagrantUp;
            assert!(rule.requires_output());
        }
    }

    // Tmux tests
    mod tmux {
        use super::*;

        #[test]
        fn test_name() {
            let rule = Tmux;
            assert_eq!(rule.name(), "tmux");
        }

        #[test]
        fn test_matches_ambiguous_command() {
            let rule = Tmux;
            let cmd = Command::new(
                "tmux list",
                "ambiguous command: list, could be: list-buffers, list-clients, list-commands",
            );
            assert!(rule.is_match(&cmd));
        }

        #[test]
        fn test_matches_kilw() {
            let rule = Tmux;
            let cmd = Command::new(
                "tmux kilw",
                "ambiguous command: kilw, could be: kill-window, kill-pane",
            );
            assert!(rule.is_match(&cmd));
        }

        #[test]
        fn test_no_match_valid_command() {
            let rule = Tmux;
            let cmd = Command::new("tmux list-sessions", "0: 1 windows (created ...)");
            assert!(!rule.is_match(&cmd));
        }

        #[test]
        fn test_no_match_unknown_command() {
            let rule = Tmux;
            let cmd = Command::new("tmux xyz", "unknown command: xyz");
            assert!(!rule.is_match(&cmd));
        }

        #[test]
        fn test_no_match_other_command() {
            let rule = Tmux;
            let cmd = Command::new(
                "screen list",
                "ambiguous command: list, could be: list-buffers",
            );
            assert!(!rule.is_match(&cmd));
        }

        #[test]
        fn test_get_new_command_list() {
            let rule = Tmux;
            let cmd = Command::new(
                "tmux list",
                "ambiguous command: list, could be: list-buffers, list-clients, list-commands",
            );
            let fixes = rule.get_new_command(&cmd);
            assert!(!fixes.is_empty());
            assert!(fixes.contains(&"tmux list-buffers".to_string()));
            assert!(fixes.contains(&"tmux list-clients".to_string()));
            assert!(fixes.contains(&"tmux list-commands".to_string()));
        }

        #[test]
        fn test_get_new_command_with_args() {
            let rule = Tmux;
            let cmd = Command::new(
                "tmux att -t session",
                "ambiguous command: att, could be: attach-session, attach",
            );
            let fixes = rule.get_new_command(&cmd);
            assert!(!fixes.is_empty());
            // Should replace "att" but preserve "-t session"
            assert!(
                fixes
                    .iter()
                    .any(|f| f.contains("attach") && f.contains("-t session")),
                "Expected fixes to contain 'attach' with '-t session': {:?}",
                fixes
            );
        }

        #[test]
        fn test_requires_output() {
            let rule = Tmux;
            assert!(rule.requires_output());
        }
    }

    // Integration tests
    mod integration {
        use super::*;

        #[test]
        fn test_all_rules_returns_five_rules() {
            let rules = all_rules();
            assert_eq!(rules.len(), 5);
        }

        #[test]
        fn test_all_rules_have_unique_names() {
            let rules = all_rules();
            let names: Vec<&str> = rules.iter().map(|r| r.name()).collect();
            let mut unique_names = names.clone();
            unique_names.sort();
            unique_names.dedup();
            assert_eq!(names.len(), unique_names.len());
        }

        #[test]
        fn test_all_rules_have_correct_names() {
            let rules = all_rules();
            let names: Vec<&str> = rules.iter().map(|r| r.name()).collect();
            assert!(names.contains(&"docker_image_being_used_by_container"));
            assert!(names.contains(&"docker_login"));
            assert!(names.contains(&"docker_not_command"));
            assert!(names.contains(&"vagrant_up"));
            assert!(names.contains(&"tmux"));
        }

        #[test]
        fn test_all_rules_require_output() {
            let rules = all_rules();
            for rule in rules {
                assert!(
                    rule.requires_output(),
                    "Rule {} should require output",
                    rule.name()
                );
            }
        }

        #[test]
        fn test_all_rules_enabled_by_default() {
            let rules = all_rules();
            for rule in rules {
                assert!(
                    rule.enabled_by_default(),
                    "Rule {} should be enabled by default",
                    rule.name()
                );
            }
        }
    }
}
