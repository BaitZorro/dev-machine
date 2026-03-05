<# scripts/configure-vscode.ps1 #>
[CmdletBinding()]
param(
  [Parameter(Mandatory=$true)][string]$RepoRoot,
  [Parameter(Mandatory=$true)][string]$ConfigPath
)

$ErrorActionPreference = "Stop"
function Write-Section($t){ Write-Host ""; Write-Host "=== $t ===" -ForegroundColor Cyan }

$vscodeUser = Join-Path $env:APPDATA "Code\User"
$dotVscode  = Join-Path $RepoRoot "dotfiles\vscode"

if (Test-Path $dotVscode) {
  New-Item -ItemType Directory -Force -Path $vscodeUser | Out-Null
  Write-Host "Copying VS Code settings to $vscodeUser"
  Copy-Item "$dotVscode\*" -Destination $vscodeUser -Recurse -Force
} else {
  Write-Warning "VS Code dotfiles not found at $dotVscode"
}

# Extensions
if (Get-Command code -ErrorAction SilentlyContinue) {
  $configDir = Split-Path -Parent $ConfigPath
  $extensionsFile = Join-Path $configDir "vscode-extensions.json"
  
  if (Test-Path $extensionsFile) {
    $extConfig = Get-Content $extensionsFile -Raw | ConvertFrom-Json
    $extensions = @($extConfig.extensions)
    
    if ($extensions.Count -gt 0) {
      Write-Host "Installing VS Code extensions..."
      foreach ($ext in $extensions) {
        try {
          code --install-extension $ext --force | Out-Host
        } catch {
          Write-Warning "Failed installing VS Code extension $ext: $($_.Exception.Message)"
        }
      }
    }
  } else {
    Write-Host "No vscode-extensions.json found. Skipping extensions."
  }
} else {
  Write-Warning "VS Code CLI ('code') not found in PATH. Open VS Code once and enable 'Shell Command: Install 'code' command in PATH', then re-run this script."
}
