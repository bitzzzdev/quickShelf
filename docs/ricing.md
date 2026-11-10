# Quick Shelf ricing

Quick Shelf keeps runtime data separate from appearance. Shortcuts and image
metadata stay in `~/.local/share/quick-shelf/`. Ricing files live in
`~/.config/quick/shelf/`, so a dotfiles repository can own the shelf profile
without copying a user's saved items.

## Files

| File | Purpose |
|---|---|
| `config.json` | Active profile, colors, layout, and section order |
| `colors.css` | Optional generated palette, normally written by Matugen |
| `themes/<name>.css` | Optional palette source for a named profile |
| `style.css` | Legacy GTK compatibility file, ignored by the TUI |

Quick Shelf loads `config.json`, then overlays matching colors from
`colors.css`. This makes a generated wallpaper palette win over the static
profile colors while leaving layout and copy under your control.

## Minimal profile

Create `~/.config/quick/shelf/config.json`:

```json
{
  "theme": "my-shelf",
  "tui": {
    "accent": "#e9a1b5",
    "background": "#171018",
    "foreground": "#f8e9ee",
    "muted": "#c7aeb8",
    "selected": "#4b2038",
    "card": "#281824",
    "layout": {
      "width_percent": 88,
      "height_percent": 78,
      "header_height": 3,
      "footer_height": 2,
      "content_padding": 1,
      "detail_width": 36,
      "title": "MY SHELF",
      "subtitle": "click to open, drag to reorder",
      "footer_hint": "q quit   enter open   d delete   drag reorder",
      "section_order": ["apps", "images"],
      "show_header": true,
      "show_footer": true,
      "show_details": true
    }
  }
}
```

Unknown keys are ignored by the JSON loader. Missing keys use the bitzdots
defaults, which makes it safe to keep a small profile and override only the
parts you want to change.

## Layout controls

`width_percent` and `height_percent` control the centered panel size inside
the terminal. `header_height`, `footer_height`, and `content_padding` control
the panel geometry. `show_header` and `show_footer` let a compact profile
remove either region. `show_details` hides command lines and image dimensions
when you want a tighter list.

`section_order` accepts `apps` or `shortcuts`, and `images` or `media`. For
example, put images before applications:

```json
{
  "tui": {
    "layout": {
      "section_order": ["images", "apps"],
      "show_details": false
    }
  }
}
```

The order changes the visual shelf and mouse hit targets. Dragging remains
limited to one data type, so an application cannot accidentally be inserted
into the image list.

## Bitzdots integration

When bitzdots is installed, Matugen writes the current wallpaper palette to:

```text
~/.config/quick/shelf/colors.css
```

The generated file uses `@define-color` values. Quick Shelf accepts both
hexadecimal values and `rgb(...)` or `rgba(...)` values, ignoring alpha when
rendering terminal cells. The next `quick-shelf --reload-style` or a fresh
TUI session loads the new palette.

No separate Quick Shelf palette is required when using bitzdots. The shelf
uses the same generated colors as the Rofi, Waybar, Kitty, Hyprland, and
SwayNC profiles.

## Other dotfiles

If you do not use bitzdots, generate the same file from your own wallpaper
tool:

```css
@define-color bg #171018;
@define-color fg #f8e9ee;
@define-color fg-muted #c7aeb8;
@define-color selected #4b2038;
@define-color border-color #e9a1b5;
@define-color card-bg #281824;
```

Save it as `~/.config/quick/shelf/colors.css`, then reload:

```bash
quick-shelf --reload-style
```

For static themes, put a CSS file in
`~/.config/quick/shelf/themes/<name>.css` and select it with:

```bash
quick-shelf --theme <name>
```

The CSS variables are a compatibility format. The active TUI reads the
following names: `bg`, `fg`, `fg-muted`, `selected`, `border-color`, and
`card-bg`.

## Dotfiles repository example

A dotfiles repository can keep the profile as a tracked file:

```text
dotfiles/
└── quick/
    └── shelf/
        ├── config.json
        ├── colors.css
        └── themes/
            └── monochrome.css
```

Link only the appearance files:

```bash
mkdir -p ~/.config/quick/shelf/themes
ln -sfn "$HOME/dotfiles/quick/shelf/config.json" \
  ~/.config/quick/shelf/config.json
ln -sfn "$HOME/dotfiles/quick/shelf/themes/monochrome.css" \
  ~/.config/quick/shelf/themes/monochrome.css
```

Do not link `~/.local/share/quick-shelf/` unless you intentionally want to
share saved shortcuts and images between machines.

## Mouse behavior

Quick Shelf enables terminal mouse reporting like OpenCode's TUI. A left
button press selects an item, drag events update the target, and release
reorders within the same shelf. Releasing without moving opens the item.
Right-click removes the selected item. Wheel events move the selection.

This requires a terminal with mouse reporting. Bracketed paste is used for
file paths because a terminal cannot receive a desktop drag payload directly.
Pasting a `.desktop` path adds an application, while pasting an image path
adds an image entry.
