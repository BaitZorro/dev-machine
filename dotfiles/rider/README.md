# Rider settings overlay

Put exported / curated settings overlays here.

This repo's `scripts/configure-rider.ps1` will copy only these subfolders (if present)
into detected Rider config folders under:

  %APPDATA%\JetBrains\Rider20xx.x\

Supported overlay subfolders:
- codestyles/
- keymaps/
- options/
- templates/
- colors/

Tip: For most people, JetBrains **Settings Sync** is the easiest long-term solution.
