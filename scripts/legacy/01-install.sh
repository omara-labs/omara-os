#!/usr/bin/env bash
set -euo pipefail

echo "🎬 Swapping ffmpeg-free for full ffmpeg (RPM Fusion)..."
sudo dnf swap -y ffmpeg-free ffmpeg --allowerasing

# Define common packages installed in all sessions
PACKAGES=(
    fish
    kitty
    steam
    msty-studio
    ledger-live-desktop
    synology-drive-client
    tailscale
    walker
    elephant
    nmap-ncat
    syncthing
    openrgb
    openrgb-udev-rules
    cockpit
    cockpit-podman
    cockpit-pcp
    podman-compose
    distrobox
    git
    make
    gcc
    gcc-c++
    gettext
    golang
    nodejs
    rustup
    python3-devel
    gstreamer1-plugins-bad-freeworld
    gstreamer1-plugins-ugly
    libde265
    svt-hevc-libs
)

if [ "${OMARA_SESSION:-gnome}" = "gnome" ]; then
    echo "📦 Adding GNOME-specific packages..."
    PACKAGES+=(
        gnome-shell-extension-pop-shell
        gnome-shell-extension-appindicator
        gnome-shell-extensions-common
    )
elif [ "${OMARA_SESSION:-gnome}" = "wayland" ]; then
    echo "📦 Adding Wayland compositor-specific packages..."
    PACKAGES+=(
        quickshell
        yazi
        ly
        pulsemixer
        brightnessctl
        google-chrome-stable
        microsoft-edge-stable
        libreoffice
    )
    if [ "${WAYLAND_COMPOSITOR:-niri}" = "niri" ]; then
        PACKAGES+=(niri)
    elif [ "${WAYLAND_COMPOSITOR:-niri}" = "hyprland" ]; then
        PACKAGES+=(hyprland)
    fi
fi

echo "📥 Installing packages..."
sudo dnf install -y "${PACKAGES[@]}"

echo "🛍️  Installing Bazaar graphical store (Flatpak)..."
sudo flatpak install -y flathub io.github.kolunmi.Bazaar

echo "✓ All packages successfully installed!"

