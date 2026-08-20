#!/usr/bin/env bash
set -e

PREFIX="${PREFIX:-$HOME/.local}"
BINDIR="$PREFIX/bin"
DESKTOPDIR="$PREFIX/share/applications"
ICONDIR="$PREFIX/share/icons/hicolor/scalable/apps"
METAINFODIR="$PREFIX/share/metainfo"

echo "=== Uninstalling M-Game Solo ==="

rm -f "$BINDIR/mgame-solo"
rm -f "$DESKTOPDIR/com.mgame.Solo.desktop"
rm -f "$ICONDIR/com.mgame.Solo.svg"
rm -f "$METAINFODIR/com.mgame.Solo.metainfo.xml"

if [ -w "/etc/udev/rules.d/99-mgame-solo.rules" ]; then
    rm -f "/etc/udev/rules.d/99-mgame-solo.rules"
    udevadm control --reload-rules 2>/dev/null || true
fi

if command -v update-desktop-database &>/dev/null; then
    update-desktop-database "$DESKTOPDIR" 2>/dev/null || true
fi

echo "M-Game Solo has been uninstalled."
