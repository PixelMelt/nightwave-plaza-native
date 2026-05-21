#!/usr/bin/env bash
# Install nightwave-plaza as a desktop app on Linux.
# Builds + installs the binary to ~/.cargo/bin, then registers the
# .desktop entry and icon so it shows up in your application launcher.
set -euo pipefail

repo_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

echo "Installing binary via cargo..."
cargo install --path "$repo_dir" --force

echo "Installing icon..."
icon_dir="${XDG_DATA_HOME:-$HOME/.local/share}/icons/hicolor/256x256/apps"
mkdir -p "$icon_dir"
install -m644 "$repo_dir/src/assets/icons/nightwave-plaza.png" "$icon_dir/nightwave-plaza.png"

echo "Installing desktop entry..."
app_dir="${XDG_DATA_HOME:-$HOME/.local/share}/applications"
mkdir -p "$app_dir"
install -m644 "$repo_dir/dist/nightwave-plaza.desktop" "$app_dir/nightwave-plaza.desktop"

# Refresh caches if the tools are available (harmless if not).
update-desktop-database "$app_dir" 2>/dev/null || true
gtk-update-icon-cache "${XDG_DATA_HOME:-$HOME/.local/share}/icons/hicolor" 2>/dev/null || true

echo "Done. 'Nightwave Plaza' should now appear in your launcher."
