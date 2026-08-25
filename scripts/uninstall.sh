#!/usr/bin/env bash
set -euo pipefail

# Uninstaller for mgame-solo. Usage: ./uninstall.sh [PREFIX]

if [[ $# -gt 1 ]]; then
    echo "Usage: $0 [PREFIX]" >&2
    exit 64
fi

PREFIX="${1:-${PREFIX:-}}"
if [[ -z "$PREFIX" ]]; then
    if [[ "$(id -u)" -eq 0 ]]; then
        PREFIX="/usr/local"
    else
        PREFIX="$HOME/.local"
    fi
fi

echo "Removing M-Game Solo from $PREFIX ..."
rm -f "$PREFIX/bin/mgame-solo"
rm -f "$PREFIX/share/applications/com.mgame.Solo.desktop"
rm -f "$PREFIX/share/icons/hicolor/scalable/apps/com.mgame.Solo.svg"
rm -f "$PREFIX/share/metainfo/com.mgame.Solo.metainfo.xml"

if [[ "$(id -u)" -eq 0 && -f /etc/udev/rules.d/99-mgame-solo.rules ]]; then
    rm -f /etc/udev/rules.d/99-mgame-solo.rules
    udevadm control --reload-rules 2>/dev/null || true
fi

command -v update-desktop-database >/dev/null 2>&1 && \
    update-desktop-database "$PREFIX/share/applications" 2>/dev/null || true

echo "M-Game Solo uninstalled."
