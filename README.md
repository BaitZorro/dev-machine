# Dev Machine Bootstrapper

A CLI tool for managing development machine configuration across multiple machines. Export, import, and upgrade settings for VS Code, WinGet packages, PowerShell profiles, Git config, and WSL dotfiles.

## Prerequisites

- **Rust** (1.70 or later) - Required to build the project
- **Visual Studio Build Tools** - Required for Rust to compile native code on Windows
- **Windows 10/11** - This tool is designed for Windows development environments
- **Optional:** WSL with Ubuntu for Linux dotfile management

## Installing Prerequisites

### Install Visual Studio Build Tools

Rust requires the MSVC C++ build tools for linking on Windows:

```powershell
winget install Microsoft.VisualStudio.2022.BuildTools
```

During installation, select the **"Desktop development with C++"** workload, or run:

```powershell
# Install with required components via command line
winget install Microsoft.VisualStudio.2022.BuildTools --override "--wait --passive --add Microsoft.VisualStudio.Workload.VCTools --includeRecommended"
```

### Install Rust

Install Rust using rustup (the official installer):

```powershell
# Download and run the rustup installer
winget install Rustlang.Rustup

# Or download manually from https://rustup.rs
```

After installation, restart your terminal and verify:

```powershell
rustc --version
cargo --version
```

### Optional: Install WSL

If you want WSL dotfile management:

```powershell
wsl --install -d Ubuntu-24.04
```

## Building

### Debug Build

```powershell
cargo build
```

### Release Build (Optimized)

```powershell
cargo build --release
```

The executable will be at `target/release/dev-machine.exe`.

## Running

### Commands

```powershell
# Show help
dev-machine --help

# Set up a new machine (import all configs)
dev-machine setup

# Set up with a specific config directory
dev-machine setup --config-root C:\dev\dotfiles

# Set up only specific components
dev-machine setup --vscode --git

# Export current machine config
dev-machine export

# Export only specific components
dev-machine export --winget --vscode

# Upgrade all packages and extensions
dev-machine upgrade

# Upgrade only WinGet packages
dev-machine upgrade --winget
```

### Component Flags

Each command supports these flags to target specific components:

| Flag | Description |
|------|-------------|
| `--vscode` | VS Code settings and extensions |
| `--winget` | WinGet packages |
| `--powershell` | PowerShell profile |
| `--git` | Git configuration |
| `--wsl` | WSL dotfiles |

If no flags are specified, all components are processed.

## Project Structure

```
src/
├── main.rs              # CLI entry point
├── lib.rs               # Library root, public exports
├── cli.rs               # CLI argument definitions (clap)
├── error.rs             # Custom error types (thiserror)
├── output.rs            # Colored console output utilities
├── utils/
│   ├── mod.rs           # Module exports
│   ├── shell.rs         # Command execution utilities
│   └── paths.rs         # Windows path utilities
├── components/
│   ├── mod.rs           # Module exports
│   ├── traits.rs        # Exportable, Importable, Upgradable traits
│   ├── vscode.rs        # VS Code component
│   ├── winget.rs        # WinGet component
│   ├── powershell.rs    # PowerShell component
│   ├── git.rs           # Git component
│   └── wsl.rs           # WSL component
└── commands/
    ├── mod.rs           # Module exports
    ├── setup.rs         # Import command implementation
    ├── export.rs        # Export command implementation
    └── upgrade.rs       # Upgrade command implementation
```

### Architecture

The project follows SOLID principles with a trait-based design:

- **Single Responsibility**: Each component handles one tool (VS Code, WinGet, etc.)
- **Open/Closed**: New components can be added by implementing traits
- **Interface Segregation**: Three separate traits for different capabilities:
  - `Exportable` - Can export configuration
  - `Importable` - Can import/apply configuration
  - `Upgradable` - Can upgrade packages/extensions

### Config Directory Structure

```
config-root/
├── config/
│   ├── winget-packages.json    # WinGet package list
│   └── vscode-extensions.json  # VS Code extensions list
└── dotfiles/
    ├── vscode/
    │   ├── settings.json       # VS Code settings
    │   └── keybindings.json    # VS Code keybindings
    ├── powershell/
    │   └── Microsoft.PowerShell_profile.ps1
    ├── git/
    │   └── .gitconfig          # Git configuration
    └── wsl/
        ├── .bashrc
        ├── .zshrc
        ├── .ssh/               # SSH keys (be careful!)
        └── installed-packages.txt
```

## Testing

### Running Tests

```powershell
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run a specific test
cargo test test_name

# Run tests for a specific module
cargo test components::vscode
```

### Writing Tests

Tests are written inline in each module using `#[cfg(test)]`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_example() {
        assert!(true);
    }
}
```

### Test Coverage

To generate test coverage reports, install `cargo-tarpaulin`:

```powershell
cargo install cargo-tarpaulin
cargo tarpaulin --out Html
```

## Development

### Code Formatting

```powershell
cargo fmt
```

### Linting

```powershell
cargo clippy
```

### Documentation

```powershell
# Generate and open documentation
cargo doc --open
```

## License

MIT License

Copyright (c) 2026

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
