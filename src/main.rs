//! Dev Machine Bootstrapper
//!
//! A CLI tool for managing development machine configuration.
//!
//! # Commands
//!
//! - `setup`: Import configurations from the repository to the local machine
//! - `export`: Export current machine configuration to the repository
//! - `upgrade`: Upgrade installed packages and extensions
//!
//! # Examples
//!
//! ```bash
//! # Set up a new machine using configs from the current directory
//! dev-machine setup
//!
//! # Export current config to a specific path
//! dev-machine export --config-root C:\dev\dotfiles
//!
//! # Upgrade only winget packages
//! dev-machine upgrade --winget
//! ```

use clap::Parser;
use dev_machine::{commands, output, Cli, Command};
use std::path::PathBuf;
use std::process::ExitCode;

fn main() -> ExitCode {
    let cli = Cli::parse();

    // Determine config root (from args or current directory)
    let config_root = get_config_root(&cli.command);

    // Verify config root exists (except for export, which creates it)
    if !matches!(cli.command, Command::Export(_)) && !config_root.exists() {
        output::error(&format!(
            "Config root does not exist: {}",
            config_root.display()
        ));
        return ExitCode::FAILURE;
    }

    // Execute the appropriate command
    let result = match &cli.command {
        Command::Setup(args) => commands::setup(args, &config_root),
        Command::Export(args) => commands::export(args, &config_root),
        Command::Upgrade(args) => commands::upgrade(args, &config_root),
    };

    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            output::error(&e.to_string());
            ExitCode::FAILURE
        }
    }
}

/// Determine the config root path from command arguments or current directory.
fn get_config_root(command: &Command) -> PathBuf {
    let path = match command {
        Command::Setup(args) => args.config_root.clone(),
        Command::Export(args) => args.config_root.clone(),
        Command::Upgrade(args) => args.config_root.clone(),
    };

    path.unwrap_or_else(|| std::env::current_dir().expect("Failed to get current directory"))
}
