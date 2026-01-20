# Architecture Overview

oops is designed as a fast, cross-platform command-line correction tool.

## System Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                         User's Shell                             │
│  ┌──────────┐    ┌──────────┐    ┌──────────┐                   │
│  │  Bash    │    │   Zsh    │    │   Fish   │    ...            │
│  └────┬─────┘    └────┬─────┘    └────┬─────┘                   │
│       │               │               │                          │
│       └───────────────┼───────────────┘                          │
│                       │                                          │
│                       ▼                                          │
│              ┌────────────────┐                                  │
│              │  Shell Alias   │  (oops function)                 │
│              └────────┬───────┘                                  │
└───────────────────────┼─────────────────────────────────────────┘
                        │
                        ▼
┌─────────────────────────────────────────────────────────────────┐
│                         oops Binary                              │
│                                                                  │
│  ┌──────────┐    ┌──────────────┐    ┌──────────────────┐       │
│  │   CLI    │───▶│    Core      │───▶│   Rule Engine    │       │
│  │ (clap)   │    │  (Command)   │    │  (175+ rules)    │       │
│  └──────────┘    └──────────────┘    └──────────────────┘       │
│       │                │                      │                  │
│       │                │                      ▼                  │
│       │                │           ┌──────────────────┐          │
│       │                │           │  Corrected       │          │
│       │                │           │  Commands        │          │
│       │                │           └────────┬─────────┘          │
│       │                │                    │                    │
│       ▼                ▼                    ▼                    │
│  ┌──────────┐    ┌──────────────┐    ┌──────────────┐           │
│  │  Config  │    │   Output     │    │     UI       │           │
│  │  Loader  │    │   Capture    │    │  (selector)  │           │
│  └──────────┘    └──────────────┘    └──────────────┘           │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

## Core Components

### 1. CLI Layer (`src/cli.rs`)

Handles command-line argument parsing using clap:
- `--alias` - Generate shell integration code
- `-y/--yes` - Auto-execute first suggestion
- `-r/--repeat` - Retry on failure
- `--debug` - Enable debug output

### 2. Core Engine (`src/core/`)

The heart of oops:

- **Command** (`command.rs`) - Represents a failed command with its script and output
- **Rule** (`rule.rs`) - Trait defining how rules match and correct commands
- **CorrectedCommand** (`corrected.rs`) - A suggested correction with priority
- **Corrector** (`corrector.rs`) - Matches commands against rules

### 3. Rules (`src/rules/`)

175+ rules organized by category:
- `git/` - Git operations
- `package_managers/` - Package manager commands
- `system.rs` - File and system operations
- `cloud.rs` - Cloud CLIs (AWS, Azure)
- And more...

### 4. Shells (`src/shells/`)

Shell-specific implementations:
- Alias generation
- History retrieval
- Command combination (`&&`, `;`)
- Built-in command detection

### 5. Configuration (`src/config/`)

Layered configuration:
1. Default values
2. Config file (`~/.config/oops/config.toml`)
3. Environment variables (`THEFUCK_*`)
4. CLI arguments

### 6. Output (`src/output/`)

Command execution and output capture:
- Runs failed commands to get output
- Handles timeouts for slow commands
- Captures stderr and stdout

### 7. UI (`src/ui/`)

Terminal user interface:
- Command selector with navigation
- Colored output
- Keyboard input handling

## Data Flow

1. **User types wrong command** → Shell executes and fails
2. **User types `oops`** → Shell alias captures last command
3. **oops binary starts** → Parses arguments, loads config
4. **Command created** → Original script + output captured
5. **Rules evaluated** → Each rule checks if it matches
6. **Corrections generated** → Matching rules produce suggestions
7. **UI displayed** → User selects a correction
8. **Correction executed** → Shell runs the corrected command

## Performance Considerations

- **Lazy evaluation**: Rules are evaluated only until enough matches found
- **Early exit**: Rules can skip expensive checks if basic patterns don't match
- **Caching**: Executable lookups and fuzzy matches are cached
- **Single binary**: No startup overhead from interpreters or package loading

## Thread Safety

All rules implement `Send + Sync` for potential parallel evaluation:
```rust
pub trait Rule: Send + Sync {
    // ...
}
```

## Error Handling

Uses `anyhow` for error propagation and `thiserror` for custom error types:
- Configuration errors
- Shell detection failures
- Command execution timeouts
