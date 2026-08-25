#!/usr/bin/env bash
set -euo pipefail

# Minimal installer for the mgame-solo release tarball.
# Usage: ./install.sh [PREFIX]   (default: ~/.local, or /usr/local as root)

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

BIN_DIR="$PREFIX/bin"
DATA_DIR="$PREFIX/share"
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

for file in "$ROOT/bin/mgame-solo" "$ROOT/data/com.mgame.Solo.desktop"; do
    if [[ ! -f "$file" ]]; then
        echo "Error: expected release layout missing ($file). Run this script from the extracted tarball root." >&2
        exit 1
    fi
done

echo "Installing M-Game Solo to $PREFIX ..."
install -Dm755 "$ROOT/bin/mgame-solo" "$BIN_DIR/mgame-solo"
install -Dm644 "$ROOT/data/com.mgame.Solo.desktop" "$DATA_DIR/applications/com.mgame.Solo.desktop"
install -Dm644 "$ROOT/data/com.mgame.Solo.svg" "$DATA_DIR/icons/hicolor/scalable/apps/com.mgame.Solo.svg"
install -Dm644 "$ROOT/data/com.mgame.Solo.metainfo.xml" "$DATA_DIR/metainfo/com.mgame.Solo.metainfo.xml"

# udev rules grant non-root access to the device; needs write access to /etc
UDEV_RULE="$ROOT/data/99-mgame-solo.rules"
if [[ -f "$UDEV_RULE" ]] && [[ -w /etc/udev/rules.d || "$(id -u)" -eq 0 ]]; then
    install -Dm644 "$UDEV_RULE" /etc/udev/rules.d/99-mgame-solo.rules
    udevadm control --reload-rules 2>/dev/null || true
else
    echo "Skipped udev rule (no permission). Install it manually:"
    echo "  sudo cp $UDEV_RULE /etc/udev/rules.d/"
fi

command -v update-desktop-database >/dev/null 2>&1 && \
    update-desktop-database "$DATA_DIR/applications" 2>/dev/null || true

echo "Done. Run 'mgame-solo' or launch 'M-Game Solo' from your application menu."
