//! Component traits defining the contract for exportable, importable, and upgradable components.
//!
//! These traits follow the Interface Segregation Principle - components only
//! implement the traits that make sense for their functionality.

use crate::error::Result;
use std::path::Path;

/// Trait for components that can export their configuration.
pub trait Exportable {
    /// The name of the component (for logging/display).
    fn name(&self) -> &'static str;

    /// Export the component's configuration to the given config root.
    fn export(&self, config_root: &Path) -> Result<()>;
}

/// Trait for components that can import/apply configuration.
pub trait Importable {
    /// The name of the component (for logging/display).
    fn name(&self) -> &'static str;

    /// Import and apply configuration from the given config root.
    fn import(&self, config_root: &Path) -> Result<()>;
}

/// Trait for components that can be upgraded.
pub trait Upgradable {
    /// The name of the component (for logging/display).
    fn name(&self) -> &'static str;

    /// Upgrade the component (update packages, extensions, etc.).
    fn upgrade(&self, config_root: &Path) -> Result<()>;
}
