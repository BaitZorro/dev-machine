<# 
  scripts/winget-install.ps1
  Reads config/winget-packages.json and installs packages using winget
#>
[CmdletBinding()]
param(
  [Parameter(Mandatory=$true)][string]$ConfigPath,
  [switch]$Minimal,
  [switch]$SkipDocker,
  [switch]$SkipWSL,
  [switch]$SkipSpotify,
  [switch]$OnlySpotify
)

$ErrorActionPreference = "Stop"

function Write-Section($t){ Write-Host ""; Write-Host "=== $t ===" -ForegroundColor Cyan }
function Test-IsAdmin {
  $id=[Security.Principal.WindowsIdentity]::GetCurrent()
  $p=New-Object Security.Principal.WindowsPrincipal($id)
  $p.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
}

function Install-WingetPackage {
  param(
    [Parameter(Mandatory=$true)][string]$Id,
    [Parameter(Mandatory=$true)][string]$Name,
    [ValidateSet("user","machine")][string]$Scope="user"
  )

  Write-Host "Installing: $Name ($Id)..." -ForegroundColor Green
  $args = @(
    "install","--exact","--id",$Id,
    "--accept-source-agreements","--accept-package-agreements",
    "--silent"
  )

  # Not all packages support scope, but harmless for most
  $args += @("--scope",$Scope)

  try {
    winget @args | Out-Host
  } catch {
    Write-Warning "Failed: $Name ($Id): $($_.Exception.Message)"
  }
}

function Enable-WSL2Features {
  Write-Section "Enable WSL2 features (Admin; may require reboot)"
  $features = @("Microsoft-Windows-Subsystem-Linux","VirtualMachinePlatform")
  foreach ($f in $features) {
    $state = (dism.exe /online /Get-FeatureInfo /FeatureName:$f 2>$null | Select-String "State").ToString()
    if ($state -match "Enabled") {
      Write-Host "Feature already enabled: $f"
    } else {
      Write-Host "Enabling feature: $f"
      dism.exe /online /Enable-Feature /FeatureName:$f /All /NoRestart | Out-Host
    }
  }
  try { wsl --set-default-version 2 | Out-Host } catch { Write-Warning "Run after reboot: wsl --set-default-version 2" }
}

$config = Get-Content $ConfigPath -Raw | ConvertFrom-Json
$isAdmin = Test-IsAdmin

if ($OnlySpotify) {
  if ($SkipSpotify) { Write-Host "SkipSpotify specified; nothing to do."; return }
  if ($isAdmin) {
    Write-Warning "Spotify often fails from elevated terminals. Run this in a NON-admin PowerShell:"
    Write-Host "  .\bootstrap.ps1 -OnlySpotify"
    return
  }
  foreach ($p in $config.optional) {
    if ($p.id -eq "Spotify.Spotify") { Install-WingetPackage -Id $p.id -Name $p.name -Scope $p.scope }
  }
  return
}

# Core
foreach ($p in $config.core) {
  if ($SkipWSL -and $p.id -like "Canonical.Ubuntu.*") { continue }
  if ($SkipDocker -and $p.id -eq "Docker.DockerDesktop") { continue }

  # Enable WSL features before installing Ubuntu (recommended)
  if (-not $SkipWSL -and $p.id -like "Canonical.Ubuntu.*") {
    if ($isAdmin) { Enable-WSL2Features } else { Write-Warning "WSL feature enablement requires Admin. Re-run as Administrator." }
  }

  Install-WingetPackage -Id $p.id -Name $p.name -Scope $p.scope
}

# Dev extras
if (-not $Minimal) {
  foreach ($p in $config.dev) {
    if ($SkipDocker -and $p.id -eq "Docker.DockerDesktop") { continue }
    if ($SkipWSL -and $p.id -like "Canonical.Ubuntu.*") { continue }
    Install-WingetPackage -Id $p.id -Name $p.name -Scope $p.scope
  }

  # Optional (Spotify)
  if (-not $SkipSpotify) {
    if ($isAdmin) {
      Write-Warning "Skipping Spotify because you're running elevated. Install later in NON-admin PowerShell:"
      Write-Host "  .\bootstrap.ps1 -OnlySpotify"
    } else {
      foreach ($p in $config.optional) {
        if ($p.id -eq "Spotify.Spotify") { Install-WingetPackage -Id $p.id -Name $p.name -Scope $p.scope }
      }
    }
  }
}
