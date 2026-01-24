# Rules Catalog

This guide summarizes the correction rules that ship with oops and how to
explore them in the codebase.

## Coverage Snapshot

Oops ships with 175+ rules covering the most common CLI mistakes. Rules are
compiled into the binary for speed and are evaluated lazily.

| Category | Examples | Notes |
| --- | --- | --- |
| Git | `git_push`, `git_checkout`, `git_commit_amend` | Branching, pushing, and commit flow |
| Package managers | `brew`, `npm`, `pip`, `cargo`, `apt` | Install, run, and script suggestions |
| System | `sudo`, `cd_mkdir`, `chmod_x`, `mkdir_p` | Permissions and file ops |
| Dev tools | `terraform`, `docker`, `kubectl`, `gradle` | Infrastructure and build workflows |
| Frameworks | `react_native`, `rails`, `django` | Framework-specific errors |
| Shell utilities | `grep`, `tar`, `history` | Common CLI helper commands |

## Where Rules Live

All rules are defined under `src/rules/`:

```
src/rules/
├── git/
├── package_managers/
├── docker/
├── devtools.rs
├── frameworks.rs
├── cloud.rs
├── shell_utils.rs
└── misc.rs
```

Each rule implements the `Rule` trait and is registered in
`src/rules/mod.rs`. See the [creating rules guide](creating-rules.md) for
implementation details.

## Inspecting Rule Coverage

Use the parity checker to compare coverage against thefuck:

```bash
cargo run --bin check_parity -- --output json
```

## Adding More Rules

If you spot a missing correction:

1. Check the parity report.
2. Review the original thefuck rule (if applicable).
3. Follow the [contributing rules guide](contributing-rules.md).
4. Add tests for each rule.

Need help? Open a discussion on GitHub or read the migration guide.
