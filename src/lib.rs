//! oops - A blazingly fast command-line typo corrector
//!
//! This is a Rust rewrite inspired by the original Python thefuck project.
//! It provides faster startup time while maintaining full feature parity.
//!
//! This library crate exposes the core functionality for testing and extension.

pub mod cli;
pub mod config;
pub mod core;
pub mod output;
pub mod rules;
pub mod shells;
pub mod ui;
pub mod utils;
