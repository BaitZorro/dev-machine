<# 
  bootstrap.ps1
  Orchestrates: winget installs + config steps + optional WSL provisioning
  
  Usage:
    bootstrap.ps1 setup <path-to-configroot>   # Install and configure using config from path
    bootstrap.ps1 export <path-to-configroot>  # Export current settings to config path
    bootstrap.ps1 upgrade                       # Update all installed applications
#>
[CmdletBinding()]
param(
  [Parameter(Position=0)]
  [ValidateSet("setup", "export", "upgrade")]
  [string]$Command = "setup",
  
  [Parameter(Position=1)]
  [string]$ConfigRoot = "",
  
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

# ============================================================================
# WSL HELPER FUNCTIONS
# ============================================================================
function Test-WSLAvailable {
  param([string]$DistroName = "Ubuntu-24.04")
  
  if (-not (Get-Command wsl -ErrorAction SilentlyContinue)) {
    return $false
  }
  
  try {
    $distros = wsl -l -q 2>$null
    return ($distros -match $DistroName)
  } catch {
    return $false
  }
}

function ConvertTo-WSLPath {
  param([string]$WindowsPath)
  $drive = $WindowsPath.Substring(0,1).ToLower()
  $rest = $WindowsPath.Substring(2).Replace("\","/")
  return "/mnt/$drive$rest"
}

function Invoke-WSLCommand {
  param(
    [string]$Command,
    [string]$DistroName = "Ubuntu-24.04"
  )
  wsl -d $DistroName -- bash -c $Command
}

# WSL dotfiles to export/import (relative to home directory)
$script:WSLDotfiles = @(
  ".bashrc",
  ".zshrc",
  ".profile",
  ".bash_profile",
  ".bash_aliases",
  ".gitconfig",
  ".vimrc",
  ".tmux.conf"
)

# WSL directories to export/import
$script:WSLDotDirs = @(
  ".ssh",
  ".config/starship",
  ".oh-my-zsh/custom"
)

$repoRoot = Split-Path -Parent $MyInvocation.MyCommand.Path

# Resolve ConfigRoot - default to repoRoot if not specified
if ([string]::IsNullOrWhiteSpace($ConfigRoot)) {
  $ConfigRoot = $repoRoot
} else {
  # Resolve to absolute path
  $ConfigRoot = [System.IO.Path]::GetFullPath($ConfigRoot)
}

$configPath = Join-Path $ConfigRoot "config\winget-packages.json"
$dotfilesPath = Join-Path $ConfigRoot "dotfiles"

# ============================================================================
# EXPORT COMMAND
# ============================================================================
function Invoke-Export {
  param([string]$ConfigRoot)
  
  Write-Section "Export Configuration"
  Write-Host "Exporting current settings to: $ConfigRoot"
  
  # Create directory structure
  $dotfilesPath = Join-Path $ConfigRoot "dotfiles"
  $configDir = Join-Path $ConfigRoot "config"
  
  New-Item -ItemType Directory -Force -Path $dotfilesPath | Out-Null
  New-Item -ItemType Directory -Force -Path $configDir | Out-Null
  
  # Export VS Code settings
  Write-Section "Export VS Code"
  $vscodeUser = Join-Path $env:APPDATA "Code\User"
  $vscodeExport = Join-Path $dotfilesPath "vscode"
  
  if (Test-Path $vscodeUser) {
    New-Item -ItemType Directory -Force -Path $vscodeExport | Out-Null
    
    $settingsFile = Join-Path $vscodeUser "settings.json"
    $keybindingsFile = Join-Path $vscodeUser "keybindings.json"
    
    if (Test-Path $settingsFile) {
      Copy-Item $settingsFile -Destination $vscodeExport -Force
      Write-Host "Exported: settings.json"
    }
    if (Test-Path $keybindingsFile) {
      Copy-Item $keybindingsFile -Destination $vscodeExport -Force
      Write-Host "Exported: keybindings.json"
    }
  } else {
    Write-Warning "VS Code user folder not found at $vscodeUser"
  }
  
  # Export VS Code extensions list (read from filesystem to avoid launching VS Code)
  Write-Host "Exporting VS Code extensions list..."
  $vscodeExtDir = Join-Path $env:USERPROFILE ".vscode\extensions"
  
  if (Test-Path $vscodeExtDir) {
    $extensions = @()
    $extFolders = Get-ChildItem -Path $vscodeExtDir -Directory -ErrorAction SilentlyContinue
    
    foreach ($folder in $extFolders) {
      # Read package.json to get accurate extension ID
      $packageJson = Join-Path $folder.FullName "package.json"
      if (Test-Path $packageJson) {
        try {
          $pkg = Get-Content $packageJson -Raw | ConvertFrom-Json
          if ($pkg.publisher -and $pkg.name) {
            $extensions += "$($pkg.publisher).$($pkg.name)"
          }
        } catch {
          # Fallback: parse folder name (publisher.name-version)
          if ($folder.Name -match '^(.+)-\d+\.\d+\.\d+') {
            $extensions += $Matches[1]
          }
        }
      }
    }
    
    $extensions = $extensions | Sort-Object -Unique
    
    if ($extensions.Count -gt 0) {
      $extensionsFile = Join-Path $configDir "vscode-extensions.json"
      @{ extensions = $extensions } | ConvertTo-Json -Depth 10 | Set-Content $extensionsFile -Encoding UTF8
      Write-Host "Exported $($extensions.Count) VS Code extensions to: vscode-extensions.json"
    } else {
      Write-Host "No VS Code extensions found to export."
    }
  } else {
    Write-Warning "VS Code extensions folder not found at $vscodeExtDir"
  }
  
  # Export installed WinGet packages
  Write-Section "Export WinGet Packages"
  if (Get-Command winget -ErrorAction SilentlyContinue) {
    Write-Host "Exporting installed WinGet packages..."
    
    try {
      $packagesFile = Join-Path $configDir "winget-packages.json"
      $tempFile = Join-Path $env:TEMP "winget-export-temp.json"
      
      # Export to temp file (winget export writes JSON to file, warnings to console)
      $null = winget export -o $tempFile --accept-source-agreements 2>&1
      
      if (Test-Path $tempFile) {
        # Read and validate JSON
        $content = Get-Content $tempFile -Raw
        $parsed = $content | ConvertFrom-Json
        
        # Write validated JSON to destination
        $content | Set-Content $packagesFile -Encoding UTF8
        Remove-Item $tempFile -Force -ErrorAction SilentlyContinue
        
        $packageCount = ($parsed.Sources | ForEach-Object { $_.Packages.Count } | Measure-Object -Sum).Sum
        Write-Host "Exported $packageCount WinGet packages to: winget-packages.json"
      } else {
        Write-Warning "WinGet export failed - no output file created."
      }
    } catch {
      Write-Warning "Failed to export WinGet packages: $($_.Exception.Message)"
      Remove-Item $tempFile -Force -ErrorAction SilentlyContinue
    }
  } else {
    Write-Warning "WinGet not found. Skipping packages export."
  }
  
  # Export PowerShell profile
  Write-Section "Export PowerShell Profile"
  $psExport = Join-Path $dotfilesPath "powershell"
  New-Item -ItemType Directory -Force -Path $psExport | Out-Null
  
  if (Test-Path $PROFILE) {
    Copy-Item $PROFILE -Destination (Join-Path $psExport "Microsoft.PowerShell_profile.ps1") -Force
    Write-Host "Exported PowerShell profile"
  } else {
    # Copy default profile from repo if user has no profile
    $defaultProfile = Join-Path $repoRoot "dotfiles\powershell\Microsoft.PowerShell_profile.ps1"
    if (Test-Path $defaultProfile) {
      Copy-Item $defaultProfile -Destination (Join-Path $psExport "Microsoft.PowerShell_profile.ps1") -Force
      Write-Host "No user profile found. Exported default profile from repo."
    } else {
      Write-Warning "PowerShell profile not found at $PROFILE and no default in repo."
    }
  }
  
  # Export Git config
  Write-Section "Export Git Config"
  $gitExport = Join-Path $dotfilesPath "git"
  New-Item -ItemType Directory -Force -Path $gitExport | Out-Null
  
  $globalGitConfig = Join-Path $env:USERPROFILE ".gitconfig"
  if (Test-Path $globalGitConfig) {
    Copy-Item $globalGitConfig -Destination (Join-Path $gitExport ".gitconfig") -Force
    Write-Host "Exported .gitconfig"
  } else {
    Write-Warning "Global .gitconfig not found"
  }
  
  Write-Section "Export Complete"
  Write-Host "Configuration exported to: $ConfigRoot"
  Write-Host "  - dotfiles/vscode/"
  Write-Host "  - dotfiles/powershell/"
  Write-Host "  - dotfiles/git/"
  Write-Host "  - config/winget-packages.json"
  Write-Host "  - config/vscode-extensions.json"
  
  # Export WSL dotfiles
  Export-WSLDotfiles -ConfigRoot $ConfigRoot
}

# ============================================================================
# WSL EXPORT FUNCTION
# ============================================================================
function Export-WSLDotfiles {
  param(
    [string]$ConfigRoot,
    [string]$DistroName = "Ubuntu-24.04"
  )
  
  Write-Section "Export WSL Dotfiles"
  
  if (-not (Test-WSLAvailable -DistroName $DistroName)) {
    Write-Warning "WSL distro '$DistroName' not available. Skipping WSL export."
    return
  }
  
  $wslExportPath = Join-Path $ConfigRoot "dotfiles\wsl"
  New-Item -ItemType Directory -Force -Path $wslExportPath | Out-Null
  
  # Get WSL home directory path  
  $wslHome = (Invoke-WSLCommand -Command 'echo $HOME' -DistroName $DistroName).Trim()
  Write-Host "WSL Home: $wslHome"
  
  # Export individual dotfiles
  foreach ($dotfile in $script:WSLDotfiles) {
    $wslFilePath = "$wslHome/$dotfile"
    $exists = Invoke-WSLCommand -Command "test -f '$wslFilePath' && echo 'yes' || echo 'no'" -DistroName $DistroName
    
    if ($exists.Trim() -eq "yes") {
      $destFile = Join-Path $wslExportPath $dotfile
      $destDir = Split-Path -Parent $destFile
      New-Item -ItemType Directory -Force -Path $destDir | Out-Null
      
      # Read file content from WSL and write to Windows
      $content = Invoke-WSLCommand -Command "cat '$wslFilePath'" -DistroName $DistroName
      $content | Set-Content -Path $destFile -Encoding UTF8 -NoNewline
      Write-Host "Exported: $dotfile"
    }
  }
  
  # Export directories
  foreach ($dotdir in $script:WSLDotDirs) {
    $wslDirPath = "$wslHome/$dotdir"
    $exists = Invoke-WSLCommand -Command "test -d '$wslDirPath' && echo 'yes' || echo 'no'" -DistroName $DistroName
    
    if ($exists.Trim() -eq "yes") {
      $destDir = Join-Path $wslExportPath $dotdir
      New-Item -ItemType Directory -Force -Path $destDir | Out-Null
      
      # Use tar to copy directory contents
      $wslDestPath = ConvertTo-WSLPath -WindowsPath $destDir
      Invoke-WSLCommand -Command "cd '$wslDirPath' && tar cf - . 2>/dev/null | (cd '$wslDestPath' && tar xf -)" -DistroName $DistroName
      Write-Host "Exported directory: $dotdir/"
      
      # Special warning for .ssh
      if ($dotdir -eq ".ssh") {
        Write-Host "  [!] Note: .ssh contains sensitive keys. Review before committing to version control." -ForegroundColor Yellow
      }
    }
  }
  
  # Export list of installed apt packages (for reference)
  Write-Host "Exporting installed apt packages list..."
  $aptPackages = Invoke-WSLCommand -Command "apt list --installed 2>/dev/null | grep -v 'Listing...' | cut -d'/' -f1" -DistroName $DistroName
  $aptPackages | Set-Content -Path (Join-Path $wslExportPath "installed-packages.txt") -Encoding UTF8
  
  Write-Host "WSL dotfiles exported to: $wslExportPath"
}

# ============================================================================
# WSL IMPORT FUNCTION
# ============================================================================
function Import-WSLDotfiles {
  param(
    [string]$ConfigRoot,
    [string]$DistroName = "Ubuntu-24.04"
  )
  
  Write-Section "Import WSL Dotfiles"
  
  if (-not (Test-WSLAvailable -DistroName $DistroName)) {
    Write-Warning "WSL distro '$DistroName' not available. Skipping WSL dotfiles import."
    return
  }
  
  $wslImportPath = Join-Path $ConfigRoot "dotfiles\wsl"
  if (-not (Test-Path $wslImportPath)) {
    Write-Warning "No WSL dotfiles found at $wslImportPath. Skipping."
    return
  }
  
  # Get WSL home directory path
  $wslHome = (Invoke-WSLCommand -Command 'echo $HOME' -DistroName $DistroName).Trim()
  Write-Host "WSL Home: $wslHome"
  
  # Import individual dotfiles
  foreach ($dotfile in $script:WSLDotfiles) {
    $srcFile = Join-Path $wslImportPath $dotfile
    if (Test-Path $srcFile) {
      $wslDestPath = "$wslHome/$dotfile"
      $wslSrcPath = ConvertTo-WSLPath -WindowsPath $srcFile
      
      # Backup existing file if it exists
      Invoke-WSLCommand -Command "if [ -f '$wslDestPath' ]; then cp '$wslDestPath' '$wslDestPath.backup' 2>/dev/null || true; fi" -DistroName $DistroName
      
      # Copy file
      Invoke-WSLCommand -Command "cp '$wslSrcPath' '$wslDestPath'" -DistroName $DistroName
      Write-Host "Imported: $dotfile"
    }
  }
  
  # Import directories
  foreach ($dotdir in $script:WSLDotDirs) {
    $srcDir = Join-Path $wslImportPath $dotdir
    if (Test-Path $srcDir) {
      $wslDestDir = "$wslHome/$dotdir"
      $wslSrcPath = ConvertTo-WSLPath -WindowsPath $srcDir
      
      # Create destination directory
      Invoke-WSLCommand -Command "mkdir -p '$wslDestDir'" -DistroName $DistroName
      
      # Copy directory contents using tar
      Invoke-WSLCommand -Command "cd '$wslSrcPath' && tar cf - . 2>/dev/null | (cd '$wslDestDir' && tar xf -)" -DistroName $DistroName
      Write-Host "Imported directory: $dotdir/"
      
      # Fix permissions for .ssh
      if ($dotdir -eq ".ssh") {
        Write-Host "  Setting correct permissions for .ssh..."
        Invoke-WSLCommand -Command "chmod 700 '$wslDestDir' && chmod 600 '$wslDestDir'/* 2>/dev/null || true && chmod 644 '$wslDestDir'/*.pub 2>/dev/null || true" -DistroName $DistroName
      }
    }
  }
  
  Write-Host "WSL dotfiles imported successfully."
}

# ============================================================================
# SETUP COMMAND
# ============================================================================
function Invoke-Setup {
  param(
    [string]$ConfigRoot,
    [string]$RepoRoot,
    [string]$ConfigPath,
    [switch]$Minimal,
    [switch]$SkipDocker,
    [switch]$SkipWSL,
    [switch]$NoWSLProvision,
    [string]$DotfilesRepo
  )
  
  Write-Section "Preflight"
  if (-not (Get-Command winget -ErrorAction SilentlyContinue)) {
    throw "WinGet not found. Install 'App Installer' from Microsoft Store, then re-run."
  }
  
  if (-not (Test-Path $ConfigPath)) { throw "Missing config: $ConfigPath" }
  
  $isAdmin = Test-IsAdmin
  Write-Host ("Running as Admin: " + $isAdmin)
  Write-Host ("Config Root: " + $ConfigRoot)
  
  # Optional external dotfiles repo
  if ($DotfilesRepo -and -not (Test-Path (Join-Path $ConfigRoot "dotfiles\.external"))) {
    Write-Section "Cloning external dotfiles"
    $dest = Join-Path $ConfigRoot "dotfiles\.external"
    git clone $DotfilesRepo $dest
    Write-Host "External dotfiles cloned to: $dest"
    Write-Host "You can now point scripts to it by setting -DotfilesRepo or editing scripts."
  }
  
  Write-Section "Update WinGet sources"
  try { winget source update | Out-Host } catch { Write-Warning $_.Exception.Message }
  
  # Install packages
  Write-Section "Install packages"
  & (Join-Path $RepoRoot "scripts\winget-install.ps1") `
    -ConfigPath $ConfigPath `
    -Minimal:$Minimal `
    -SkipDocker:$SkipDocker `
    -SkipWSL:$SkipWSL `
    -SkipSpotify:$SkipSpotify `
    -OnlySpotify:$OnlySpotify
  
  if ($OnlySpotify) { return }
  
  # Configure pieces - use ConfigRoot for dotfiles location
  Write-Section "Configure Git"
  & (Join-Path $RepoRoot "scripts\configure-git.ps1") -RepoRoot $ConfigRoot
  
  Write-Section "Configure PowerShell"
  & (Join-Path $RepoRoot "scripts\configure-powershell.ps1") -RepoRoot $ConfigRoot
  
  Write-Section "Configure VS Code"
  & (Join-Path $RepoRoot "scripts\configure-vscode.ps1") -RepoRoot $ConfigRoot -ConfigPath $ConfigPath
  
  Write-Section "Configure Rider (optional settings copy)"
  & (Join-Path $RepoRoot "scripts\configure-rider.ps1") -RepoRoot $ConfigRoot
  
  # Optional: provision WSL (installs zsh, tools, etc. inside Ubuntu)
  if (-not $SkipWSL -and -not $NoWSLProvision) {
    Write-Section "Provision WSL (Ubuntu)"
    & (Join-Path $RepoRoot "scripts\configure-wsl.ps1") -RepoRoot $ConfigRoot
    
    # Import WSL dotfiles after provisioning
    Import-WSLDotfiles -ConfigRoot $ConfigRoot
  } else {
    Write-Host "WSL provisioning skipped."
  }
  
  Write-Section "Done"
  Write-Host "If WSL features were newly enabled, a reboot may be required."
  Write-Host "If Docker prompts for WSL2 backend or requires reboot/logout, do that before using it."
}

# ============================================================================
# UPGRADE COMMAND
# ============================================================================
function Invoke-Upgrade {
  param(
    [string]$ConfigRoot,
    [string]$ConfigPath,
    [switch]$SkipWSL
  )
  
  Write-Section "Upgrade Applications"
  
  if (-not (Get-Command winget -ErrorAction SilentlyContinue)) {
    throw "WinGet not found. Install 'App Installer' from Microsoft Store, then re-run."
  }
  
  # Update WinGet sources
  Write-Section "Update WinGet sources"
  try { winget source update | Out-Host } catch { Write-Warning $_.Exception.Message }
  
  # Upgrade all winget packages
  Write-Section "Upgrade WinGet packages"
  Write-Host "Checking for updates..."
  
  try {
    # List available upgrades first
    $upgrades = winget upgrade --include-unknown 2>$null
    Write-Host $upgrades
    
    # Perform upgrade with user confirmation
    Write-Host ""
    Write-Host "Upgrading all packages..." -ForegroundColor Yellow
    winget upgrade --all --silent --accept-source-agreements --accept-package-agreements | Out-Host
  } catch {
    Write-Warning "WinGet upgrade encountered issues: $($_.Exception.Message)"
  }
  
  # Upgrade VS Code extensions
  if (Get-Command code -ErrorAction SilentlyContinue) {
    Write-Section "Upgrade VS Code Extensions"
    Write-Host "Updating VS Code extensions..."
    
    $configDir = Split-Path -Parent $ConfigPath
    $extensionsFile = Join-Path $configDir "vscode-extensions.json"
    
    if (Test-Path $extensionsFile) {
      $extConfig = Get-Content $extensionsFile -Raw | ConvertFrom-Json
      $extensions = @($extConfig.extensions)
      
      foreach ($ext in $extensions) {
        try {
          code --install-extension $ext --force 2>&1 | Out-Null
        } catch {
          Write-Warning "Failed updating VS Code extension $ext"
        }
      }
      Write-Host "VS Code extensions updated."
    } else {
      Write-Host "No vscode-extensions.json found. Skipping."
    }
  }
  
  # Upgrade WSL packages
  if (-not $SkipWSL -and (Test-WSLAvailable)) {
    Write-Section "Upgrade WSL Packages"
    Write-Host "Updating apt packages in WSL..."
    
    try {
      Invoke-WSLCommand -Command "sudo apt-get update -y && sudo apt-get upgrade -y && sudo apt-get autoremove -y"
      Write-Host "WSL packages updated."
    } catch {
      Write-Warning "WSL upgrade encountered issues: $($_.Exception.Message)"
    }
  }
  
  Write-Section "Upgrade Complete"
  Write-Host "All applications have been updated."
}

# ============================================================================
# MAIN EXECUTION
# ============================================================================
switch ($Command) {
  "export" {
    Invoke-Export -ConfigRoot $ConfigRoot
  }
  "setup" {
    Invoke-Setup `
      -ConfigRoot $ConfigRoot `
      -RepoRoot $repoRoot `
      -ConfigPath $configPath `
      -Minimal:$Minimal `
      -SkipDocker:$SkipDocker `
      -SkipWSL:$SkipWSL `
      -SkipSpotify:$SkipSpotify `
      -OnlySpotify:$OnlySpotify `
      -NoWSLProvision:$NoWSLProvision `
      -DotfilesRepo $DotfilesRepo
  }
  "upgrade" {
    Invoke-Upgrade `
      -ConfigRoot $ConfigRoot `
      -ConfigPath $configPath `
      -SkipWSL:$SkipWSL
  }
}
