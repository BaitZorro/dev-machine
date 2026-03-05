//! Upgrade command implementation.
//!
//! Upgrades installed packages and extensions.

use crate::cli::UpgradeArgs;
use crate::components::{Upgradable, VsCode, WinGet, Wsl};
use crate::error::Result;
use crate::output;
use std::path::Path;

/// Execute the upgrade command.
pub fn execute(args: &UpgradeArgs, config_root: &Path) -> Result<()> {
    output::section("Upgrading Packages");

    // Determine which components to upgrade
    let all = !args.vscode && !args.winget && !args.wsl;

    // WinGet packages
    if all || args.winget {
        output::section("WinGet Packages");
        let winget = WinGet::new();
        if let Err(e) = winget.upgrade(config_root) {
            output::error(&format!("WinGet upgrade failed: {}", e));
        }
    }

    // VS Code extensions
    if all || args.vscode {
        output::section("VS Code Extensions");
        let vscode = VsCode::new();
        if let Err(e) = vscode.upgrade(config_root) {
            output::error(&format!("VS Code upgrade failed: {}", e));
        }
    }

    // WSL packages
    if all || args.wsl {
        output::section("WSL Packages");
        let wsl = Wsl::default();
        if let Err(e) = wsl.upgrade(config_root) {
            output::error(&format!("WSL upgrade failed: {}", e));
        }
    }

    output::section("Upgrade Complete");
    output::success("All packages upgraded successfully.");

    Ok(())
}
