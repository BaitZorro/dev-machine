<# scripts/configure-powershell.ps1 #>
[CmdletBinding()]
param(
  [Parameter(Mandatory=$true)][string]$RepoRoot
)

$ErrorActionPreference = "Stop"
function Write-Section($t){ Write-Host ""; Write-Host "=== $t ===" -ForegroundColor Cyan }

$profileDir = Split-Path -Parent $PROFILE
New-Item -ItemType Directory -Force -Path $profileDir | Out-Null

$dotProfile = Join-Path $RepoRoot "dotfiles\powershell\Microsoft.PowerShell_profile.ps1"
if (Test-Path $dotProfile) {
  Write-Host "Copying PowerShell profile to: $PROFILE"
  Copy-Item $dotProfile -Destination $PROFILE -Force
} else {
  Write-Warning "PowerShell profile dotfile not found: $dotProfile"
}

Write-Host "PowerShell profile applied. Open a new terminal to load it."
