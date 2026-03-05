//! Custom error types for the dev-machine bootstrapper.
//!
//! Uses `thiserror` for ergonomic error definitions with proper
//! Display implementations and error chaining.

use std::path::PathBuf;
use thiserror::Error;

/// Main error type for the bootstrapper.
#[derive(Error, Debug)]
pub enum BootstrapError {
    #[error("Configuration error: {0}")]
    Config(String),

    #[error("IO error at path '{path}': {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("Command '{command}' failed: {message}")]
    CommandFailed { command: String, message: String },

    #[error("Command '{command}' not found. {hint}")]
    CommandNotFound { command: String, hint: String },

    #[error("JSON parsing error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("WSL error: {0}")]
    Wsl(String),

    #[error("Component '{component}' error: {message}")]
    Component { component: String, message: String },

    #[error("Path error: {0}")]
    Path(String),
}

impl BootstrapError {
    /// Create an IO error with path context.
    pub fn io(path: impl Into<PathBuf>, source: std::io::Error) -> Self {
        Self::Io {
            path: path.into(),
            source,
        }
    }

    /// Create a command failed error.
    pub fn command_failed(command: impl Into<String>, message: impl Into<String>) -> Self {
        Self::CommandFailed {
            command: command.into(),
            message: message.into(),
        }
    }

    /// Create a command not found error.
    pub fn command_not_found(command: impl Into<String>, hint: impl Into<String>) -> Self {
        Self::CommandNotFound {
            command: command.into(),
            hint: hint.into(),
        }
    }

    /// Create a component error.
    pub fn component(component: impl Into<String>, message: impl Into<String>) -> Self {
        Self::Component {
            component: component.into(),
            message: message.into(),
        }
    }
}

/// Result type alias using BootstrapError.
pub type Result<T> = std::result::Result<T, BootstrapError>;
