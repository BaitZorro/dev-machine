//! PowerShell profile component.

use crate::components::traits::{Exportable, Importable};
use crate::error::{BootstrapError, Result};
use crate::output;
use crate::utils;
use std::fs;
use std::path::Path;

/// PowerShell profile component.
pub struct PowerShell {
    /// Path to the repository root (for default profile fallback).
    repo_root: Option<std::path::PathBuf>,
}

impl PowerShell {
    pub fn new() -> Self {
        Self { repo_root: None }
    }

    pub fn with_repo_root(repo_root: impl Into<std::path::PathBuf>) -> Self {
        Self {
            repo_root: Some(repo_root.into()),
        }
    }
}

impl Default for PowerShell {
    fn default() -> Self {
        Self::new()
    }
}

impl Exportable for PowerShell {
    fn name(&self) -> &'static str {
        "PowerShell Profile"
    }

    fn export(&self, config_root: &Path) -> Result<()> {
        let dotfiles_path = config_root.join("dotfiles").join("powershell");
        utils::ensure_dir(&dotfiles_path)?;

        let profile_path = utils::powershell_profile_path()?;
        let dest_path = dotfiles_path.join("Microsoft.PowerShell_profile.ps1");

        if profile_path.exists() {
            fs::copy(&profile_path, &dest_path)
                .map_err(|e| BootstrapError::io(&profile_path, e))?;
            output::success("Exported PowerShell profile");
        } else {
            // Try to copy default profile from repo if available
            if let Some(ref repo_root) = self.repo_root {
                let default_profile = repo_root
                    .join("dotfiles")
                    .join("powershell")
                    .join("Microsoft.PowerShell_profile.ps1");

                if default_profile.exists() {
                    fs::copy(&default_profile, &dest_path)
                        .map_err(|e| BootstrapError::io(&default_profile, e))?;
                    output::info("No user profile found. Exported default profile from repo.");
                } else {
                    output::warning(&format!(
                        "PowerShell profile not found at {} and no default in repo.",
                        profile_path.display()
                    ));
                }
            } else {
                output::warning(&format!(
                    "PowerShell profile not found at {}",
                    profile_path.display()
                ));
            }
        }

        Ok(())
    }
}

impl Importable for PowerShell {
    fn name(&self) -> &'static str {
        "PowerShell Profile"
    }

    fn import(&self, config_root: &Path) -> Result<()> {
        let src_path = config_root
            .join("dotfiles")
            .join("powershell")
            .join("Microsoft.PowerShell_profile.ps1");

        if !src_path.exists() {
            output::warning(&format!(
                "PowerShell profile dotfile not found at {}",
                src_path.display()
            ));
            return Ok(());
        }

        let profile_path = utils::powershell_profile_path()?;

        // Ensure profile directory exists
        if let Some(parent) = profile_path.parent() {
            utils::ensure_dir(parent)?;
        }

        output::info(&format!(
            "Copying PowerShell profile to: {}",
            profile_path.display()
        ));

        fs::copy(&src_path, &profile_path).map_err(|e| BootstrapError::io(&src_path, e))?;

        output::success("PowerShell profile applied. Open a new terminal to load it.");
        Ok(())
    }
}
