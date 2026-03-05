//! Output formatting utilities for consistent console output.
//!
//! Provides colored, formatted output for different message types.

use colored::Colorize;

/// Print a section header.
pub fn section(title: &str) {
    println!();
    println!("{}", format!("=== {} ===", title).cyan().bold());
}

/// Print an informational message.
pub fn info(message: &str) {
    println!("{}", message);
}

/// Print a success message.
pub fn success(message: &str) {
    println!("{} {}", "✓".green(), message);
}

/// Print a warning message.
pub fn warning(message: &str) {
    println!("{} {}", "⚠".yellow(), message.yellow());
}

/// Print an error message.
pub fn error(message: &str) {
    eprintln!("{} {}", "✗".red(), message.red());
}

/// Print a key-value pair.
pub fn kv(key: &str, value: &str) {
    println!("{}: {}", key.bold(), value);
}
