//! Utility functions for oops.
//!
//! This module provides common utilities used throughout the application:
//! - [`cache`] - Memoization utilities using the `cached` crate
//! - [`fuzzy`] - Fuzzy string matching similar to Python's difflib
//! - [`executables`] - PATH scanning and executable lookup

pub mod cache;
pub mod executables;
pub mod fuzzy;

pub use cache::which as cached_which;
pub use executables::{get_all_executables, replace_argument, which};
pub use fuzzy::{get_close_matches, get_closest};
