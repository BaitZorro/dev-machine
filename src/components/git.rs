//! Git configuration component.

use crate::components::traits::{Exportable, Importable};
use crate::error::{BootstrapError, Result};
use crate::output;
use crate::utils::{self, shell};
use std::fs;
use std::path::Path;

/// Git configuration component.
pub struct Git;

impl Git {
    pub fn new() -> Self {
        Self
    }
}

impl Default for Git {
    fn default() -> Self {
        Self::new()
    }
}

impl Exportable for Git {
    fn name(&self) -> &'static str {
        "Git Config"
    }

    fn export(&self, config_root: &Path) -> Result<()> {
        let dotfiles_path = config_root.join("dotfiles").join("git");
        utils::ensure_dir(&dotfiles_path)?;

        let git_config = utils::git_config_path()?;

        if git_config.exists() {
            let dest = dotfiles_path.join(".gitconfig");
            fs::copy(&git_config, &dest).map_err(|e| BootstrapError::io(&git_config, e))?;
            output::success("Exported .gitconfig");
        } else {
            output::warning("Global .gitconfig not found");
        }

        Ok(())
    }
}

impl Importable for Git {
    fn name(&self) -> &'static str {
        "Git Config"
    }

    fn import(&self, config_root: &Path) -> Result<()> {
        if !shell::command_exists("git") {
            output::warning("Git not found. Skipping.");
            return Ok(());
        }

        let dotfiles_gitconfig = config_root.join("dotfiles").join("git").join(".gitconfig");

        if dotfiles_gitconfig.exists() {
            output::info("Applying dotfiles gitconfig include...");

            // Use git config --global include.path to include the dotfiles config
            let path_str = dotfiles_gitconfig
                .to_str()
                .ok_or_else(|| BootstrapError::Path("Invalid gitconfig path".into()))?;

            shell::run_command("git", &["config", "--global", "include.path", path_str])?;
        }

        // Apply safe defaults
        let defaults = [
            ("init.defaultBranch", "main"),
            ("fetch.prune", "true"),
            ("pull.rebase", "false"),
            ("core.autocrlf", "false"),
            ("credential.helper", "manager"),
        ];

        for (key, value) in defaults {
            let _ = shell::run_command("git", &["config", "--global", key, value]);
        }

        output::success("Git defaults applied.");
        output::info("Set identity if not set:");
        output::info("  git config --global user.name \"Your Name\"");
        output::info("  git config --global user.email \"you@company.com\"");

        Ok(())
    }
}
