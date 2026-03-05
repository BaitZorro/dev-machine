# Dev Machine Bootstrap (Windows 11)

This repo bootstraps a fresh Windows 11 developer machine with **WinGet** + a few configuration scripts.

It installs and configures a workflow optimized for:
- JetBrains Rider
- Git
- .NET 10
- WSL2 (Ubuntu 24.04)
- Docker Desktop
- Kubernetes tooling (kubectl/helm)
- Azure CLI
- ngrok, Notion, LINQPad, NVM for Windows
- VS Code + extensions
- PowerShell profile (oh-my-posh, useful aliases)

## Quick start

1) Open **PowerShell as Administrator**
2) Run:

```powershell
Set-ExecutionPolicy -Scope Process Bypass -Force
.\bootstrap.ps1 setup
```

### Using a custom config path

You can specify a different config root (where `dotfiles/` and `config/` folders are located):

```powershell
.\bootstrap.ps1 setup C:\path\to\my-config
```

### Export current settings

Export your current VS Code settings, PowerShell profile, Git config, and extensions list:

```powershell
.\bootstrap.ps1 export C:\path\to\backup-config
```

This will create/update:
- `dotfiles/vscode/` — VS Code settings and keybindings
- `dotfiles/powershell/` — PowerShell profile
- `dotfiles/git/` — Git configuration
- `dotfiles/wsl/` — WSL dotfiles (see below)
- `config/winget-packages.json` — Installed WinGet packages
- `config/vscode-extensions.json` — Installed VS Code extensions

### WSL Dotfiles

Both `setup` and `export` handle WSL (Ubuntu) dotfiles:

**Exported/Imported files:**
- `.bashrc`, `.zshrc`, `.profile`, `.bash_profile`, `.bash_aliases`
- `.gitconfig` (WSL-specific)
- `.vimrc`, `.tmux.conf`

**Exported/Imported directories:**
- `.ssh/` — SSH keys and config (⚠️ contains sensitive data)
- `.config/starship/` — Starship prompt config
- `.oh-my-zsh/custom/` — Custom oh-my-zsh themes and plugins

**Also exported (for reference):**
- `installed-packages.txt` — List of installed apt packages

> **Note:** The `.ssh` directory contains sensitive keys. Review contents before committing to version control. Consider using `.gitignore` to exclude private keys.

### Spotify (non-admin)

If Spotify fails (common when running elevated), run it separately in a **non-admin** terminal:

```powershell
.\bootstrap.ps1 setup -OnlySpotify
```

### Upgrade installed applications

Update all installed applications (WinGet packages, VS Code extensions, and WSL apt packages):

```powershell
.\bootstrap.ps1 upgrade
```

Use `-SkipWSL` to skip updating WSL packages:

```powershell
.\bootstrap.ps1 upgrade -SkipWSL
```

## Repo structure

- `bootstrap.ps1` — orchestrates the whole setup
- `config/winget-packages.json` — package list (edit to taste)
- `scripts/` — install/config scripts
- `dotfiles/` — version-controlled settings copied into place
  - `vscode/` — VS Code settings
  - `powershell/` — PowerShell profile
  - `git/` — Git configuration
  - `wsl/` — WSL dotfiles (.bashrc, .zshrc, .ssh, etc.)
- `wsl/` — scripts that run inside Ubuntu

## Customize

- Add/remove packages in `config/winget-packages.json`
- VS Code settings: `dotfiles/vscode/`
- PowerShell profile: `dotfiles/powershell/Microsoft.PowerShell_profile.ps1`
- Git defaults: `dotfiles/git/.gitconfig`
- WSL shell configs: `dotfiles/wsl/.bashrc`, `dotfiles/wsl/.zshrc`
- WSL SSH keys: `dotfiles/wsl/.ssh/`

## Notes

- Some Windows features (WSL2) may require a reboot.
- Docker Desktop may require logout/reboot after install.
- Rider settings: this repo includes **config scaffolding** and a safe way to apply them, but Rider’s exact config folders depend on version. See `scripts/configure-rider.ps1`.

Generated: 2026-03-05
