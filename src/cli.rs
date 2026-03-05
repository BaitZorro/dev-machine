//! Command-line interface definitions using clap.
//!
//! Defines the CLI structure with subcommands for setup, export, and upgrade.

use clap::{Parser, Subcommand};
use std::path::PathBuf;

/// Dev Machine Bootstrapper - Configure and maintain your development environment.
#[derive(Parser, Debug)]
#[command(name = "dev-machine")]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,

    /// Enable verbose output
    #[arg(short, long, global = true)]
    pub verbose: bool,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Set up the development environment using configuration from a path
    Setup(SetupArgs),

    /// Export current system settings to a configuration path
    Export(ExportArgs),

    /// Upgrade all installed applications
    Upgrade(UpgradeArgs),
}

/// Arguments for the setup command.
#[derive(Parser, Debug)]
pub struct SetupArgs {
    /// Path to the configuration root (contains dotfiles/ and config/ folders)
    #[arg(default_value = ".")]
    pub config_root: PathBuf,

    /// Minimal installation (skip optional packages)
    #[arg(long)]
    pub minimal: bool,

    /// Skip Docker installation
    #[arg(long)]
    pub skip_docker: bool,

    /// Skip WSL installation and provisioning
    #[arg(long)]
    pub skip_wsl: bool,

    /// Skip WSL provisioning (WSL will be installed but not configured)
    #[arg(long)]
    pub no_wsl_provision: bool,

    /// URL to external dotfiles repository to clone
    #[arg(long)]
    pub dotfiles_repo: Option<String>,
}

/// Arguments for the export command.
#[derive(Parser, Debug)]
pub struct ExportArgs {
    /// Path to export configuration to
    #[arg(default_value = ".")]
    pub config_root: PathBuf,

    /// Skip WSL dotfiles export
    #[arg(long)]
    pub skip_wsl: bool,

    /// WSL distribution name
    #[arg(long, default_value = "Ubuntu-24.04")]
    pub wsl_distro: String,
}

/// Arguments for the upgrade command.
#[derive(Parser, Debug)]
pub struct UpgradeArgs {
    /// Path to configuration root (for extension lists)
    #[arg(default_value = ".")]
    pub config_root: PathBuf,

    /// Skip WSL package upgrades
    #[arg(long)]
    pub skip_wsl: bool,

    /// WSL distribution name
    #[arg(long, default_value = "Ubuntu-24.04")]
    pub wsl_distro: String,
}
