//! Shell command execution utilities.
//!
//! Provides safe wrappers around shell command execution with proper
//! error handling and output capture.

use crate::error::{BootstrapError, Result};
use std::process::{Command, Output, Stdio};

/// Result of a shell command execution.
#[derive(Debug)]
pub struct CommandResult {
    pub stdout: String,
    pub stderr: String,
    pub success: bool,
    pub exit_code: Option<i32>,
}

impl CommandResult {
    /// Check if the command succeeded, returning an error if not.
    pub fn ensure_success(&self, command_name: &str) -> Result<&Self> {
        if self.success {
            Ok(self)
        } else {
            Err(BootstrapError::command_failed(
                command_name,
                if self.stderr.is_empty() {
                    format!("Exit code: {:?}", self.exit_code)
                } else {
                    self.stderr.clone()
                },
            ))
        }
    }
}

/// Check if a command exists in PATH.
pub fn command_exists(name: &str) -> bool {
    Command::new("where")
        .arg(name)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

/// Run a shell command and capture output.
pub fn run_command(program: &str, args: &[&str]) -> Result<CommandResult> {
    let output = Command::new(program)
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                BootstrapError::command_not_found(program, "Ensure it is installed and in PATH")
            } else {
                BootstrapError::command_failed(program, e.to_string())
            }
        })?;

    Ok(parse_output(output))
}

/// Run a shell command, inheriting stdout/stderr for interactive output.
pub fn run_command_interactive(program: &str, args: &[&str]) -> Result<CommandResult> {
    let status = Command::new(program)
        .args(args)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                BootstrapError::command_not_found(program, "Ensure it is installed and in PATH")
            } else {
                BootstrapError::command_failed(program, e.to_string())
            }
        })?;

    Ok(CommandResult {
        stdout: String::new(),
        stderr: String::new(),
        success: status.success(),
        exit_code: status.code(),
    })
}

/// Run a PowerShell command.
pub fn run_powershell(script: &str) -> Result<CommandResult> {
    run_command("powershell", &["-NoProfile", "-Command", script])
}

/// Run a command in WSL.
pub fn run_wsl_command(distro: &str, command: &str) -> Result<CommandResult> {
    run_command("wsl", &["-d", distro, "--", "bash", "-c", command])
}

fn parse_output(output: Output) -> CommandResult {
    CommandResult {
        stdout: String::from_utf8_lossy(&output.stdout).to_string(),
        stderr: String::from_utf8_lossy(&output.stderr).to_string(),
        success: output.status.success(),
        exit_code: output.status.code(),
    }
}
