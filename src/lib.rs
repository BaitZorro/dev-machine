//! Dev Machine Bootstrapper Library
//!
//! A cross-platform tool for managing development machine configuration.
//! Exports, imports, and upgrades various components like VS Code, WinGet,
//! PowerShell, Git, and WSL dotfiles.

pub mod cli;
pub mod commands;
pub mod components;
pub mod error;
pub mod output;
pub mod utils;

pub use cli::{Cli, Command};
pub use error::{BootstrapError, Result};
