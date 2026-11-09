# Quick Shelf

Desktop shelf widget and Waybar popover for application shortcuts and image drop previews.

Quick Shelf is a Wayland desktop widget and Waybar popover written in Rust with GTK3 and gtk-layer-shell. It provides a quick-access shelf for application shortcuts and an image drop zone with instant thumbnail previews, metadata inspection, and two-way drag and drop.

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
curl -fsSL https://github.com/bitzzdev/quickShelf/releases/download/quick-shelf-latest/quick-shelf-x86_64 -o ~/.local/bin/quick-shelf
chmod +x ~/.local/bin/quick-shelf
```

Ensure `~/.local/bin` is in your `PATH`.

## Usage

### Homescreen Widget Mode

Run Quick Shelf pinned to the desktop layer:

```bash
quick-shelf --homescreen
```

By default, it anchors to the top right of the desktop below Waybar. You can customize margins, width, and anchor in `~/.config/qshelf/config.json`.

### Toggle Popover Mode

Toggle the floating popover shelf (useful for keyboard shortcuts or Waybar clicks):

```bash
quick-shelf --toggle
```

If the widget is already open, `--toggle` hides it. If hidden or not running, it displays the shelf.

Popup mode is a regular floating window with the `quick-shelf` Hyprland class.
Use the compositor's normal `SUPER + left mouse` binding to move it and
Hyprland window rules to control its placement.

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
    --theme <NAME>            Apply a theme by name (built-in or from ~/.config/qshelf/themes/)
    --list-themes             List all available built-in and user themes
    --reload-style            Reload CSS styling in running instance
    --add-shortcut <PATH>     Add an application .desktop file or executable
    --add-image <PATH>        Add an image to the shelf
    --clear-images            Clear all images from the shelf
    --version, -v             Print version information
    --help, -h                Print help message
```

## Configuration

Quick Shelf stores its configuration and CSS in `~/.config/qshelf/`. Shelf
items remain in the XDG data directory at `~/.local/share/quick-shelf/`.

```json
{
  "theme": "default",
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

- `theme`: Active color theme (`default`, `tokyonight`, `catppuccin`, `nord`, `dracula`, `gruvbox`, `everforest`, `rose-pine`, or the name of a custom CSS file in `~/.config/qshelf/themes/`).
- `homescreen_anchor`: Screen corner anchor (`top-right`, `top-left`, `bottom-right`, `bottom-left`).
- `margin_top`: Offset from top screen edge in pixels (recommended 40px to clear Waybar).
- `margin_right`: Offset from right screen edge in pixels.
- `width`: Widget width in pixels.
- `default_view`: Default tab to show (`both`, `apps`, or `media`).
- `layer`: Layer shell level (`bottom` sits behind normal windows on the desktop).

## Theming and Styling

Quick Shelf uses a Rofi-like theming approach. The default layout uses square surfaces, a two-pixel accent border, compact controls, and flat selection states. Colors are decoupled from layout styles through GTK CSS variables (`@define-color`). You can swap built-in themes, create your own palette file to match your dotfiles, or hook into dynamic wallpaper generators like Matugen, Pywal, or Wallust.

### Built-in Themes

Eight themes are included out of the box:

- `default`: Warm berry dark mode
- `tokyonight`: Classic Tokyo Night dark blue palette
- `catppuccin`: Catppuccin Mocha style pastel accents
- `nord`: Arctic North dark slate and frost blue
- `dracula`: High contrast purple and pink vampire palette
- `gruvbox`: Retro warm groove tones
- `everforest`: Muted natural green forest aesthetic
- `rose-pine`: Soft pine, rose, and iris palette

List all themes available on your system:

```bash
quick-shelf --list-themes
```

Switch the theme on a running instance without restarting:

```bash
quick-shelf --theme tokyonight
```

### Custom Themes

To make a theme that matches your Rofi or Waybar config, place a `.css` file inside `~/.config/qshelf/themes/`. For example, create `~/.config/qshelf/themes/mytheme.css`:

```css
/* Quick Shelf theme variables matching Rofi conventions */
@define-color bg rgba(20, 20, 28, 0.94);
@define-color bg-alt rgba(30, 30, 42, 0.75);
@define-color fg #e0e2ea;
@define-color fg-muted #8a90a4;
@define-color selected #7289da;
@define-color active #43b581;
@define-color urgent #f04747;
@define-color border-color rgba(70, 75, 95, 0.55);
@define-color card-bg rgba(30, 30, 42, 0.65);
@define-color card-hover rgba(70, 75, 95, 0.85);
```

The built-in layout is loaded before `~/.config/qshelf/style.css`, so the
last file can override geometry and typography as well as colors:

```css
.quick-shelf-window {
    border-width: 3px;
}

.qs-app-card {
    min-width: 76px;
    min-height: 76px;
}
```

Then select it via command line or config:

```bash
quick-shelf --theme mytheme
```

### Automatic Wallpaper Theming (Matugen, Pywal, Wallust)

If you use an automated wallpaper color generator, configure it to write colors to `~/.config/qshelf/colors.css`. Whenever this file exists, Quick Shelf automatically imports its variable definitions on top of the active theme.

Example template for Matugen or Pywal:

```css
@define-color bg {{colors.surface.default.hex}};
@define-color bg-alt {{colors.surface_container.default.hex}};
@define-color fg {{colors.on_surface.default.hex}};
@define-color fg-muted {{colors.on_surface_variant.default.hex}};
@define-color selected {{colors.primary.default.hex}};
@define-color active {{colors.secondary.default.hex}};
@define-color urgent {{colors.error.default.hex}};
@define-color border-color {{colors.outline.default.hex}};
@define-color card-bg {{colors.surface_container_low.default.hex}};
@define-color card-hover {{colors.surface_container_high.default.hex}};
```

Trigger a style reload after updating colors:

```bash
quick-shelf --reload-style
```

### Custom Layout Overrides

If you want to alter borders, paddings, fonts, or animations beyond color tokens, add custom CSS rules to `~/.config/qshelf/style.css`. Rules defined here load last and override default stylesheet properties.

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
