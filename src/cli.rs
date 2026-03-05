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
    #[arg(long, short = 'c')]
    pub config_root: Option<PathBuf>,

    /// Only set up VS Code settings and extensions
    #[arg(long)]
    pub vscode: bool,

    /// Only install WinGet packages
    #[arg(long)]
    pub winget: bool,

    /// Only set up PowerShell profile
    #[arg(long)]
    pub powershell: bool,

    /// Only set up Git configuration
    #[arg(long)]
    pub git: bool,

    /// Only set up WSL dotfiles
    #[arg(long)]
    pub wsl: bool,

    /// WSL distribution name
    #[arg(long, default_value = "Ubuntu-24.04")]
    pub wsl_distro: String,
}

/// Arguments for the export command.
#[derive(Parser, Debug)]
pub struct ExportArgs {
    /// Path to export configuration to
    #[arg(long, short = 'c')]
    pub config_root: Option<PathBuf>,

    /// Only export VS Code settings and extensions
    #[arg(long)]
    pub vscode: bool,

    /// Only export WinGet packages
    #[arg(long)]
    pub winget: bool,

    /// Only export PowerShell profile
    #[arg(long)]
    pub powershell: bool,

    /// Only export Git configuration
    #[arg(long)]
    pub git: bool,

    /// Only export WSL dotfiles
    #[arg(long)]
    pub wsl: bool,

    /// WSL distribution name
    #[arg(long, default_value = "Ubuntu-24.04")]
    pub wsl_distro: String,

    /// Force overwrite if export directory is not empty
    #[arg(long, short = 'f')]
    pub force: bool,
}

/// Arguments for the upgrade command.
#[derive(Parser, Debug)]
pub struct UpgradeArgs {
    /// Path to configuration root (for extension lists)
    #[arg(long, short = 'c')]
    pub config_root: Option<PathBuf>,

    /// Only upgrade VS Code extensions
    #[arg(long)]
    pub vscode: bool,

    /// Only upgrade WinGet packages
    #[arg(long)]
    pub winget: bool,

    /// Only upgrade WSL packages
    #[arg(long)]
    pub wsl: bool,

    /// WSL distribution name
    #[arg(long, default_value = "Ubuntu-24.04")]
    pub wsl_distro: String,
}
