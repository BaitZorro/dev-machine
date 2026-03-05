//! Path utilities for common Windows paths.
//!
//! Provides functions to resolve standard Windows paths for various
//! applications and configurations.

use crate::error::{BootstrapError, Result};
use std::path::PathBuf;

/// Get the user's home directory.
pub fn home_dir() -> Result<PathBuf> {
    dirs::home_dir().ok_or_else(|| BootstrapError::Path("Could not determine home directory".into()))
}

/// Get the APPDATA directory (%APPDATA%).
pub fn appdata_dir() -> Result<PathBuf> {
    dirs::config_dir()
        .ok_or_else(|| BootstrapError::Path("Could not determine APPDATA directory".into()))
}

/// Get the local APPDATA directory (%LOCALAPPDATA%).
pub fn local_appdata_dir() -> Result<PathBuf> {
    dirs::cache_dir()
        .or_else(|| std::env::var("LOCALAPPDATA").ok().map(PathBuf::from))
        .ok_or_else(|| BootstrapError::Path("Could not determine LOCALAPPDATA directory".into()))
}

/// Get the VS Code user settings directory.
pub fn vscode_user_dir() -> Result<PathBuf> {
    Ok(appdata_dir()?.join("Code").join("User"))
}

/// Get the VS Code extensions directory.
pub fn vscode_extensions_dir() -> Result<PathBuf> {
    Ok(home_dir()?.join(".vscode").join("extensions"))
}

/// Get the PowerShell profile path.
pub fn powershell_profile_path() -> Result<PathBuf> {
    let docs = dirs::document_dir()
        .ok_or_else(|| BootstrapError::Path("Could not determine Documents directory".into()))?;
    Ok(docs.join("PowerShell").join("Microsoft.PowerShell_profile.ps1"))
}

/// Get the global Git config path.
pub fn git_config_path() -> Result<PathBuf> {
    Ok(home_dir()?.join(".gitconfig"))
}

/// Convert a Windows path to WSL path format.
pub fn to_wsl_path(path: &std::path::Path) -> Result<String> {
    let path_str = path
        .to_str()
        .ok_or_else(|| BootstrapError::Path("Invalid path characters".into()))?;

    // Handle paths like C:\Users\...
    if path_str.len() >= 2 && path_str.chars().nth(1) == Some(':') {
        let drive = path_str
            .chars()
            .next()
            .unwrap()
            .to_lowercase()
            .next()
            .unwrap();
        let rest = &path_str[2..].replace('\\', "/");
        Ok(format!("/mnt/{}{}", drive, rest))
    } else {
        Err(BootstrapError::Path(format!(
            "Cannot convert path to WSL format: {}",
            path_str
        )))
    }
}

/// Ensure a directory exists, creating it if necessary.
pub fn ensure_dir(path: &std::path::Path) -> Result<()> {
    if !path.exists() {
        std::fs::create_dir_all(path).map_err(|e| BootstrapError::io(path, e))?;
    }
    Ok(())
}

/// Resolve a path to absolute.
pub fn resolve_path(path: &std::path::Path) -> Result<PathBuf> {
    std::fs::canonicalize(path).or_else(|_| {
        // If path doesn't exist yet, resolve relative to current dir
        let current = std::env::current_dir()
            .map_err(|e| BootstrapError::io(path, e))?;
        Ok(current.join(path))
    })
}
