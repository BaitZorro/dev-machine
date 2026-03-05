# PowerShell profile (dotfiles)
# Loaded from: $PROFILE
# Requires: Oh My Posh (JanDeDobbeleer.OhMyPosh) - installed via winget in this repo.

# Faster completion
Set-PSReadLineOption -PredictionSource History
Set-PSReadLineOption -EditMode Windows

# Oh My Posh prompt (safe if not installed)
try {
  oh-my-posh init pwsh --config "$env:POSH_THEMES_PATH\jandedobbeleer.omp.json" | Invoke-Expression
} catch { }

# Useful aliases
Set-Alias ll Get-ChildItem
function .. { Set-Location .. }
function ... { Set-Location ../.. }

# Git shortcuts (if git exists)
if (Get-Command git -ErrorAction SilentlyContinue) {
  function gs { git status }
  function gl { git log --oneline --decorate --graph -n 30 }
}

# Kubernetes shortcuts (if kubectl exists)
if (Get-Command kubectl -ErrorAction SilentlyContinue) {
  function k { kubectl @args }
  function kns($ns) { kubectl config set-context --current --namespace=$ns }
}

# Docker shortcuts
if (Get-Command docker -ErrorAction SilentlyContinue) {
  function dps { docker ps }
  function dim { docker images }
}
