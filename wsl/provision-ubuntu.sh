#!/usr/bin/env bash
set -euo pipefail

echo "== WSL Provisioning (Ubuntu) =="

# Update + essentials
sudo apt-get update -y
sudo apt-get upgrade -y

sudo apt-get install -y \
  ca-certificates \
  curl \
  git \
  unzip \
  zip \
  build-essential \
  jq \
  ripgrep \
  fd-find \
  bat \
  fzf \
  zsh

# Symlinks for fd/bat on Ubuntu
if command -v fdfind >/dev/null 2>&1 && ! command -v fd >/dev/null 2>&1; then
  sudo ln -sf "$(command -v fdfind)" /usr/local/bin/fd || true
fi
if command -v batcat >/dev/null 2>&1 && ! command -v bat >/dev/null 2>&1; then
  sudo ln -sf "$(command -v batcat)" /usr/local/bin/bat || true
fi

# Optional: install oh-my-zsh (non-interactive)
if [ ! -d "$HOME/.oh-my-zsh" ]; then
  echo "Installing oh-my-zsh..."
  RUNZSH=no CHSH=no KEEP_ZSHRC=yes \
    sh -c "$(curl -fsSL https://raw.githubusercontent.com/ohmyzsh/ohmyzsh/master/tools/install.sh)" || true
fi

# Set zsh as default shell (best-effort)
if command -v zsh >/dev/null 2>&1; then
  if [ "${SHELL:-}" != "$(command -v zsh)" ]; then
    echo "Attempting to set zsh as default shell..."
    chsh -s "$(command -v zsh)" || true
  fi
fi

echo "WSL provisioning complete."
echo "Tip: Restart your WSL session for default shell changes to apply."
