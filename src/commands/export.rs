//! Export command implementation.
//!
//! Exports current machine configuration to the config root for version control.

use crate::cli::ExportArgs;
use crate::components::{Exportable, Git, PowerShell, VsCode, WinGet, Wsl};
use crate::error::{BootstrapError, Result};
use crate::output;
use std::fs;
use std::path::Path;

/// Check if a directory has any contents.
fn is_dir_non_empty(path: &Path) -> bool {
    if !path.exists() {
        return false;
    }
    match fs::read_dir(path) {
        Ok(mut entries) => entries.next().is_some(),
        Err(_) => false,
    }
}

/// Execute the export command.
pub fn execute(args: &ExportArgs, config_root: &Path) -> Result<()> {
    output::section("Exporting Configuration");
    output::kv("Config Root", &config_root.display().to_string());

    // Create config root if it doesn't exist
    if !config_root.exists() {
        output::info(&format!("Creating config directory: {}", config_root.display()));
        fs::create_dir_all(config_root).map_err(|e| BootstrapError::io(config_root, e))?;
    }

    // Check if target directories are non-empty (only if not forcing)
    if !args.force {
        let dotfiles_dir = config_root.join("dotfiles");
        let config_dir = config_root.join("config");

        if is_dir_non_empty(&dotfiles_dir) || is_dir_non_empty(&config_dir) {
            output::warning("Export directory is not empty. Existing files may be overwritten.");
            return Err(BootstrapError::Config(
                "Use --force (-f) to overwrite existing configuration files.".to_string(),
            ));
        }
    }

    // Determine which components to export
    let all = !args.vscode && !args.winget && !args.powershell && !args.git && !args.wsl;

    // WinGet packages
    if all || args.winget {
        output::section("WinGet Packages");
        let winget = WinGet::new();
        if let Err(e) = winget.export(config_root) {
            output::error(&format!("WinGet export failed: {}", e));
        }
    }

    // VS Code settings and extensions
    if all || args.vscode {
        output::section("VS Code Configuration");
        let vscode = VsCode::new();
        if let Err(e) = vscode.export(config_root) {
            output::error(&format!("VS Code export failed: {}", e));
        }
    }

    // PowerShell profile
    if all || args.powershell {
        output::section("PowerShell Profile");
        let powershell = PowerShell::new();
        if let Err(e) = powershell.export(config_root) {
            output::error(&format!("PowerShell export failed: {}", e));
        }
    }

    // Git configuration
    if all || args.git {
        output::section("Git Configuration");
        let git = Git::new();
        if let Err(e) = git.export(config_root) {
            output::error(&format!("Git export failed: {}", e));
        }
    }

    // WSL dotfiles
    if all || args.wsl {
        output::section("WSL Dotfiles");
        let wsl = Wsl::default();
        if let Err(e) = wsl.export(config_root) {
            output::error(&format!("WSL export failed: {}", e));
        }
    }

    output::section("Export Complete");
    output::success("Configuration exported successfully.");
    output::info("Review changes and commit to version control.");

    Ok(())
}
