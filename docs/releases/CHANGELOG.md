# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0] - 2026-01-19

### Added

- Initial release of oops, inspired by thefuck
- 175+ correction rules including:
  - 49 Git rules (push, checkout, add, branch, commit, etc.)
  - 27 package manager rules (apt, brew, cargo, npm, pip, pacman, etc.)
  - 18 system command rules (sudo, mkdir, chmod, cp, rm, etc.)
  - 16 development tool rules (go, java, maven, gradle, terraform, etc.)
  - 15 framework rules (python, rails, react-native, yarn, etc.)
  - 15 shell utility rules (grep, sed, history, etc.)
  - 10 cloud/network rules (aws, azure, heroku, ssh, etc.)
  - 5 docker rules
  - 4 cd navigation rules
  - And many more
- Shell integration for:
  - Bash
  - Zsh
  - Fish
  - PowerShell
  - Tcsh
- TOML-based configuration system
- Environment variable configuration support
- Cross-platform support (Linux, macOS, Windows)
- Sub-50ms startup time
- Single binary distribution

### Changed

- Configuration format changed from Python (settings.py) to TOML (config.toml)

### Acknowledgments

- Based on the original [thefuck](https://github.com/nvbn/thefuck) project by Vladimir Iakovlev
- Rule patterns and shell integration approaches adapted from the Python implementation

---

## Version History

This project was inspired by the Python thefuck project. The goal was to achieve:

1. **Performance**: 10x faster startup time through native compilation
2. **Distribution**: Single binary with no runtime dependencies
3. **Compatibility**: Full feature parity with the Python version
4. **Professionalism**: A name suitable for all work environments

oops achieves all these goals. Configuration environment variables are backward compatible for easy migration from thefuck.
