#!/usr/bin/env bash
set -euo pipefail

echo "⚙️  Optimizing DNF5 settings..."
sudo sed -i '/max_parallel_downloads/d' /etc/dnf/dnf.conf
sudo sed -i '/fastestmirror/d' /etc/dnf/dnf.conf
echo "max_parallel_downloads=10" | sudo tee -a /etc/dnf/dnf.conf > /dev/null
echo "fastestmirror=True" | sudo tee -a /etc/dnf/dnf.conf > /dev/null

echo "📦 Enabling RPM Fusion Repositories..."
sudo dnf install -y https://mirrors.rpmfusion.org/free/fedora/rpmfusion-free-release-$(rpm -E %fedora).noarch.rpm \
                    https://mirrors.rpmfusion.org/nonfree/fedora/rpmfusion-nonfree-release-$(rpm -E %fedora).noarch.rpm

echo "🚀 Enabling Walker Copr Repository..."
sudo dnf copr enable -y errornointernet/walker

echo "🌐 Enabling Tailscale Repository..."
sudo dnf config-manager addrepo --from-repofile=https://pkgs.tailscale.com/stable/fedora/tailscale.repo

echo "📦 Enabling Terra Repository..."
sudo dnf install -y --nogpgcheck --repofrompath "terra,https://repos.fyralabs.com/terra$(rpm -E %fedora)" terra-release || true

echo "📦 Enabling Omara Labs RPM Repository..."
echo '[omara-rpms]
name=Omara Labs DNF Repository
baseurl=https://omara-labs.github.io/RPMs/
enabled=1
gpgcheck=0
metadata_expire=1h' | sudo tee /etc/yum.repos.d/omara-rpms.repo > /dev/null


if [ "${OMARA_SESSION:-gnome}" = "wayland" ]; then
    echo "🚀 Enabling Wayland Session COPR Repositories (Niri, Quickshell, Yazi)..."
    sudo dnf copr enable -y yalter/niri
    sudo dnf copr enable -y errornointernet/quickshell
    sudo dnf copr enable -y lihaohong/yazi

    echo "🌐 Enabling Google Chrome and Microsoft Edge Repositories..."
    sudo dnf config-manager addrepo --from-repofile=https://dl.google.com/linux/chrome/rpm/stable/x86_64/google-chrome.repo || true
    sudo dnf config-manager addrepo --from-repofile=https://packages.microsoft.com/yumrepos/edge/config.repo || true
fi

echo "🛍️  Installing Flatpak package..."
sudo dnf install -y flatpak

echo "🛍️  Configuring unfiltered Flathub Remote..."
sudo flatpak remote-add --if-not-exists flathub https://dl.flathub.org/repo/flathub.flatpakrepo
sudo flatpak remote-modify --no-filter flathub

echo "✓ All repositories successfully configured!"

