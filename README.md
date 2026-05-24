# omara-os

The source of truth for Omara releases and architecture.

## Purpose

This repository defines what constitutes a complete, coherent release of Omara:

- Version alignment between `omara-configs`, `omara-cli`, `omara-art`, `omara-os`, and `omara-rpms`
- Release manifests and compatibility matrices
- High-level build and ISO orchestration (future)
- Upgrade paths and release notes

## Manifests

Omara uses three package managers. Each has its own manifest directory:

```
manifests/
├── dnf/           # RPM packages (Fedora)
│   ├── repos.txt  # Third-party repositories
│   ├── default.txt
│   └── categories/
│       ├── browsers.txt
│       ├── cloud.txt
│       ├── desktop.txt
│       ├── dev.txt
│       ├── media.txt
│       ├── tools.txt
│       ├── utilities.txt
│       └── wayland-extras.txt
│
├── flatpak/       # Flatpak applications
│   ├── remotes.txt
│   └── apps.txt
│
└── webapps/      # Web-based applications
    └── apps.txt
```

### DNF (RPM)

Native Fedora packages. Installed via `dnf`.

### Flatpak

Sandboxed applications from Flathub. Installed via `flatpak`.

### Webapps

Browser-based applications launched via desktop entries. Config files live in `omara-configs/configs/apps/webapps/`.

## Current Contents

- `releases/` — Version manifests
- `scripts/` — Legacy installation scripts (for reference)
- `MIGRATION.md` — History of the Big Bang refactor

## App Configs

Default configurations for apps are stored in `omara-configs/configs/apps/`. Swapping defaults is handled by `omara-cli`.

## Philosophy

If something is not declared here, it's not part of an official Omara release.

---

*If it's not declared here, it's not Omara.*
