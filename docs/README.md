# oops Documentation

Welcome to the oops documentation.

## Contents

### Architecture

Technical documentation about how oops works internally.

- [Overview](architecture/overview.md) - System architecture and design
- [Rule System](architecture/rule-system.md) - How correction rules work
- [Shell Integration](architecture/shell-integration.md) - How shell integration works

### Architecture Decision Records (ADRs)

Records of significant technical decisions.

- [ADR-001: Rust Rewrite](adr/001-rust-rewrite.md) - Why we chose Rust
- [ADR-002: TOML Configuration](adr/002-toml-config.md) - Why TOML over Python config
- [ADR-003: Compiled Rules](adr/003-compiled-rules.md) - Why rules are compiled in
- [ADR-004: Shell Detection](adr/004-shell-detection.md) - How we detect the current shell

### Guides

User and developer guides.

- [Installation](guides/installation.md) - Detailed installation instructions
- [Configuration](guides/configuration.md) - All configuration options
- [Creating Rules](guides/creating-rules.md) - How to add new correction rules
- [Migration from thefuck](guides/migration-from-thefuck.md) - Migrating from the Python version
- [Quick Release Guide](QUICK_RELEASE_GUIDE.md) - How releases work (TL;DR)
- [Automated Releases](AUTOMATED_RELEASES.md) - Complete release workflow documentation

### History

- [Changelog](history/CHANGELOG.md) - Version history
