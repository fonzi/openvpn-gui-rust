#!/bin/bash
# Installation script for OpenVPN3 GUI (libcosmic version)

set -e

APP_ID="xyz.fonzi.openvpn3gui"
BIN_NAME="openvpn_gui"
BIN_PATH="$HOME/.local/bin/$BIN_NAME"
DESKTOP_PATH="$HOME/.local/share/applications/openvpn3-gui.desktop"

echo "Installing OpenVPN3 GUI (libcosmic)..."
echo ""

# Note: System dependencies check removed - cargo will fail with clear error if libs are missing
# Required: libxkbcommon-dev libfontconfig-dev libfreetype-dev libexpat1-dev pkg-config

# Build release version
echo "Building release version with libcosmic..."
cargo build --release

# Create directories if they don't exist
mkdir -p "$HOME/.local/bin"
mkdir -p "$HOME/.local/share/applications"
mkdir -p "$HOME/.local/share/icons"

# Copy binary
echo "Installing binary to $BIN_PATH..."
cp target/release/$BIN_NAME "$BIN_PATH"
chmod +x "$BIN_PATH"

# Install .desktop file
echo "Installing desktop file..."
cp openvpn3-gui.desktop "$DESKTOP_PATH"
# Update Exec path in desktop file to use absolute path
sed -i "s|Exec=openvpn_gui|Exec=$BIN_PATH|" "$DESKTOP_PATH"

# Install icons with correct naming
echo "Installing icons..."
mkdir -p "$HOME/.local/share/icons/hicolor/256x256/apps"

# Install main 256x256 icon (required)
if [ -f "icons/openvpn3-gui.png" ]; then
    cp "icons/openvpn3-gui.png" "$HOME/.local/share/icons/hicolor/256x256/apps/${APP_ID}.png"
    echo "Installed 256x256 icon"
fi

# Install other icon sizes
for size in 16 24 32 48 64 128; do
    ICON_SRC="icons/openvpn3-gui-${size}.png"
    ICON_DIR="$HOME/.local/share/icons/hicolor/${size}x${size}/apps"
    ICON_DEST="${ICON_DIR}/${APP_ID}.png"
    if [ -f "$ICON_SRC" ]; then
        mkdir -p "$ICON_DIR"
        cp "$ICON_SRC" "$ICON_DEST"
        echo "Installed ${size}x${size} icon"
    fi
done

# Update desktop database
if command -v update-desktop-database &> /dev/null; then
    echo "Updating desktop database..."
    update-desktop-database "$HOME/.local/share/applications" 2>/dev/null || true
fi

# Update icon cache
if command -v gtk-update-icon-cache &> /dev/null; then
    echo "Updating icon cache..."
    gtk-update-icon-cache "$HOME/.local/share/icons/hicolor" 2>/dev/null || true
fi

echo ""
echo "Installation complete!"
echo ""
echo "OpenVPN3 GUI has been installed with libcosmic support."
echo ""
echo "You can now:"
echo "  - Run 'openvpn_gui' from the terminal"
echo "  - Find 'OpenVPN3 GUI' in your application menu"
echo "  - Native COSMIC DE integration with automatic theme switching"
echo ""
echo "Note: Make sure ~/.local/bin is in your PATH"
echo ""
echo "For COSMIC Desktop users:"
echo "  - The app will automatically match your system theme"
echo "  - System tray icon available in the top panel"
