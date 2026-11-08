# Quick Shelf

Quick Shelf is a lightweight Wayland desktop widget and Waybar popover written in Rust with GTK3 and gtk-layer-shell. It provides a quick-access shelf for application shortcuts and an image drop zone with instant thumbnail previews, metadata inspection, and two-way drag-and-drop.

## Features

- App Shortcuts: Pin frequently used applications, launch with a single click, or search and add installed system applications.
- Image Drop Shelf: Drop images directly from file managers (like Nautilus) or web browsers into the shelf.
- Two-way Drag and Drop: Drag images out of the shelf into external apps like Discord, GIMP, web forms, or file managers.
- Instant Preview: Click any thumbnail to open an instant preview with image dimensions, file size, copy image data, copy file path, or open in the default viewer.
- Homescreen Widget Mode: Runs on the desktop background layer behind normal windows using Wayland Layer Shell.
- Waybar Integration: Can be placed as an interactive Waybar module with dynamic item counts, status tooltips, and click-to-toggle popover.
- Single Instance IPC: Fast Unix domain socket communication handles toggling, styling reloads, and adding items from scripts or keybindings.
- Dynamic Theming: Adapts to wallpaper colors using Matugen with live CSS reloading.

## Quick Installation

Because compiling GUI Rust applications on low-resource machines is time consuming, precompiled release binaries are built automatically via GitHub Actions workflows.

### Method 1: Using the DEVos Shell Helper (in bitzdots)

If you use the bitzdots dotfiles setup, fetch the precompiled binary with:

```bash
dev shelf fetch
```

To update an existing installation:

```bash
dev shelf update
```

### Method 2: Direct Download via Curl

Download the latest release binary directly:

```bash
mkdir -p ~/.local/bin
curl -fsSL https://github.com/bitzzdev/quick-shelf/releases/download/quick-shelf-latest/quick-shelf-x86_64 -o ~/.local/bin/quick-shelf
chmod +x ~/.local/bin/quick-shelf
```

Ensure `~/.local/bin` is in your `PATH`.

## Usage

### Homescreen Widget Mode

Run Quick Shelf pinned to the desktop layer:

```bash
quick-shelf --homescreen
```

By default, it anchors to the top right of the desktop below Waybar. You can customize margins, width, and anchor in `~/.config/quick-shelf/config.json`.

### Toggle Popover Mode

Toggle the floating popover shelf (useful for keyboard shortcuts or Waybar clicks):

```bash
quick-shelf --toggle
```

If the widget is already open, `--toggle` hides it. If hidden or not running, it displays the shelf.

### Waybar Status Output

Print Waybar compatible JSON status:

```bash
quick-shelf --status
```

Example JSON output:

```json
{"text":"shelf","alt":"shelf","tooltip":"Quick Shelf\n  • Shortcuts: 4\n  • Images: 3\n\nLeft-click: Toggle shelf\nRight-click: Add shortcut","class":"has-items","count":7}
```

### Command Line Options

```text
USAGE:
    quick-shelf [OPTIONS]

OPTIONS:
    --homescreen, --desktop   Run as persistent homescreen widget on desktop background
    --toggle                  Toggle popup shelf visibility (or launch popup if not running)
    --popup                   Show popup shelf window
    --hide                    Hide running popup shelf window
    --status                  Output Waybar-compatible JSON status
    --reload-style            Reload CSS styling in running instance
    --add-shortcut <PATH>     Add an application .desktop file or executable
    --add-image <PATH>        Add an image to the shelf
    --clear-images            Clear all images from the shelf
    --version, -v             Print version information
    --help, -h                Print help message
```

## Configuration

Quick Shelf stores its configuration in `~/.config/quick-shelf/config.json`.

```json
{
  "homescreen_anchor": "top-right",
  "margin_top": 40,
  "margin_right": 24,
  "margin_bottom": 24,
  "margin_left": 24,
  "width": 380,
  "max_height": 620,
  "default_view": "both",
  "layer": "bottom"
}
```

- `homescreen_anchor`: Screen corner anchor (`top-right`, `top-left`, `bottom-right`, `bottom-left`).
- `margin_top`: Offset from top screen edge in pixels (recommended 40px to clear Waybar).
- `margin_right`: Offset from right screen edge in pixels.
- `width`: Widget width in pixels.
- `default_view`: Default tab to show (`both`, `apps`, or `media`).
- `layer`: Layer shell level (`bottom` sits behind normal windows on the desktop).

## Building from Source

Dependencies:
- GTK 3.24 development files (`libgtk-3-dev`)
- GTK Layer Shell development files (`libgtk-layer-shell-dev`)
- GLib and GdkPixbuf development files
- Rust stable toolchain (1.75+)

Build with Cargo:

```bash
cargo build --release
```

The resulting binary will be placed at `target/release/quick-shelf`.

## License

MIT License. See [LICENSE](LICENSE) for details.
