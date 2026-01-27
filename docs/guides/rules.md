# Supported Rules

oops includes **177+ correction rules** that help fix common command-line mistakes. Rules are organized by category for easy reference.

## Table of Contents

- [Git Rules](#git-rules)
- [Package Manager Rules](#package-manager-rules)
- [System & File Rules](#system--file-rules)
- [Development Tool Rules](#development-tool-rules)
- [Cloud & DevOps Rules](#cloud--devops-rules)
- [Shell Utility Rules](#shell-utility-rules)
- [Miscellaneous Rules](#miscellaneous-rules)

---

## Git Rules

Over 50 rules for Git command corrections:

| Rule | Description |
|------|-------------|
| `git_add` | Fixes "pathspec did not match any files" errors |
| `git_add_all` | Suggests `git add -A` when appropriate |
| `git_add_force` | Adds `--force` for `.gitignore`d files |
| `git_bisect_usage` | Fixes `git bisect` subcommand typos |
| `git_branch_delete` | Changes `-d` to `-D` when branch isn't merged |
| `git_branch_delete_checked_out` | Suggests checking out different branch first |
| `git_branch_exists` | Handles "branch already exists" errors |
| `git_branch_flag_position` | Fixes flag position in branch commands |
| `git_branch_list` | Corrects `git branch list` to `git branch` |
| `git_branch_not_found` | Suggests similar branch names |
| `git_checkout` | Fixes branch name typos |
| `git_checkout_uncommitted_changes` | Suggests stashing changes first |
| `git_clone_git_clone` | Removes duplicate `git clone` |
| `git_clone_missing` | Adds `git clone` to repository URLs |
| `git_command_typo` | Fixes general git command typos |
| `git_commit_add` | Suggests `git commit -a` or staging files |
| `git_commit_amend` | Offers `--amend` for commit fixes |
| `git_commit_reset` | Suggests `git reset HEAD~` |
| `git_diff_no_index` | Adds `--no-index` for untracked files |
| `git_diff_staged` | Adds `--staged` flag |
| `git_fix_stash` | Fixes stash subcommand typos |
| `git_flag_after_filename` | Fixes flag position after filename |
| `git_help_aliased` | Replaces alias with actual command |
| `git_hook_bypass` | Adds `--no-verify` when hooks fail |
| `git_lfs_mistype` | Fixes LFS command typos |
| `git_main_master` | Switches between `main` and `master` |
| `git_merge` | Adds remote to branch names |
| `git_merge_unrelated` | Adds `--allow-unrelated-histories` |
| `git_not_command` | Fixes misspelled git commands |
| `git_pull` | Sets upstream before pulling |
| `git_pull_clone` | Clones when repo doesn't exist |
| `git_pull_uncommitted_changes` | Stashes before pull |
| `git_push` | Sets upstream origin branch |
| `git_push_different_branch_names` | Fixes branch name mismatches |
| `git_push_force` | Adds `--force` flag |
| `git_push_pull` | Suggests pull before push |
| `git_push_without_commits` | Suggests committing first |
| `git_rebase` | Fixes rebase issues |
| `git_rebase_merge_dir` | Cleans up failed rebase |
| `git_rebase_no_changes` | Handles no-changes scenario |
| `git_remote_delete` | Fixes remote deletion syntax |
| `git_remote_seturl_add` | Fixes remote URL commands |
| `git_rm_local_modifications` | Adds `--force` for modified files |
| `git_rm_recursive` | Adds `-r` for directories |
| `git_rm_staged` | Handles staged file removal |
| `git_stash` | Fixes stash command issues |
| `git_stash_pop` | Fixes stash pop errors |
| `git_tag_force` | Adds `-f` for existing tags |
| `git_two_dashes` | Fixes single-dash options |

---

## Package Manager Rules

### APT (Debian/Ubuntu)

| Rule | Description |
|------|-------------|
| `apt_get` | Adds `sudo` when needed |
| `apt_get_search` | Switches to `apt search` |
| `apt_invalid_operation` | Fixes invalid apt operations |
| `apt_list_upgradable` | Corrects list upgradable syntax |
| `apt_upgrade` | Suggests `apt upgrade` |

### Homebrew (macOS/Linux)

| Rule | Description |
|------|-------------|
| `brew_cask_dependency` | Installs cask dependencies |
| `brew_install` | Fixes install command issues |
| `brew_link` | Handles linking errors |
| `brew_reinstall` | Suggests reinstall for existing packages |
| `brew_uninstall` | Fixes uninstall issues |
| `brew_unknown_command` | Corrects brew command typos |
| `brew_update` | Adds `brew update` first |
| `brew_update_formula` | Updates specific formula |

### Cargo (Rust)

| Rule | Description |
|------|-------------|
| `cargo_no_command` | Fixes missing cargo subcommand |
| `cargo_wrong_command` | Corrects cargo command typos |

### npm/Yarn (Node.js)

| Rule | Description |
|------|-------------|
| `npm_missing_script` | Suggests similar script names |
| `npm_run_script` | Adds `run` for custom scripts |
| `npm_wrong_command` | Fixes npm command typos |
| `yarn_alias` | Handles yarn aliases |
| `yarn_command_not_found` | Fixes yarn command typos |
| `yarn_command_replaced` | Updates deprecated commands |
| `yarn_help` | Corrects help syntax |

### pip (Python)

| Rule | Description |
|------|-------------|
| `pip_install` | Adds `pip install` |
| `pip_module_not_found` | Suggests installing packages |
| `pip_unknown_command` | Fixes pip command typos |

### Pacman (Arch Linux)

| Rule | Description |
|------|-------------|
| `pacman` | General pacman fixes |
| `pacman_invalid_option` | Fixes invalid options |
| `pacman_not_found` | Suggests package names |

### Other Package Managers

| Rule | Description |
|------|-------------|
| `choco_install` | Chocolatey install fixes (Windows) |
| `conda_mistype` | Conda command fixes |
| `dnf_no_such_command` | DNF command fixes (Fedora) |
| `gem_unknown_command` | Ruby gem fixes |
| `yum_invalid_operation` | YUM fixes (RHEL/CentOS) |

---

## System & File Rules

| Rule | Description |
|------|-------------|
| `cat_dir` | Replaces `cat dir` with `ls dir` |
| `cd_correction` | Fixes directory name typos |
| `cd_cs` | Corrects `cs` to `cd` |
| `cd_mkdir` | Creates directory before cd |
| `cd_parent` | Fixes `cd..` to `cd ..` |
| `chmod_x` | Adds execute permission |
| `cp_create_destination` | Creates destination directory |
| `cp_omitting_directory` | Adds `-r` for directories |
| `dirty_untar` | Fixes tar extraction issues |
| `dirty_unzip` | Fixes unzip issues |
| `fix_file` | Opens file with error in editor |
| `ln_no_hard_link` | Converts to symbolic link |
| `ln_s_order` | Fixes ln argument order |
| `ls_all` | Adds `-a` flag |
| `ls_lah` | Adds `-lah` flags |
| `mkdir_p` | Adds `-p` for nested directories |
| `rm_dir` | Adds `-r` for directories |
| `rm_root` | Prevents accidental root deletion |
| `sudo` | Adds sudo for permission errors |
| `touch` | Creates parent directories |

---

## Development Tool Rules

### Go

| Rule | Description |
|------|-------------|
| `go_run` | Fixes `go run` issues |
| `go_unknown_command` | Corrects go command typos |

### Java/JVM

| Rule | Description |
|------|-------------|
| `java` | Fixes java command issues |
| `javac` | Fixes javac compilation errors |
| `gradle_no_task` | Suggests similar task names |
| `gradle_wrapper` | Uses gradlew instead |
| `mvn_no_command` | Fixes maven command issues |
| `mvn_unknown_lifecycle_phase` | Corrects lifecycle phases |
| `lein_not_task` | Leiningen task fixes |

### PHP

| Rule | Description |
|------|-------------|
| `composer_not_command` | Fixes composer commands |
| `php_s` | Fixes PHP server command |

### Python

| Rule | Description |
|------|-------------|
| `python_command` | Fixes python command invocation |
| `python_execute` | Adds `python` prefix |
| `python_module_error` | Handles module import errors |

### Ruby/Rails

| Rule | Description |
|------|-------------|
| `rails_migrations_pending` | Runs pending migrations |

### JavaScript/Node

| Rule | Description |
|------|-------------|
| `grunt_task_not_found` | Fixes grunt task names |
| `gulp_not_task` | Fixes gulp task names |
| `react_native_command_unrecognized` | React Native fixes |

### Other

| Rule | Description |
|------|-------------|
| `cpp11` | Adds `-std=c++11` flag |
| `fab_command_not_found` | Fabric command fixes |
| `prove_recursively` | Adds `-r` to prove |

---

## Cloud & DevOps Rules

### AWS

| Rule | Description |
|------|-------------|
| `aws_cli` | Fixes AWS CLI command typos |

### Azure

| Rule | Description |
|------|-------------|
| `az_cli` | Fixes Azure CLI command typos |

### Docker

| Rule | Description |
|------|-------------|
| `docker_image_being_used_by_container` | Removes container first |
| `docker_login` | Prompts for docker login |
| `docker_not_command` | Fixes docker command typos |

### Terraform

| Rule | Description |
|------|-------------|
| `terraform_init` | Runs terraform init first |
| `terraform_no_command` | Fixes terraform command typos |

### Heroku

| Rule | Description |
|------|-------------|
| `heroku_multiple_apps` | Specifies app name |
| `heroku_not_command` | Fixes heroku command typos |

### Other

| Rule | Description |
|------|-------------|
| `port_already_in_use` | Kills process using port |
| `ssh_known_hosts` | Removes stale host keys |
| `tsuru_login` | Tsuru authentication |
| `tsuru_not_command` | Tsuru command fixes |
| `vagrant_up` | Starts vagrant VM first |

---

## Shell Utility Rules

| Rule | Description |
|------|-------------|
| `ag_literal` | Adds `-Q` for literal search |
| `grep_arguments_order` | Fixes grep argument order |
| `grep_recursive` | Adds `-r` for recursive search |
| `history` | Fixes history command |
| `man` | Fixes man page lookup |
| `man_no_space` | Adds space after man |
| `sed_unterminated_s` | Fixes sed substitution |
| `switch_lang` | Handles keyboard layout issues |
| `tmux` | Fixes tmux commands |

---

## Miscellaneous Rules

| Rule | Description |
|------|-------------|
| `adb_unknown_command` | Android ADB fixes |
| `django_south_ghost` | Django migration fixes |
| `django_south_merge` | Django migration merge |
| `dry` | Removes duplicate commands |
| `fix_alt_space` | Fixes Alt+Space characters |
| `has_exists_script` | Suggests existing scripts |
| `ifconfig_device_not_found` | Network interface fixes |
| `long_form_help` | Converts `-h` to `--help` |
| `mercurial` | Mercurial (hg) fixes |
| `missing_space_before_subcommand` | Adds missing spaces |
| `nixos_cmd_not_found` | NixOS command suggestions |
| `no_command` | Finds similar commands |
| `no_such_file` | Handles missing files |
| `omnienv_no_such_command` | rbenv/pyenv fixes |
| `open` | Fixes open command |
| `path_from_history` | Suggests paths from history |
| `quotation_marks` | Fixes quotation mark issues |
| `remove_shell_prompt_literal` | Removes copied prompt |
| `remove_trailing_cedilla` | Removes trailing cedilla |
| `scm_correction` | Version control fixes |
| `sl_ls` | Corrects `sl` to `ls` |
| `sudo_command_from_user_path` | Uses full path with sudo |
| `systemctl` | Systemd service fixes |
| `unsudo` | Removes unnecessary sudo |
| `whois` | Whois command fixes |
| `workon_doesnt_exists` | Python virtualenvwrapper fixes |
| `wrong_hyphen_before_subcommand` | Fixes hyphen issues |

---

## Enabling/Disabling Rules

### Enable All Rules (default)

```toml
# ~/.config/oops/config.toml
rules = ["ALL"]
```

### Enable Specific Rules

```toml
rules = ["git_push", "sudo", "no_command"]
```

### Exclude Specific Rules

```toml
rules = ["ALL"]
exclude_rules = ["rm_root", "systemctl"]
```

### Environment Variables

```bash
# Enable specific rules
export THEFUCK_RULES="git_push:sudo:no_command"

# Exclude rules
export THEFUCK_EXCLUDE_RULES="rm_root:systemctl"
```

---

## Rule Priority

Rules have priorities that determine their order when multiple rules match:

- **Lower number = Higher priority**
- Default priority: `1000`
- Critical fixes (like `sudo`): `100-500`
- Fallback suggestions: `1001+`

When multiple corrections are available, they're presented in priority order.

---

## Creating Custom Rules

See the [Creating Rules Guide](creating-rules.md) for information on adding new rules.
