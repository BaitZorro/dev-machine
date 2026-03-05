<# 
  bootstrap.ps1
  Orchestrates: winget installs + config steps + optional WSL provisioning
#>
[CmdletBinding()]
param(
  [switch]$Minimal,
  [switch]$SkipDocker,
  [switch]$SkipWSL,
  [switch]$SkipSpotify,
  [switch]$OnlySpotify,
  [switch]$NoWSLProvision,
  [string]$DotfilesRepo = ""  # optional: if you keep dotfiles elsewhere, provide a git url
)

$ErrorActionPreference = "Stop"

function Write-Section($t){ Write-Host ""; Write-Host "=== $t ===" -ForegroundColor Cyan }
function Test-IsAdmin {
  $id=[Security.Principal.WindowsIdentity]::GetCurrent()
  $p=New-Object Security.Principal.WindowsPrincipal($id)
  $p.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
}

Write-Section "Preflight"
if (-not (Get-Command winget -ErrorAction SilentlyContinue)) {
  throw "WinGet not found. Install 'App Installer' from Microsoft Store, then re-run."
}

$repoRoot = Split-Path -Parent $MyInvocation.MyCommand.Path
$configPath = Join-Path $repoRoot "config\winget-packages.json"
if (-not (Test-Path $configPath)) { throw "Missing config: $configPath" }

$isAdmin = Test-IsAdmin
Write-Host ("Running as Admin: " + $isAdmin)

# Optional external dotfiles repo
if ($DotfilesRepo -and -not (Test-Path (Join-Path $repoRoot "dotfiles\.external"))) {
  Write-Section "Cloning external dotfiles"
  $dest = Join-Path $repoRoot "dotfiles\.external"
  git clone $DotfilesRepo $dest
  Write-Host "External dotfiles cloned to: $dest"
  Write-Host "You can now point scripts to it by setting -DotfilesRepo or editing scripts."
}

Write-Section "Update WinGet sources"
try { winget source update | Out-Host } catch { Write-Warning $_.Exception.Message }

# Install packages
Write-Section "Install packages"
& (Join-Path $repoRoot "scripts\winget-install.ps1") `
  -ConfigPath $configPath `
  -Minimal:$Minimal `
  -SkipDocker:$SkipDocker `
  -SkipWSL:$SkipWSL `
  -SkipSpotify:$SkipSpotify `
  -OnlySpotify:$OnlySpotify

if ($OnlySpotify) { return }

# Configure pieces
Write-Section "Configure Git"
& (Join-Path $repoRoot "scripts\configure-git.ps1") -RepoRoot $repoRoot

Write-Section "Configure PowerShell"
& (Join-Path $repoRoot "scripts\configure-powershell.ps1") -RepoRoot $repoRoot

Write-Section "Configure VS Code"
& (Join-Path $repoRoot "scripts\configure-vscode.ps1") -RepoRoot $repoRoot -ConfigPath $configPath

Write-Section "Configure Rider (optional settings copy)"
& (Join-Path $repoRoot "scripts\configure-rider.ps1") -RepoRoot $repoRoot

# Optional: provision WSL (installs zsh, tools, etc. inside Ubuntu)
if (-not $SkipWSL -and -not $NoWSLProvision) {
  Write-Section "Provision WSL (Ubuntu)"
  & (Join-Path $repoRoot "scripts\configure-wsl.ps1") -RepoRoot $repoRoot
} else {
  Write-Host "WSL provisioning skipped."
}

Write-Section "Done"
Write-Host "If WSL features were newly enabled, a reboot may be required."
Write-Host "If Docker prompts for WSL2 backend or requires reboot/logout, do that before using it."
