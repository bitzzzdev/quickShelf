# Changelog

All notable changes to this project will be documented in this file.

The format is based on Keep a Changelog, and this project adheres to Semantic Versioning.

## [Unreleased]

### Added
- Added 1:1 default square aspect ratio geometry for the shelf surface.
- Added `--aspect-ratio` and `--ratio` commands to change or query default aspect ratio.
- Added Bento box card grid format (`bento`) and vertical list format (`list`) with `v` or `m` key runtime toggle.
- Added half-block graphical thumbnail rendering for images and app logo icon resolution for applications in both bento and list formats.
- Added an in-TUI application search and picker modal (press `a` or `s`) and `--add-app` command to pin installed desktop applications.
- Added external drag-and-drop support (via `x`, middle-click, or dragging outside the shelf area), spawning `ripdrag` or `dragon` when installed and copying to Wayland clipboard.
- Added `--update` command that downloads the latest release binary, stops running instances (`pkill`), and emits a desktop notification.
- Added a mouse-enabled Ratatui terminal UI with keyboard navigation, click
  actions, right-click removal, drag-to-reorder, and bracketed-paste path
  drops.
- Added a documented ricing schema for panel geometry, copy, visibility,
  detail rendering, and application or image section order.
- Added an isolated per-user configuration root at `~/.config/quick/shelf/`
  with migration from the previous `qshelf` and `quick-shelf` locations.
- Added the `bitzdots` built-in profile and made it the default palette.
- Added Rofi-like CSS theming system powered by `@define-color` GTK color definitions.
- Added eight built-in themes: `default`, `tokyonight`, `catppuccin`, `nord`, `dracula`, `gruvbox`, `everforest`, and `rose-pine`.
- Added support for custom theme stylesheets in
  `~/.config/quick/shelf/themes/<theme>.css`.
- Added support for automated wallpaper colors via
  `~/.config/quick/shelf/colors.css` (Matugen, Pywal, Wallust compatible).
- Added `--theme <NAME>` command line option to set the active theme and switch running instances.
- Added `--list-themes` command line option to display available built-in and user-installed themes.
- Added `set-theme <name>` command to Unix domain socket IPC for instant live switching.
- Added `theme` configuration key in `~/.config/quick/shelf/config.json`.
- Added sample theme stylesheets in `themes/` directory for dotfile sharing.
- Added repository description and metadata in Cargo manifest.
- Added unit tests for desktop entry parsing, exec argument sanitation, and image format detection.

### Changed
- Updated repository license to Quick Shelf Source License and Non-Commercial Agreement with mandatory attribution, no relicensing, and commercial authorization requirements.
- Changed default shelf surface geometry to 1:1 aspect ratio.
- Made mouse drag events update the reorder target before release, matching
  terminal UI pointer behavior instead of treating every click as a drag.
- Made section order configurable while keeping application and image data in
  separate lists.
- Changed the active shelf surface from a GTK popup/widget to a terminal UI
  launched in the user's terminal emulator.
- Moved Quick Shelf configuration and theme files to
  `~/.config/quick/shelf/`.
- Migrated existing files from `~/.config/qshelf/` and
  `~/.config/quick-shelf/` when the new configuration directory is first
  initialized.
- Made popup mode a normal floating window so Hyprland rules and SUPER plus mouse movement apply.
- Updated the default layout to use the square surfaces, flat controls, and accent border used by the bitzdots Rofi themes.

### Fixed
- Fixed generated bitzdots `rgb(...)` and `rgba(...)` palette values being
  ignored by the terminal color parser.
- Fixed GTK3 and gtk-layer-shell compilation failures in GitHub Actions CI runner.
- Corrected enum variant casing to PascalCase for `Orientation`, `Align`, and `StackTransitionType`.
- Replaced deprecated file chooser constructor with `FileChooserDialog::with_buttons` and owned `FileFilter`.
- Integrated `LayerShell` trait methods for layer configuration, anchors, margins, and keyboard interactivity.
- Removed unused imports and dead code warnings across modules.

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
