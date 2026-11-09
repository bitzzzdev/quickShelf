# Changelog

All notable changes to this project will be documented in this file.

The format is based on Keep a Changelog, and this project adheres to Semantic Versioning.

## [Unreleased]

### Fixed
- Fixed GTK3 and gtk-layer-shell compilation failures in GitHub Actions CI runner.
- Corrected enum variant casing to PascalCase for `Orientation`, `Align`, and `StackTransitionType`.
- Replaced deprecated file chooser constructor with `FileChooserDialog::with_buttons` and owned `FileFilter`.
- Integrated `LayerShell` trait methods for layer configuration, anchors, margins, and keyboard interactivity.
- Removed unused imports and dead code warnings across modules.

### Added
- Added Rofi-like CSS theming system powered by `@define-color` GTK color definitions.
- Added eight built-in themes: `default`, `tokyonight`, `catppuccin`, `nord`, `dracula`, `gruvbox`, `everforest`, and `rose-pine`.
- Added support for custom theme stylesheets in `~/.config/quick-shelf/themes/<theme>.css`.
- Added support for automated wallpaper colors via `~/.config/quick-shelf/colors.css` (Matugen, Pywal, Wallust compatible).
- Added `--theme <NAME>` command line option to set the active theme and switch running instances.
- Added `--list-themes` command line option to display available built-in and user-installed themes.
- Added `set-theme <name>` command to Unix domain socket IPC for instant live switching.
- Added `theme` configuration key in `~/.config/quick-shelf/config.json`.
- Added sample theme stylesheets in `themes/` directory for dotfile sharing.
- Added repository description and metadata in Cargo manifest.
- Added unit tests for desktop entry parsing, exec argument sanitation, and image format detection.

## [0.1.0] - 2026-11-08

### Added
- Initial release of Quick Shelf widget.
- Persistent homescreen widget mode using Wayland Layer Shell (bottom layer behind windows).
- Popup drawer mode with toggle support via local Unix domain socket IPC.
- App Shortcuts manager with system desktop file search, parsing, and execution.
- Image drop shelf with instant thumbnail generation and rounded preview cards.
- Instant preview lightbox with resolution, file size, copy image data, copy file path, and open in viewer.
- Two-way drag and drop support for dragging images into and out of the shelf via text/uri-list.
- Waybar module support via JSON status reporting (`--status`).
- Dynamic styling loader with live reloading on theme changes.
- Automated GitHub Actions CI workflow to compile and release x86_64 Linux binaries.
