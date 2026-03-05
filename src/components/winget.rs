//! WinGet component for managing Windows packages.

use crate::components::traits::{Exportable, Importable, Upgradable};
use crate::error::{BootstrapError, Result};
use crate::output;
use crate::utils::{self, shell};
use std::fs;
use std::path::Path;

/// WinGet component for Windows package management.
pub struct WinGet;

impl WinGet {
    pub fn new() -> Self {
        Self
    }

    /// Check if WinGet is available.
    pub fn is_available() -> bool {
        shell::command_exists("winget")
    }

    /// Ensure WinGet is available, returning an error if not.
    fn ensure_available(&self) -> Result<()> {
        if !Self::is_available() {
            return Err(BootstrapError::command_not_found(
                "winget",
                "Install 'App Installer' from Microsoft Store.",
            ));
        }
        Ok(())
    }
}

impl Default for WinGet {
    fn default() -> Self {
        Self::new()
    }
}

impl Exportable for WinGet {
    fn name(&self) -> &'static str {
        "WinGet Packages"
    }

    fn export(&self, config_root: &Path) -> Result<()> {
        self.ensure_available()?;

        let config_dir = config_root.join("config");
        utils::ensure_dir(&config_dir)?;

        output::info("Exporting installed WinGet packages...");

        // Use temp file to avoid capturing warning messages in JSON
        let temp_dir = std::env::temp_dir();
        let temp_file = temp_dir.join("winget-export-temp.json");

        // Export to temp file
        let result = shell::run_command(
            "winget",
            &[
                "export",
                "-o",
                temp_file.to_str().unwrap(),
                "--accept-source-agreements",
            ],
        );

        // Winget export may return non-zero even on partial success
        if let Err(e) = result {
            output::warning(&format!("WinGet export warnings: {}", e));
        }

        if temp_file.exists() {
            // Read and validate JSON
            let content =
                fs::read_to_string(&temp_file).map_err(|e| BootstrapError::io(&temp_file, e))?;

            // Validate it's valid JSON
            let parsed: serde_json::Value = serde_json::from_str(&content)?;

            // Count packages
            let package_count = parsed
                .get("Sources")
                .and_then(|s| s.as_array())
                .map(|sources| {
                    sources
                        .iter()
                        .filter_map(|s| s.get("Packages"))
                        .filter_map(|p| p.as_array())
                        .map(|a| a.len())
                        .sum::<usize>()
                })
                .unwrap_or(0);

            // Write to destination
            let dest_path = config_dir.join("winget-packages.json");
            fs::write(&dest_path, content).map_err(|e| BootstrapError::io(&dest_path, e))?;

            // Clean up temp file
            let _ = fs::remove_file(&temp_file);

            output::success(&format!(
                "Exported {} WinGet packages to: winget-packages.json",
                package_count
            ));
        } else {
            output::warning("WinGet export failed - no output file created.");
        }

        Ok(())
    }
}

impl Importable for WinGet {
    fn name(&self) -> &'static str {
        "WinGet Packages"
    }

    fn import(&self, config_root: &Path) -> Result<()> {
        self.ensure_available()?;

        let config_path = config_root.join("config").join("winget-packages.json");

        if !config_path.exists() {
            output::warning(&format!(
                "WinGet config not found at {}",
                config_path.display()
            ));
            return Ok(());
        }

        output::info("Updating WinGet sources...");
        let _ = shell::run_command_interactive("winget", &["source", "update"]);

        output::info("Installing WinGet packages...");
        shell::run_command_interactive(
            "winget",
            &[
                "import",
                "-i",
                config_path.to_str().unwrap(),
                "--accept-source-agreements",
                "--accept-package-agreements",
                "--ignore-unavailable",
            ],
        )?;

        output::success("WinGet packages installed.");
        Ok(())
    }
}

impl Upgradable for WinGet {
    fn name(&self) -> &'static str {
        "WinGet Packages"
    }

    fn upgrade(&self, _config_root: &Path) -> Result<()> {
        self.ensure_available()?;

        // Update sources
        output::info("Updating WinGet sources...");
        let _ = shell::run_command_interactive("winget", &["source", "update"]);

        // List available upgrades
        output::info("Checking for updates...");
        let _ = shell::run_command_interactive("winget", &["upgrade", "--include-unknown"]);

        // Perform upgrade
        output::info("Upgrading all packages...");
        shell::run_command_interactive(
            "winget",
            &[
                "upgrade",
                "--all",
                "--silent",
                "--accept-source-agreements",
                "--accept-package-agreements",
            ],
        )?;

        output::success("WinGet packages upgraded.");
        Ok(())
    }
}
