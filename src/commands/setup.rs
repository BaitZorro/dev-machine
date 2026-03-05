//! Setup command implementation.
//!
//! Imports configurations from the config root to the local machine.

use crate::cli::SetupArgs;
use crate::components::{Git, Importable, PowerShell, VsCode, WinGet, Wsl};
use crate::error::Result;
use crate::output;
use std::path::Path;

/// Execute the setup command.
pub fn execute(args: &SetupArgs, config_root: &Path) -> Result<()> {
    output::section("Development Machine Setup");
    output::kv("Config Root", &config_root.display().to_string());

    // Determine which components to set up
    let all = !args.vscode && !args.winget && !args.powershell && !args.git && !args.wsl;

    // WinGet packages (run first as other tools may depend on them)
    if all || args.winget {
        output::section("WinGet Packages");
        let winget = WinGet::new();
        if let Err(e) = winget.import(config_root) {
            output::error(&format!("WinGet setup failed: {}", e));
        }
    }

    // VS Code settings and extensions
    if all || args.vscode {
        output::section("VS Code Configuration");
        let vscode = VsCode::new();
        if let Err(e) = vscode.import(config_root) {
            output::error(&format!("VS Code setup failed: {}", e));
        }
    }

    // PowerShell profile
    if all || args.powershell {
        output::section("PowerShell Profile");
        let powershell = PowerShell::new();
        if let Err(e) = powershell.import(config_root) {
            output::error(&format!("PowerShell setup failed: {}", e));
        }
    }

    // Git configuration
    if all || args.git {
        output::section("Git Configuration");
        let git = Git::new();
        if let Err(e) = git.import(config_root) {
            output::error(&format!("Git setup failed: {}", e));
        }
    }

    // WSL dotfiles
    if all || args.wsl {
        output::section("WSL Dotfiles");
        let wsl = Wsl::default();
        if let Err(e) = wsl.import(config_root) {
            output::error(&format!("WSL setup failed: {}", e));
        }
    }

    output::section("Setup Complete");
    output::success("Development environment configured successfully.");

    Ok(())
}
