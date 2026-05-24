#!/usr/bin/env bash
set -euo pipefail

echo "⚙️  Enabling and starting Tailscale system service..."
sudo systemctl enable --now tailscaled

echo "⚙️  Enabling and starting Cockpit system service..."
sudo systemctl enable --now cockpit.socket

if [ "${OMARA_SESSION:-gnome}" = "wayland" ]; then
    echo "⚙️  Enabling Ly display manager system service..."
    sudo systemctl disable gdm.service || true
    sudo systemctl enable ly.service || true
fi

