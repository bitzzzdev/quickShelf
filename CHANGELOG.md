# Changelog

All notable changes to this project will be documented in this file.

The format is based on Keep a Changelog, and this project adheres to Semantic Versioning.

## [Unreleased]

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
