#!/usr/bin/env bash
set -e

# Standalone Installer for M-Game Solo on Linux
PREFIX="${PREFIX:-$HOME/.local}"
BINDIR="$PREFIX/bin"
DESKTOPDIR="$PREFIX/share/applications"
ICONDIR="$PREFIX/share/icons/hicolor/scalable/apps"
METAINFODIR="$PREFIX/share/metainfo"
UDEVDIR="/etc/udev/rules.d"

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

echo "=== Installing M-Game Solo to $PREFIX ==="

mkdir -p "$BINDIR" "$DESKTOPDIR" "$ICONDIR" "$METAINFODIR"

install -m 755 bin/mgame-solo "$BINDIR/mgame-solo"
install -m 644 data/com.mgame.Solo.desktop "$DESKTOPDIR/com.mgame.Solo.desktop"
install -m 644 data/com.mgame.Solo.svg "$ICONDIR/com.mgame.Solo.svg"
install -m 644 data/com.mgame.Solo.metainfo.xml "$METAINFODIR/com.mgame.Solo.metainfo.xml"

# Update desktop & icon caches
if command -v update-desktop-database &>/dev/null; then
    update-desktop-database "$DESKTOPDIR" 2>/dev/null || true
fi
if command -v gtk-update-icon-cache &>/dev/null; then
    gtk-update-icon-cache -f -t "$PREFIX/share/icons/hicolor" 2>/dev/null || true
fi

# Optional udev rule installation for non-root hardware access
if [ -d "$UDEVDIR" ] && [ -w "$UDEVDIR" ]; then
    echo "Installing hardware udev rules to $UDEVDIR..."
    install -m 644 data/99-mgame-solo.rules "$UDEVDIR/99-mgame-solo.rules"
    udevadm control --reload-rules 2>/dev/null || true
    udevadm trigger 2>/dev/null || true
elif [ -f data/99-mgame-solo.rules ]; then
    echo ""
    echo "NOTE: To grant non-root USB/RawMIDI permissions, run:"
    echo "  sudo cp data/99-mgame-solo.rules /etc/udev/rules.d/"
    echo "  sudo udevadm control --reload-rules && sudo udevadm trigger"
fi

echo ""
echo "Installation complete! Launch with 'mgame-solo' or from your app launcher."
