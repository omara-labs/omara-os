#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="${SCRIPT_DIR:-$(cd "$(dirname "${BASH_SOURCE[0]}")/.." &>/dev/null && pwd)}"

echo "🐟 Deploying Fish shell configuration..."
mkdir -p ~/.config/fish
if [ -f "$SCRIPT_DIR/configs/fish/config.fish" ]; then
    cp "$SCRIPT_DIR/configs/fish/config.fish" ~/.config/fish/config.fish
    echo "   Fish config deployed."
else
    echo "   ⚠️ configs/fish/config.fish not found, skipping config deployment."
fi

echo "🐱 Deploying Kitty terminal configuration..."
mkdir -p ~/.config/kitty
if [ -f "$SCRIPT_DIR/configs/kitty/kitty.conf" ]; then
    cp "$SCRIPT_DIR/configs/kitty/kitty.conf" ~/.config/kitty/kitty.conf
    echo "   Kitty configuration deployed."
else
    echo "   ⚠️ configs/kitty/kitty.conf not found, skipping config deployment."
fi

if [ "${OMARA_SESSION:-gnome}" = "wayland" ]; then
    echo "🗺️  Deploying Wayland Compositor and Panel configurations..."
    
    # Niri
    if [ "${WAYLAND_COMPOSITOR:-niri}" = "niri" ]; then
        echo "   Deploying Niri configurations..."
        mkdir -p ~/.config/niri
        if [ -f "$SCRIPT_DIR/configs/niri/config.kdl" ]; then
            cp "$SCRIPT_DIR/configs/niri/config.kdl" ~/.config/niri/config.kdl
        fi
    fi

    # Hyprland
    if [ "${WAYLAND_COMPOSITOR:-niri}" = "hyprland" ]; then
        echo "   Deploying Hyprland configurations..."
        mkdir -p ~/.config/hypr
        if [ -f "$SCRIPT_DIR/configs/hypr/hyprland.conf" ]; then
            cp "$SCRIPT_DIR/configs/hypr/hyprland.conf" ~/.config/hypr/hyprland.conf
        fi
    fi

    # Quickshell
    echo "   Deploying Quickshell status bar configs..."
    mkdir -p ~/.config/quickshell
    if [ -f "$SCRIPT_DIR/configs/quickshell/shell.qml" ]; then
        cp "$SCRIPT_DIR/configs/quickshell/shell.qml" ~/.config/quickshell/shell.qml
    fi
fi

echo "🛍️  Creating user desktop override for Bazaar to add custom keywords (bazarr, appstore)..."
mkdir -p ~/.local/share/applications
SYSTEM_DESKTOP="/var/lib/flatpak/exports/share/applications/io.github.kolunmi.Bazaar.desktop"
USER_DESKTOP="$HOME/.local/share/applications/io.github.kolunmi.Bazaar.desktop"

if [ -f "$SYSTEM_DESKTOP" ]; then
    cp "$SYSTEM_DESKTOP" "$USER_DESKTOP"
    # Append custom keywords to the Keywords= line
    sed -i 's/^Keywords=.*$/&bazarr;appstore;app store;/' "$USER_DESKTOP"
    echo "   Bazaar desktop override created with custom keywords."
fi

