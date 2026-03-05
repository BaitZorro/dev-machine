<# scripts/configure-git.ps1 #>
[CmdletBinding()]
param(
  [Parameter(Mandatory=$true)][string]$RepoRoot
)

$ErrorActionPreference = "Stop"
function Write-Section($t){ Write-Host ""; Write-Host "=== $t ===" -ForegroundColor Cyan }

if (-not (Get-Command git -ErrorAction SilentlyContinue)) {
  Write-Warning "Git not found. Skipping."
  return
}

# Apply dotfiles gitconfig (optional include)
$dotGit = Join-Path $RepoRoot "dotfiles\git\.gitconfig"
if (Test-Path $dotGit) {
  Write-Host "Applying dotfiles gitconfig include..."
  git config --global include.path "$dotGit"
}

# Safe defaults
git config --global init.defaultBranch main
git config --global fetch.prune true
git config --global pull.rebase false
git config --global core.autocrlf false
git config --global credential.helper manager

Write-Host "Git defaults applied."
Write-Host "Set identity if not set:"
Write-Host "  git config --global user.name  \"Your Name\""
Write-Host "  git config --global user.email \"you@company.com\""
