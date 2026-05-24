# Migration from old omara-labs/os repo (Big Bang)

> **Note**: The old short-named `os` repository has been deleted. This document is kept for historical reference.

The original single-repo `os/` has been decomposed.

## What was migrated (finished 2026-05-23)

### Into omara-core/
- configs/kitty/kitty.conf
- configs/fish/
- configs/systemd/user/walker.service
- configs/razer/ (udev + red script)
- configs/omara.conf.legacy (reference)
- scripts/hardware-setup.sh (adapted)
- scripts/omara-screensaver

### Into omara-de/niri/
- Full desktop definition (Quickshell, SwayNC, window rules, scripts)
- config.kdl (main Niri configuration)

### Into omara-os/
- scripts/legacy/ (00-repos.sh, 01-install.sh, 02-services.sh, 03-user-services.sh, 06-app-configs.sh, setup.sh)

### Deprecated / left behind
- All GNOME-specific scripts (04-gnome.sh, 05-shortcuts.sh)
- pop-shell and other non-core pieces
- Hyprland desktop definition (removed in favor of Niri)

The old short-named `os` repository (now deleted) was the legacy source of truth.
