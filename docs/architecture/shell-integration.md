# Shell Integration

oops integrates with various shells through generated alias functions.

## How It Works

1. User runs `oops --alias` to generate shell-specific code
2. User adds the output to their shell config
3. Shell creates a function that:
   - Captures the last command
   - Calls oops with the command
   - Executes the corrected command

## Supported Shells

### Bash

Generated alias:
```bash
oops () {
    TF_PYTHONIOENCODING=$PYTHONIOENCODING;
    export TF_SHELL=bash;
    export TF_ALIAS=oops;
    export TF_SHELL_ALIASES=$(alias);
    export TF_HISTORY=$(fc -ln -10);
    export PYTHONIOENCODING=utf-8;
    TF_CMD=$(
        oops THEFUCK_ARGUMENT_PLACEHOLDER "$@"
    ) && eval "$TF_CMD";
    unset TF_CMD;
    export PYTHONIOENCODING=$TF_PYTHONIOENCODING;
}
```

Key features:
- Uses `fc -ln -10` for recent history
- Exports aliases via `alias` command
- Evaluates corrected command with `eval`

### Zsh

Generated alias:
```zsh
oops () {
    export TF_SHELL=zsh;
    export TF_ALIAS=oops;
    export TF_SHELL_ALIASES=$(alias);
    export TF_HISTORY=$(fc -ln -10);
    TF_CMD=$(
        oops THEFUCK_ARGUMENT_PLACEHOLDER "$@"
    ) && eval "$TF_CMD";
    unset TF_CMD;
}
```

Key features:
- Similar to bash but with zsh-specific history format
- Extended history format support (`: timestamp:0;command`)

### Fish

Generated alias:
```fish
function oops
    set -x TF_SHELL fish
    set -x TF_ALIAS oops
    set -x TF_SHELL_ALIASES (alias)
    set -x TF_HISTORY $history[1]
    set -l TF_CMD (oops THEFUCK_ARGUMENT_PLACEHOLDER $argv)
    if test -n "$TF_CMD"
        eval $TF_CMD
    end
end
```

Key features:
- Uses fish's `$history[1]` for last command
- Uses `functions` and `alias` for shell functions

### PowerShell

Generated alias:
```powershell
function oops {
    $env:TF_SHELL = "powershell"
    $env:TF_ALIAS = "oops"
    $env:TF_HISTORY = (Get-History -Count 1).CommandLine
    $TF_CMD = & oops THEFUCK_ARGUMENT_PLACEHOLDER $args
    if ($TF_CMD) {
        Invoke-Expression $TF_CMD
    }
}
```

Key features:
- Uses `Get-History -Count 1` for last command
- Command combination uses `-and` operator

### Tcsh

Generated alias:
```tcsh
alias oops 'set TF_CMD = `oops THEFUCK_ARGUMENT_PLACEHOLDER \!*` && eval "$TF_CMD"'
```

Key features:
- Uses tcsh alias syntax
- History accessed via `history -h 2`

## Environment Variables

The shell alias sets these variables:

| Variable | Purpose |
|----------|---------|
| `TF_SHELL` | Current shell name |
| `TF_ALIAS` | Alias name (default: oops) |
| `TF_SHELL_ALIASES` | Shell alias definitions |
| `TF_HISTORY` | Recent command history |

## Shell Trait

Each shell implements the `Shell` trait:

```rust
pub trait Shell: Send + Sync {
    /// Shell name
    fn name(&self) -> &str;

    /// Generate alias function
    fn app_alias(&self, alias_name: &str, instant_mode: bool) -> String;

    /// Get command history
    fn get_history(&self) -> Vec<String>;

    /// Get shell aliases
    fn get_aliases(&self) -> HashMap<String, String>;

    /// Combine commands with AND
    fn and_(&self, commands: &[&str]) -> String;

    /// Combine commands with OR
    fn or_(&self, commands: &[&str]) -> String;

    /// Add command to history
    fn put_to_history(&self, command: &str) -> Result<()>;

    /// Get builtin commands
    fn get_builtin_commands(&self) -> &[&str];

    /// History file path
    fn get_history_file_name(&self) -> Option<String>;
}
```

## Shell Detection

oops detects the current shell:

1. Check `TF_SHELL` environment variable
2. On Unix: Inspect process parent tree
3. Default: Bash

```rust
pub fn detect_shell() -> Box<dyn Shell> {
    if let Ok(shell) = std::env::var("TF_SHELL") {
        return match shell.as_str() {
            "bash" => Box::new(Bash),
            "zsh" => Box::new(Zsh),
            "fish" => Box::new(Fish),
            "powershell" | "pwsh" => Box::new(PowerShell),
            "tcsh" | "csh" => Box::new(Tcsh),
            _ => Box::new(Bash),
        };
    }
    // ... fallback detection
}
```

## Command Combination

Different shells combine commands differently:

| Shell | AND | OR |
|-------|-----|-----|
| Bash/Zsh | `cmd1 && cmd2` | `cmd1 \|\| cmd2` |
| Fish | `cmd1; and cmd2` | `cmd1; or cmd2` |
| PowerShell | `(cmd1) -and (cmd2)` | `(cmd1) -or (cmd2)` |
| Tcsh | `cmd1 && cmd2` | `cmd1 \|\| cmd2` |

## History File Locations

| Shell | Default Location |
|-------|------------------|
| Bash | `~/.bash_history` or `$HISTFILE` |
| Zsh | `~/.zsh_history` or `$HISTFILE` |
| Fish | `~/.local/share/fish/fish_history` |
| PowerShell | Via `Get-History` cmdlet |
| Tcsh | `~/.history` or `$HISTFILE` |
