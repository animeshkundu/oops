//! PowerShell integration
//!
//! Provides the PowerShell implementation of the Shell trait, including:
//! - Alias generation for the `oops` command
//! - History reading from Get-History cmdlet (via environment)
//! - No TF_SHELL_ALIASES support (PowerShell handles aliases differently)

use std::collections::HashMap;
use std::env;

use anyhow::Result;

use super::Shell;

/// PowerShell implementation.
///
/// This works with both Windows PowerShell and PowerShell Core (pwsh).
#[derive(Debug, Clone, Default)]
pub struct PowerShell;

impl PowerShell {
    /// Creates a new PowerShell instance.
    pub fn new() -> Self {
        Self
    }

    /// Gets the history file path for PowerShell.
    fn get_history_file(&self) -> String {
        // PowerShell history location varies by platform and version
        // Windows: $env:APPDATA\Microsoft\Windows\PowerShell\PSReadLine\ConsoleHost_history.txt
        // Cross-platform PSReadLine: ~/.local/share/powershell/PSReadLine/ConsoleHost_history.txt

        #[cfg(windows)]
        {
            env::var("APPDATA")
                .map(|appdata| {
                    format!(
                        "{}\\Microsoft\\Windows\\PowerShell\\PSReadLine\\ConsoleHost_history.txt",
                        appdata
                    )
                })
                .unwrap_or_default()
        }

        #[cfg(not(windows))]
        {
            dirs::data_local_dir()
                .map(|p| {
                    p.join("powershell")
                        .join("PSReadLine")
                        .join("ConsoleHost_history.txt")
                })
                .and_then(|p| p.to_str().map(String::from))
                .unwrap_or_default()
        }
    }
}

impl Shell for PowerShell {
    fn name(&self) -> &str {
        "powershell"
    }

    fn app_alias(&self, alias_name: &str, _instant_mode: bool) -> String {
        // PowerShell function that:
        // 1. Gets the last command from history
        // 2. Passes it to oops
        // 3. Executes the correction or echoes it
        // 4. Resets console colors
        format!(
            r#"function {name} {{
    $history = (Get-History -Count 1).CommandLine;
    if (-not [string]::IsNullOrWhiteSpace($history)) {{
        $fuck = $(oops $args $history);
        if (-not [string]::IsNullOrWhiteSpace($fuck)) {{
            if ($fuck.StartsWith("echo")) {{ $fuck = $fuck.Substring(5); }}
            else {{ iex "$fuck"; }}
        }}
    }}
    [Console]::ResetColor()
}}
"#,
            name = alias_name
        )
    }

    fn get_history(&self) -> Vec<String> {
        // PowerShell gets history via Get-History cmdlet in the alias
        // It doesn't use TF_HISTORY environment variable
        Vec::new()
    }

    fn get_aliases(&self) -> HashMap<String, String> {
        // PowerShell doesn't use TF_SHELL_ALIASES
        // Aliases are handled internally by PowerShell
        HashMap::new()
    }

    fn and_(&self, commands: &[&str]) -> String {
        // PowerShell uses -and operator for combining commands
        // Each command must be wrapped in parentheses
        commands
            .iter()
            .map(|c| format!("({})", c))
            .collect::<Vec<_>>()
            .join(" -and ")
    }

    fn or_(&self, commands: &[&str]) -> String {
        // PowerShell uses -or operator for combining commands
        // Each command must be wrapped in parentheses
        commands
            .iter()
            .map(|c| format!("({})", c))
            .collect::<Vec<_>>()
            .join(" -or ")
    }

    fn put_to_history(&self, _command: &str) -> Result<()> {
        // PowerShell manages its own history via PSReadLine
        // We don't manually modify the history file
        Ok(())
    }

    fn get_history_file_name(&self) -> Option<String> {
        let path = self.get_history_file();
        if path.is_empty() {
            None
        } else {
            Some(path)
        }
    }

    fn get_builtin_commands(&self) -> &[&str] {
        // PowerShell has different built-in commands (cmdlets)
        &[
            "Add-Content", "Clear-Content", "Clear-Host", "Clear-Item",
            "Clear-Variable", "Compare-Object", "Copy-Item", "Export-Csv",
            "ForEach-Object", "Format-List", "Format-Table", "Get-Alias",
            "Get-ChildItem", "Get-Command", "Get-Content", "Get-Date",
            "Get-Help", "Get-History", "Get-Item", "Get-Location", "Get-Member",
            "Get-Process", "Get-Service", "Get-Variable", "Import-Csv",
            "Import-Module", "Invoke-Command", "Invoke-Expression", "Invoke-WebRequest",
            "Measure-Object", "Move-Item", "New-Item", "Out-File", "Out-Host",
            "Out-String", "Read-Host", "Remove-Item", "Remove-Variable",
            "Rename-Item", "Select-Object", "Select-String", "Set-Content",
            "Set-Item", "Set-Location", "Set-Variable", "Sort-Object",
            "Split-Path", "Start-Process", "Start-Service", "Stop-Process",
            "Stop-Service", "Test-Path", "Where-Object", "Write-Error",
            "Write-Host", "Write-Output", "Write-Warning",
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_powershell_name() {
        let ps = PowerShell::new();
        assert_eq!(ps.name(), "powershell");
    }

    #[test]
    fn test_powershell_and_operator() {
        let ps = PowerShell::new();
        assert_eq!(ps.and_(&["cmd1", "cmd2"]), "(cmd1) -and (cmd2)");
        assert_eq!(
            ps.and_(&["cmd1", "cmd2", "cmd3"]),
            "(cmd1) -and (cmd2) -and (cmd3)"
        );
    }

    #[test]
    fn test_powershell_or_operator() {
        let ps = PowerShell::new();
        assert_eq!(ps.or_(&["cmd1", "cmd2"]), "(cmd1) -or (cmd2)");
        assert_eq!(
            ps.or_(&["cmd1", "cmd2", "cmd3"]),
            "(cmd1) -or (cmd2) -or (cmd3)"
        );
    }

    #[test]
    fn test_powershell_alias_generation() {
        let ps = PowerShell::new();
        let alias = ps.app_alias("fuck", false);
        assert!(alias.contains("function fuck"));
        assert!(alias.contains("Get-History -Count 1"));
        assert!(alias.contains("oops $args $history"));
        assert!(alias.contains("iex"));
        assert!(alias.contains("[Console]::ResetColor()"));
    }

    #[test]
    fn test_powershell_alias_custom_name() {
        let ps = PowerShell::new();
        let alias = ps.app_alias("oops", false);
        assert!(alias.contains("function oops"));
    }

    #[test]
    fn test_powershell_empty_history() {
        let ps = PowerShell::new();
        let history = ps.get_history();
        assert!(history.is_empty());
    }

    #[test]
    fn test_powershell_empty_aliases() {
        let ps = PowerShell::new();
        let aliases = ps.get_aliases();
        assert!(aliases.is_empty());
    }

    #[test]
    fn test_builtin_commands() {
        let ps = PowerShell::new();
        let builtins = ps.get_builtin_commands();
        assert!(builtins.contains(&"Get-ChildItem"));
        assert!(builtins.contains(&"Set-Location"));
        assert!(builtins.contains(&"Get-History"));
    }
}
