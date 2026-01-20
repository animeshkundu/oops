//! Package manager correction rules
//!
//! Contains rules for common package manager mistakes:
//! - apt (Debian/Ubuntu)
//! - brew (macOS Homebrew)
//! - npm (Node.js)
//! - pip (Python)
//! - cargo (Rust)
//! - pacman (Arch Linux)
//! - dnf (Fedora)
//! - yum (CentOS/RHEL)
//! - gem (Ruby)
//! - choco (Windows Chocolatey)
//! - conda (Anaconda/Miniconda)

pub mod apt;
pub mod brew;
pub mod cargo;
pub mod choco;
pub mod conda;
pub mod dnf;
pub mod gem;
pub mod npm;
pub mod pacman;
pub mod pip;
pub mod yum;

// Re-export all rules for easier access
pub use apt::{AptGet, AptGetSearch, AptInvalidOperation, AptListUpgradable};
pub use brew::{
    BrewCaskDependency, BrewInstall, BrewLink, BrewReinstall, BrewUninstall, BrewUnknownCommand,
    BrewUpdate, BrewUpdateFormula,
};
pub use cargo::{CargoNoCommand, CargoWrongCommand};
pub use choco::ChocoInstall;
pub use conda::CondaMistype;
pub use dnf::DnfNoSuchCommand;
pub use gem::GemUnknownCommand;
pub use npm::{NpmMissingScript, NpmWrongCommand};
pub use pacman::{Pacman, PacmanInvalidOption, PacmanNotFound};
pub use pip::{PipInstall, PipModuleNotFound, PipUnknownCommand};
pub use yum::YumInvalidOperation;

use crate::core::Rule;

/// Returns all package manager rules as boxed trait objects.
///
/// This function creates instances of all package manager rules
/// for registration with the rule system.
pub fn all_rules() -> Vec<Box<dyn Rule>> {
    vec![
        // APT rules (Debian/Ubuntu)
        Box::new(AptGet),
        Box::new(AptGetSearch),
        Box::new(AptInvalidOperation),
        Box::new(AptListUpgradable),
        // Homebrew rules (macOS/Linux)
        Box::new(BrewInstall),
        Box::new(BrewUpdate),
        Box::new(BrewUpdateFormula),
        Box::new(BrewCaskDependency),
        Box::new(BrewLink),
        Box::new(BrewReinstall),
        Box::new(BrewUninstall),
        Box::new(BrewUnknownCommand),
        // Cargo rules (Rust)
        Box::new(CargoNoCommand),
        Box::new(CargoWrongCommand),
        // Chocolatey rules (Windows)
        Box::new(ChocoInstall),
        // Conda rules
        Box::new(CondaMistype),
        // DNF rules (Fedora)
        Box::new(DnfNoSuchCommand),
        // Gem rules (Ruby)
        Box::new(GemUnknownCommand),
        // NPM rules (Node.js)
        Box::new(NpmMissingScript),
        Box::new(NpmWrongCommand),
        // Pacman rules (Arch Linux)
        Box::new(Pacman),
        Box::new(PacmanInvalidOption),
        Box::new(PacmanNotFound),
        // Pip rules (Python)
        Box::new(PipInstall),
        Box::new(PipModuleNotFound),
        Box::new(PipUnknownCommand),
        // YUM rules (CentOS/RHEL)
        Box::new(YumInvalidOperation),
    ]
}
