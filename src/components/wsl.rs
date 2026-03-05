//! WSL (Windows Subsystem for Linux) component for managing Linux dotfiles.

use crate::components::traits::{Exportable, Importable, Upgradable};
use crate::error::{BootstrapError, Result};
use crate::output;
use crate::utils::{self, shell};
use std::fs;
use std::path::Path;

/// WSL dotfiles to export/import (relative to home directory).
const WSL_DOTFILES: &[&str] = &[
    ".bashrc",
    ".zshrc",
    ".profile",
    ".bash_profile",
    ".bash_aliases",
    ".gitconfig",
    ".vimrc",
    ".tmux.conf",
];

/// WSL directories to export/import.
const WSL_DOTDIRS: &[&str] = &[".ssh", ".config/starship", ".oh-my-zsh/custom"];

/// WSL component for managing Linux dotfiles.
pub struct Wsl {
    distro: String,
}

impl Wsl {
    pub fn new(distro: impl Into<String>) -> Self {
        Self {
            distro: distro.into(),
        }
    }

    /// Check if WSL and the specified distro are available.
    pub fn is_available(&self) -> bool {
        if !shell::command_exists("wsl") {
            return false;
        }

        // Check if distro exists (WSL outputs UTF-16 with null bytes)
        match shell::run_command("wsl", &["-l", "-q"]) {
            Ok(result) => {
                // Remove null bytes from UTF-16 output
                let distros: String = result.stdout.chars().filter(|c| *c != '\0').collect();
                distros.contains(&self.distro)
            }
            Err(_) => false,
        }
    }

    /// Run a command in WSL.
    fn run_wsl(&self, command: &str) -> Result<shell::CommandResult> {
        shell::run_wsl_command(&self.distro, command)
    }

    /// Get the WSL home directory.
    fn get_wsl_home(&self) -> Result<String> {
        let result = self.run_wsl("echo $HOME")?;
        Ok(result.stdout.trim().to_string())
    }

    /// Check if a file exists in WSL.
    fn wsl_file_exists(&self, path: &str) -> bool {
        self.run_wsl(&format!("test -f '{}' && echo yes || echo no", path))
            .map(|r| r.stdout.trim() == "yes")
            .unwrap_or(false)
    }

    /// Check if a directory exists in WSL.
    fn wsl_dir_exists(&self, path: &str) -> bool {
        self.run_wsl(&format!("test -d '{}' && echo yes || echo no", path))
            .map(|r| r.stdout.trim() == "yes")
            .unwrap_or(false)
    }

    /// Read a file from WSL.
    fn read_wsl_file(&self, path: &str) -> Result<String> {
        let result = self.run_wsl(&format!("cat '{}'", path))?;
        Ok(result.stdout)
    }
}

impl Default for Wsl {
    fn default() -> Self {
        Self::new("Ubuntu-24.04")
    }
}

impl Exportable for Wsl {
    fn name(&self) -> &'static str {
        "WSL Dotfiles"
    }

    fn export(&self, config_root: &Path) -> Result<()> {
        if !self.is_available() {
            output::warning(&format!(
                "WSL distro '{}' not available. Skipping WSL export.",
                self.distro
            ));
            return Ok(());
        }

        let wsl_export_path = config_root.join("dotfiles").join("wsl");
        utils::ensure_dir(&wsl_export_path)?;

        let wsl_home = self.get_wsl_home()?;
        output::info(&format!("WSL Home: {}", wsl_home));

        // Export individual dotfiles
        for dotfile in WSL_DOTFILES {
            let wsl_path = format!("{}/{}", wsl_home, dotfile);

            if self.wsl_file_exists(&wsl_path) {
                let content = self.read_wsl_file(&wsl_path)?;
                let dest_file = wsl_export_path.join(dotfile);

                // Ensure parent directory exists
                if let Some(parent) = dest_file.parent() {
                    utils::ensure_dir(parent)?;
                }

                fs::write(&dest_file, content).map_err(|e| BootstrapError::io(&dest_file, e))?;
                output::success(&format!("Exported: {}", dotfile));
            }
        }

        // Export directories using tar
        for dotdir in WSL_DOTDIRS {
            let wsl_path = format!("{}/{}", wsl_home, dotdir);

            if self.wsl_dir_exists(&wsl_path) {
                let dest_dir = wsl_export_path.join(dotdir);
                utils::ensure_dir(&dest_dir)?;

                let wsl_dest = utils::to_wsl_path(&dest_dir)?;

                self.run_wsl(&format!(
                    "cd '{}' && tar cf - . 2>/dev/null | (cd '{}' && tar xf -)",
                    wsl_path, wsl_dest
                ))?;

                output::success(&format!("Exported directory: {}/", dotdir));

                if *dotdir == ".ssh" {
                    output::warning(
                        "Note: .ssh contains sensitive keys. Review before committing to version control.",
                    );
                }
            }
        }

        // Export list of installed apt packages
        output::info("Exporting installed apt packages list...");
        let apt_result = self
            .run_wsl("apt list --installed 2>/dev/null | grep -v 'Listing...' | cut -d'/' -f1")?;

        let packages_file = wsl_export_path.join("installed-packages.txt");
        fs::write(&packages_file, apt_result.stdout)
            .map_err(|e| BootstrapError::io(&packages_file, e))?;

        output::success(&format!(
            "WSL dotfiles exported to: {}",
            wsl_export_path.display()
        ));
        Ok(())
    }
}

impl Importable for Wsl {
    fn name(&self) -> &'static str {
        "WSL Dotfiles"
    }

    fn import(&self, config_root: &Path) -> Result<()> {
        if !self.is_available() {
            output::warning(&format!(
                "WSL distro '{}' not available. Skipping WSL import.",
                self.distro
            ));
            return Ok(());
        }

        let wsl_import_path = config_root.join("dotfiles").join("wsl");

        if !wsl_import_path.exists() {
            output::warning(&format!(
                "No WSL dotfiles found at {}. Skipping.",
                wsl_import_path.display()
            ));
            return Ok(());
        }

        let wsl_home = self.get_wsl_home()?;
        output::info(&format!("WSL Home: {}", wsl_home));

        // Import individual dotfiles
        for dotfile in WSL_DOTFILES {
            let src_file = wsl_import_path.join(dotfile);

            if src_file.exists() {
                let wsl_dest = format!("{}/{}", wsl_home, dotfile);
                let wsl_src = utils::to_wsl_path(&src_file)?;

                // Backup existing file
                self.run_wsl(&format!(
                    "if [ -f '{}' ]; then cp '{}' '{}.backup' 2>/dev/null || true; fi",
                    wsl_dest, wsl_dest, wsl_dest
                ))?;

                // Copy file
                self.run_wsl(&format!("cp '{}' '{}'", wsl_src, wsl_dest))?;
                output::success(&format!("Imported: {}", dotfile));
            }
        }

        // Import directories
        for dotdir in WSL_DOTDIRS {
            let src_dir = wsl_import_path.join(dotdir);

            if src_dir.exists() {
                let wsl_dest = format!("{}/{}", wsl_home, dotdir);
                let wsl_src = utils::to_wsl_path(&src_dir)?;

                // Create destination directory
                self.run_wsl(&format!("mkdir -p '{}'", wsl_dest))?;

                // Copy directory contents using tar
                self.run_wsl(&format!(
                    "cd '{}' && tar cf - . 2>/dev/null | (cd '{}' && tar xf -)",
                    wsl_src, wsl_dest
                ))?;

                output::success(&format!("Imported directory: {}/", dotdir));

                // Fix permissions for .ssh
                if *dotdir == ".ssh" {
                    output::info("Setting correct permissions for .ssh...");
                    self.run_wsl(&format!(
                        "chmod 700 '{}' && chmod 600 '{}'/* 2>/dev/null || true && chmod 644 '{}'/*.pub 2>/dev/null || true",
                        wsl_dest, wsl_dest, wsl_dest
                    ))?;
                }
            }
        }

        output::success("WSL dotfiles imported successfully.");
        Ok(())
    }
}

impl Upgradable for Wsl {
    fn name(&self) -> &'static str {
        "WSL Packages"
    }

    fn upgrade(&self, _config_root: &Path) -> Result<()> {
        if !self.is_available() {
            output::warning(&format!(
                "WSL distro '{}' not available. Skipping.",
                self.distro
            ));
            return Ok(());
        }

        output::info("Updating apt packages in WSL...");

        // Run apt update and upgrade
        let result = shell::run_command(
            "wsl",
            &[
                "-d",
                &self.distro,
                "--",
                "bash",
                "-c",
                "sudo apt-get update -y && sudo apt-get upgrade -y && sudo apt-get autoremove -y",
            ],
        );

        match result {
            Ok(_) => output::success("WSL packages updated."),
            Err(e) => output::warning(&format!("WSL upgrade encountered issues: {}", e)),
        }

        Ok(())
    }
}
