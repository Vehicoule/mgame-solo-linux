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
rm -f "$PREFIX/share/icons/hicolor/128x128/apps/com.mgame.Solo.png"
rm -f "$PREFIX/share/metainfo/com.mgame.Solo.metainfo.xml"

if [[ "$(id -u)" -eq 0 && -f /etc/udev/rules.d/99-mgame-solo.rules ]]; then
    rm -f /etc/udev/rules.d/99-mgame-solo.rules
    udevadm control --reload-rules 2>/dev/null || true
fi

# Remove PipeWire virtual devices provisioned by the app
echo "Removing PipeWire virtual devices..."
rm -f "$HOME/.config/pipewire/pipewire.conf.d/50-m-game-solo.conf" 2>/dev/null || true
rm -f "$HOME/.var/app/com.mgame.Solo/config/pipewire/pipewire.conf.d/50-m-game-solo.conf" 2>/dev/null || true
if [ -n "${SNAP_REAL_HOME:-}" ]; then
    rm -f "$SNAP_REAL_HOME/.config/pipewire/pipewire.conf.d/50-m-game-solo.conf" 2>/dev/null || true
fi
if command -v flatpak-spawn >/dev/null 2>&1; then
    flatpak-spawn --host rm -f "$HOME/.config/pipewire/pipewire.conf.d/50-m-game-solo.conf" 2>/dev/null || true
fi
# Restart PipeWire to apply removal (best-effort)
if command -v systemctl >/dev/null 2>&1; then
    systemctl --user restart pipewire.service wireplumber.service 2>/dev/null || true
    if command -v flatpak-spawn >/dev/null 2>&1; then
        flatpak-spawn --host systemctl --user restart pipewire.service wireplumber.service 2>/dev/null || true
    fi
else
    echo "Please restart PipeWire manually: systemctl --user restart pipewire wireplumber"
fi

command -v update-desktop-database >/dev/null 2>&1 && \
    update-desktop-database "$PREFIX/share/applications" 2>/dev/null || true

echo "M-Game Solo uninstalled."
