//! Component modules for managing different aspects of the development environment.
//!
//! Each component implements one or more of the traits defined in `traits.rs`:
//! - `Exportable`: Can export its configuration
//! - `Importable`: Can import/apply configuration
//! - `Upgradable`: Can upgrade its packages/extensions

pub mod git;
pub mod powershell;
pub mod traits;
pub mod vscode;
pub mod winget;
pub mod wsl;

pub use git::Git;
pub use powershell::PowerShell;
pub use traits::{Exportable, Importable, Upgradable};
pub use vscode::VsCode;
pub use winget::WinGet;
pub use wsl::Wsl;
