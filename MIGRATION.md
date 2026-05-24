# Migration from old omara-labs/os repo (Big Bang)

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

### Into omara-de/hyprland/
- Full new desktop definition (Waybar, SwayNC, dock, window rules, scripts)
- legacy-hyprland.conf (your old full config for reference)

### Into omara-de/niri/
- legacy-config.kdl

### Into omara-os/
- scripts/legacy/ (00-repos.sh, 01-install.sh, 02-services.sh, 03-user-services.sh, 06-app-configs.sh, setup.sh)

### Deprecated / left behind
- All GNOME-specific scripts (04-gnome.sh, 05-shortcuts.sh)
- pop-shell, quickshell, and other non-core pieces

The old `os/` repo is now considered the legacy source of truth only for historical reference.
