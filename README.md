# Quick Shelf

Mouse-enabled terminal shelf for application shortcuts and image drop previews.

Quick Shelf is a Rofi-style terminal UI written in Rust. It provides a
keyboard-first shelf with mouse selection, click-to-open, right-click removal,
drag-to-reorder, and path or file drops from terminal paste events. The
bitzdots profile gives it the same compact, squared, Material color language
as the rest of the desktop.

## Features

- App Shortcuts: Pin applications with `--add-shortcut`, launch them with a click, and reorder them with the mouse.
- Image Shelf: Add image paths through terminal bracketed paste and open them with a click.
- Mouse Interaction: Click items to open them, right-click to remove them, scroll to select, and drag within a shelf to reorder it.
- Metadata: Image entries show dimensions and file size in the terminal list.
- Terminal UI: Runs in an alternate terminal screen with keyboard navigation and mouse capture.
- Waybar Integration: Can launch the TUI through the existing module and report dynamic item counts.
- Single Instance IPC: Fast Unix domain socket communication handles toggling, styling reloads, and adding items from scripts or keybindings.
- Dynamic Theming: Adapts to wallpaper colors using Matugen with live palette reloading.

## Quick Installation

Precompiled Linux release binaries are built automatically via GitHub Actions
so low-resource machines do not need to compile Rust locally.

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

### Terminal UI

Run Quick Shelf directly in a terminal:

```bash
quick-shelf --tui
```

Use `q` or `Esc` to quit, arrow keys or `j`/`k` to move, `Enter` to open,
`d` or right-click to remove, and left-button drag to reorder items within
the app or image shelf. Dropping a path into a bracketed-paste-capable
terminal adds a `.desktop` shortcut or supported image.

### Toggle Mode

Toggle the floating popover shelf (useful for keyboard shortcuts or Waybar clicks):

```bash
quick-shelf --toggle
```

If the TUI is already open, `--toggle` hides or shows it. If it is not
running, Quick Shelf opens it in the first available terminal from
`foot`, `kitty`, `alacritty`, or `wezterm`. Set
`QUICK_SHELF_TERMINAL="kitty"` to choose a terminal explicitly.

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
    --homescreen, --desktop   Compatibility alias for the terminal shelf
    --toggle                  Toggle the terminal shelf (or launch it)
    --popup                   Show the terminal shelf
    --tui                     Run the mouse-enabled terminal shelf directly
    --hide                    Hide the running terminal shelf
    --status                  Output Waybar-compatible JSON status
    --theme <NAME>            Set the active profile name for integrations
    --list-themes             List all available built-in and user themes
    --reload-style            Reload CSS styling in running instance
    --add-shortcut <PATH>     Add an application .desktop file or executable
    --add-image <PATH>        Add an image to the shelf
    --clear-images            Clear all images from the shelf
    --version, -v             Print version information
    --help, -h                Print help message
```

## Configuration

Quick Shelf stores its configuration and CSS in `~/.config/quick/shelf/`. Shelf
items remain in the XDG data directory at `~/.local/share/quick-shelf/`.

The shelf is a compact, keyboard-first Rofi-style terminal surface rather
than a desktop panel. The terminal captures mouse clicks and drag gestures,
and bracketed paste accepts paths from terminal file managers.

```json
{
  "theme": "bitzdots",
  "homescreen_anchor": "top-right",
  "margin_top": 40,
  "margin_right": 24,
  "margin_bottom": 24,
  "margin_left": 24,
  "width": 380,
  "max_height": 620,
  "default_view": "both",
  "layer": "bottom",
  "tui": {
    "accent": "#d46478",
    "background": "#160d11",
    "foreground": "#f8e4df",
    "muted": "#d4a8a2",
    "selected": "#35182c",
    "card": "#261420",
    "layout": {
      "width_percent": 92,
      "height_percent": 82,
      "section_order": ["apps", "images"],
      "show_details": true
    }
  }
}
```

- `theme`: Active color theme (`bitzdots`, `default`, `tokyonight`, `catppuccin`, `nord`, `dracula`, `gruvbox`, `everforest`, `rose-pine`, or the name of a custom CSS file in `~/.config/quick/shelf/themes/`). `bitzdots` is the default profile.
- `homescreen_anchor`: Screen corner anchor (`top-right`, `top-left`, `bottom-right`, `bottom-left`).
- `margin_top`: Offset from top screen edge in pixels (recommended 40px to clear Waybar).
- `margin_right`: Offset from right screen edge in pixels.
- `width`: Widget width in pixels.
- `default_view`: Default tab to show (`both`, `apps`, or `media`).
- `layer`: Layer shell level (`bottom` sits behind normal windows on the desktop).
- `tui`: Terminal UI color tokens. Every value is a six-digit hex color and
  can be changed without rebuilding Quick Shelf.
- `tui.layout`: Terminal panel geometry, copy, visibility, item order, and
  detail rendering. These values let a dotfiles profile change the interface
  without changing the shelf data or rebuilding the binary.

See [docs/ricing.md](docs/ricing.md) for the complete configuration reference
and copyable profiles.

## Theming and Styling

Quick Shelf uses a Rofi-like terminal palette. The default layout uses
square surfaces, an accent border, compact controls, and flat selection
states. The TUI colors can be edited directly in `config.json`, or generated
from the `colors.css` file produced by Matugen, Pywal, or Wallust.

### Built-in Themes

Nine profiles are included out of the box:

- `default`: Warm berry dark mode
- `bitzdots`: Default squared warm berry profile
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

### Legacy GTK Theme Files

To make a theme that matches your Rofi or Waybar config, place a `.css` file inside `~/.config/quick/shelf/themes/`. For example, create `~/.config/quick/shelf/themes/mytheme.css`:

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

The old GTK popup implementation used `style.css` and theme files. They are
kept in the repository for migration history, but are not loaded by the TUI.

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

If you use an automated wallpaper color generator, configure it to write colors to `~/.config/quick/shelf/colors.css`. Whenever this file exists, Quick Shelf automatically imports its variable definitions on top of the active theme.

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

Trigger a palette reload after updating colors:

```bash
quick-shelf --reload-style
```

### Custom Layout Overrides

The active interface is terminal-based. Rice it by editing
`~/.config/quick/shelf/config.json`, adding a profile stylesheet under
`~/.config/quick/shelf/themes/`, or generating `colors.css` from your
wallpaper tool. Layout changes apply the next time the TUI reloads.

The old GTK source and CSS files remain in the repository only as migration
history. They are not loaded by the TUI.

## Building from Source

Dependencies:
- GLib and GdkPixbuf development files
- A terminal emulator with mouse reporting and bracketed paste
- Rust stable toolchain

Build with Cargo:

```bash
cargo build --release
```

The resulting binary will be placed at `target/release/quick-shelf`.

## License

MIT License. See [LICENSE](LICENSE) for details.
