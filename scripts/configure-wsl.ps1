<# scripts/configure-wsl.ps1
  Runs provisioning script inside Ubuntu (WSL). 
  Assumes Ubuntu is installed and that 'wsl.exe' works.
#>
[CmdletBinding()]
param(
  [Parameter(Mandatory=$true)][string]$RepoRoot,
  [string]$DistroName = "Ubuntu-24.04"
)

$ErrorActionPreference = "Stop"

if (-not (Get-Command wsl -ErrorAction SilentlyContinue)) {
  Write-Warning "wsl.exe not found; skipping."
  return
}

$wslScriptWin = Join-Path $RepoRoot "wsl\provision-ubuntu.sh"
if (-not (Test-Path $wslScriptWin)) {
  Write-Warning "WSL provision script missing: $wslScriptWin"
  return
}

# Convert to WSL path: /mnt/c/...
$drive = $wslScriptWin.Substring(0,1).ToLower()
$rest  = $wslScriptWin.Substring(2).Replace("\","/")
$wslPath = "/mnt/$drive$rest"

Write-Host "Running WSL provisioning in distro '$DistroName'..."
Write-Host "Script: $wslPath"

# Ensure distro exists (this does not guarantee first-run user is configured)
try {
  wsl -l -q | Out-String | Out-Null
} catch {
  Write-Warning "WSL not ready. You may need to reboot and launch Ubuntu once."
  return
}

# Execute script
try {
  wsl -d $DistroName -- bash -lc "chmod +x '$wslPath' && '$wslPath'"
} catch {
  Write-Warning "WSL provisioning failed: $($_.Exception.Message)"
  Write-Host "You may need to open Ubuntu once to complete initial user creation, then re-run bootstrap."
}
