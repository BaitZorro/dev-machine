//! VS Code component for managing settings and extensions.

use crate::components::traits::{Exportable, Importable, Upgradable};
use crate::error::{BootstrapError, Result};
use crate::output;
use crate::utils::{self, shell};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

/// VS Code extension list configuration.
#[derive(Debug, Serialize, Deserialize)]
pub struct ExtensionsConfig {
    pub extensions: Vec<String>,
}

/// VS Code component handling settings and extensions.
pub struct VsCode;

impl VsCode {
    pub fn new() -> Self {
        Self
    }

    /// Get the list of installed extensions by reading from filesystem.
    fn get_installed_extensions(&self) -> Result<Vec<String>> {
        let ext_dir = utils::vscode_extensions_dir()?;

        if !ext_dir.exists() {
            return Ok(Vec::new());
        }

        let mut extensions = Vec::new();

        for entry in fs::read_dir(&ext_dir).map_err(|e| BootstrapError::io(&ext_dir, e))? {
            let entry = entry.map_err(|e| BootstrapError::io(&ext_dir, e))?;
            let path = entry.path();

            if path.is_dir() {
                // Try to read package.json for accurate extension ID
                let package_json = path.join("package.json");
                if package_json.exists() {
                    if let Ok(content) = fs::read_to_string(&package_json) {
                        if let Ok(pkg) = serde_json::from_str::<serde_json::Value>(&content) {
                            if let (Some(publisher), Some(name)) =
                                (pkg.get("publisher"), pkg.get("name"))
                            {
                                if let (Some(p), Some(n)) = (publisher.as_str(), name.as_str()) {
                                    extensions.push(format!("{}.{}", p, n));
                                    continue;
                                }
                            }
                        }
                    }
                }

                // Fallback: parse folder name (publisher.name-version)
                if let Some(folder_name) = path.file_name().and_then(|n| n.to_str()) {
                    // Remove version suffix (e.g., "ms-python.python-2024.1.0" -> "ms-python.python")
                    if let Some(idx) = folder_name.rfind('-') {
                        let potential_version = &folder_name[idx + 1..];
                        // Check if what follows is a version number
                        if potential_version
                            .chars()
                            .next()
                            .map(|c| c.is_ascii_digit())
                            .unwrap_or(false)
                        {
                            extensions.push(folder_name[..idx].to_string());
                        }
                    }
                }
            }
        }

        extensions.sort();
        extensions.dedup();
        Ok(extensions)
    }

    /// Load extensions config from file.
    fn load_extensions_config(&self, config_root: &Path) -> Result<Option<ExtensionsConfig>> {
        let config_path = config_root.join("config").join("vscode-extensions.json");

        if !config_path.exists() {
            return Ok(None);
        }

        let content =
            fs::read_to_string(&config_path).map_err(|e| BootstrapError::io(&config_path, e))?;

        let config: ExtensionsConfig = serde_json::from_str(&content)?;
        Ok(Some(config))
    }
}

impl Default for VsCode {
    fn default() -> Self {
        Self::new()
    }
}

impl Exportable for VsCode {
    fn name(&self) -> &'static str {
        "VS Code"
    }

    fn export(&self, config_root: &Path) -> Result<()> {
        let dotfiles_path = config_root.join("dotfiles").join("vscode");
        let config_dir = config_root.join("config");

        utils::ensure_dir(&dotfiles_path)?;
        utils::ensure_dir(&config_dir)?;

        // Export settings and keybindings
        let vscode_user = utils::vscode_user_dir()?;

        if vscode_user.exists() {
            for file in &["settings.json", "keybindings.json"] {
                let src = vscode_user.join(file);
                if src.exists() {
                    let dest = dotfiles_path.join(file);
                    fs::copy(&src, &dest).map_err(|e| BootstrapError::io(&src, e))?;
                    output::success(&format!("Exported: {}", file));
                }
            }
        } else {
            output::warning(&format!(
                "VS Code user folder not found at {}",
                vscode_user.display()
            ));
        }

        // Export extensions list
        output::info("Exporting VS Code extensions list...");
        let extensions = self.get_installed_extensions()?;

        if !extensions.is_empty() {
            let config = ExtensionsConfig { extensions };
            let config_path = config_dir.join("vscode-extensions.json");
            let content = serde_json::to_string_pretty(&config)?;
            fs::write(&config_path, content).map_err(|e| BootstrapError::io(&config_path, e))?;
            output::success(&format!(
                "Exported {} VS Code extensions to: vscode-extensions.json",
                config.extensions.len()
            ));
        } else {
            output::info("No VS Code extensions found to export.");
        }

        Ok(())
    }
}

impl Importable for VsCode {
    fn name(&self) -> &'static str {
        "VS Code"
    }

    fn import(&self, config_root: &Path) -> Result<()> {
        let dotfiles_path = config_root.join("dotfiles").join("vscode");
        let vscode_user = utils::vscode_user_dir()?;

        utils::ensure_dir(&vscode_user)?;

        // Copy settings and keybindings
        if dotfiles_path.exists() {
            output::info(&format!(
                "Copying VS Code settings to {}",
                vscode_user.display()
            ));

            for file in &["settings.json", "keybindings.json"] {
                let src = dotfiles_path.join(file);
                if src.exists() {
                    let dest = vscode_user.join(file);
                    fs::copy(&src, &dest).map_err(|e| BootstrapError::io(&src, e))?;
                    output::success(&format!("Imported: {}", file));
                }
            }
        } else {
            output::warning(&format!(
                "VS Code dotfiles not found at {}",
                dotfiles_path.display()
            ));
        }

        // Install extensions
        if let Some(config) = self.load_extensions_config(config_root)? {
            if !shell::command_exists("code") {
                output::warning("VS Code CLI ('code') not found. Install VS Code and add to PATH.");
                return Ok(());
            }

            output::info("Installing VS Code extensions...");
            for ext in &config.extensions {
                match shell::run_command("code", &["--install-extension", ext, "--force"]) {
                    Ok(_) => output::success(&format!("Installed: {}", ext)),
                    Err(e) => output::warning(&format!("Failed to install {}: {}", ext, e)),
                }
            }
        } else {
            output::info("No vscode-extensions.json found. Skipping extensions.");
        }

        Ok(())
    }
}

impl Upgradable for VsCode {
    fn name(&self) -> &'static str {
        "VS Code Extensions"
    }

    fn upgrade(&self, config_root: &Path) -> Result<()> {
        if !shell::command_exists("code") {
            output::warning("VS Code CLI ('code') not found. Skipping.");
            return Ok(());
        }

        if let Some(config) = self.load_extensions_config(config_root)? {
            output::info("Updating VS Code extensions...");

            for ext in &config.extensions {
                // --force updates the extension if a newer version is available
                let _ = shell::run_command("code", &["--install-extension", ext, "--force"]);
            }

            output::success("VS Code extensions updated.");
        } else {
            output::info("No vscode-extensions.json found. Skipping.");
        }

        Ok(())
    }
}
