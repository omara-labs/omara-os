#!/usr/bin/env bash

# Fedora Workspace Optimization Setup Script
# Wrapper script that runs each category-specific script under scripts/ sequentially.

set -euo pipefail

# Ensure the script is NOT run as root directly (to preserve user environment settings/GNOME configurations)
if [ "$EUID" -eq 0 ]; then
    echo "❌ Please do not run this script as root/sudo directly."
    echo "   Run it as your normal user. It will prompt for sudo where root permissions are needed."
    exit 1
fi

echo "🚀 Starting Omara Workstation Setup..."

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" &>/dev/null && pwd)"
export SCRIPT_DIR

# Load configuration profile
CONFIG_PATH="$SCRIPT_DIR/configs/omara.conf"
OMARA_SESSION="gnome"
WAYLAND_COMPOSITOR="niri"

if [ -f "$CONFIG_PATH" ]; then
    echo "⚙️  Loading configuration from $(basename "$CONFIG_PATH")..."
    # shellcheck disable=SC1090
    source "$CONFIG_PATH"
else
    echo "⚠️  Configuration file not found. Using defaults (GNOME)."
fi

export OMARA_SESSION
export WAYLAND_COMPOSITOR

echo "   Profile Session: $OMARA_SESSION"
if [ "$OMARA_SESSION" = "wayland" ]; then
    echo "   Compositor: $WAYLAND_COMPOSITOR"
fi


# Run all scripts in the legacy directory in numerical order
for script in "$SCRIPT_DIR"/[0-9]*.sh; do
    if [ -x "$script" ]; then
        echo ""
        echo "=========================================================="
        echo "🏃 Running: $(basename "$script")"
        echo "=========================================================="
        "$script"
    fi
done

echo ""
echo "=========================================================="
echo "✨ Optimization Complete!"
echo "🔄 Please log out of your session and log back in for changes to take full effect."
echo "=========================================================="
