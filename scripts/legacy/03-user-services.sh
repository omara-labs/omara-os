#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="${SCRIPT_DIR:-$(cd "$(dirname "${BASH_SOURCE[0]}")/.." &>/dev/null && pwd)}"

echo "⚙️  Enabling and starting Elephant user service..."
elephant service enable || true
systemctl --user enable --now elephant.service || true

echo "⚙️  Deploying and enabling Walker user service..."
mkdir -p ~/.config/systemd/user
if [ -f "$SCRIPT_DIR/configs/systemd/user/walker.service" ]; then
    cp "$SCRIPT_DIR/configs/systemd/user/walker.service" ~/.config/systemd/user/walker.service
    systemctl --user daemon-reload
    systemctl --user enable --now walker.service || true
    echo "   Walker user service enabled and started."
else
    echo "   ⚠️ configs/systemd/user/walker.service not found, skipping Walker service setup."
fi

echo "⚙️  Enabling and starting Syncthing user service..."
systemctl --user enable --now syncthing.service || true

