#!/bin/bash
# Uninstallation script for OpenVPN3 GUI (libcosmic version)

set -e

APP_ID="xyz.fonzi.openvpn3gui"
BIN_NAME="openvpn_gui"
BIN_PATH="$HOME/.local/bin/$BIN_NAME"
DESKTOP_PATH="$HOME/.local/share/applications/${APP_ID}.desktop"

echo "Uninstalling OpenVPN3 GUI (libcosmic)..."

# Remove binary
rm -f "$BIN_PATH" && echo "Removed binary $BIN_PATH"

# Remove desktop file
rm -f "$DESKTOP_PATH" && echo "Removed desktop entry $DESKTOP_PATH"

# Remove icon(s)
for size in 16 24 32 48 64 128 256; do
    ICON_PATH="$HOME/.local/share/icons/hicolor/${size}x${size}/apps/${APP_ID}.png"
    if [ -f "$ICON_PATH" ]; then
        rm -f "$ICON_PATH" && echo "Removed icon $ICON_PATH"
    fi
done

# Update desktop database
update-desktop-database "$HOME/.local/share/applications" 2>/dev/null || true

# Update icon cache
gtk-update-icon-cache "$HOME/.local/share/icons/hicolor" 2>/dev/null || true

echo ""
echo "Uninstallation complete!"
echo "If you want to remove config or recent files, delete ~/.config/openvpn-gui/ manually."
