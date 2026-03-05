<# scripts/configure-rider.ps1
  JetBrains Rider settings can be applied in multiple ways:
  1) Recommended: Use Rider Settings Sync (JetBrains account) OR export/import settings.
  2) Advanced: Copy selected folders into %APPDATA%\JetBrains\<RiderVersion>\

  This script implements a SAFE approach:
  - Looks for dotfiles/rider/ as "overlay" content
  - Applies to all detected Rider config folders (Rider20xx.x)
  - Only copies known subfolders (codestyles, keymaps, options, templates, colors)
#>
[CmdletBinding()]
param(
  [Parameter(Mandatory=$true)][string]$RepoRoot
)

$ErrorActionPreference = "Stop"

$overlayRoot = Join-Path $RepoRoot "dotfiles\rider"
if (-not (Test-Path $overlayRoot)) {
  Write-Host "No Rider overlay found at $overlayRoot (this is OK)."
  return
}

$jetBrainsDir = Join-Path $env:APPDATA "JetBrains"
if (-not (Test-Path $jetBrainsDir)) {
  Write-Host "JetBrains config folder not found yet at $jetBrainsDir (Rider may not have been launched)."
  return
}

$allowed = @("codestyles","keymaps","options","templates","colors")
$riderDirs = Get-ChildItem $jetBrainsDir -Directory -ErrorAction SilentlyContinue | Where-Object { $_.Name -match "^Rider\d{4}\.\d" }

if (-not $riderDirs) {
  Write-Host "No Rider version folders detected under $jetBrainsDir yet. Launch Rider once, then re-run."
  return
}

foreach ($rd in $riderDirs) {
  Write-Host "Applying Rider overlay to: $($rd.FullName)"
  foreach ($sub in $allowed) {
    $src = Join-Path $overlayRoot $sub
    if (Test-Path $src) {
      $dst = Join-Path $rd.FullName $sub
      New-Item -ItemType Directory -Force -Path $dst | Out-Null
      Copy-Item "$src\*" -Destination $dst -Recurse -Force
    }
  }
}

Write-Host "Rider overlay applied. If settings don't appear, Rider may need restart."
Write-Host "Tip: Settings Sync is often the easiest long-term approach."
