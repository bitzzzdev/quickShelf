use crate::config::{colors_file, style_file, themes_dir};
use std::fs;
use std::path::Path;

pub const DEFAULT_PALETTE: &str = r#"
@define-color bg rgba(22, 13, 17, 0.92);
@define-color bg-alt rgba(38, 20, 32, 0.6);
@define-color fg #f8e4df;
@define-color fg-muted #d4a8a2;
@define-color selected #d46478;
@define-color active rgba(53, 24, 44, 0.5);
@define-color urgent #ff5555;
@define-color border-color rgba(110, 68, 80, 0.45);
@define-color card-bg rgba(38, 20, 32, 0.6);
@define-color card-hover rgba(53, 24, 44, 0.85);
"#;

pub const TOKYONIGHT_PALETTE: &str = r#"
@define-color bg rgba(21, 22, 30, 0.94);
@define-color bg-alt rgba(26, 27, 38, 0.75);
@define-color fg #c0caf5;
@define-color fg-muted #7aa2f7;
@define-color selected #7aa2f7;
@define-color active #414868;
@define-color urgent #f7768e;
@define-color border-color rgba(65, 72, 104, 0.55);
@define-color card-bg rgba(26, 27, 38, 0.65);
@define-color card-hover rgba(65, 72, 104, 0.85);
"#;

pub const CATPPUCCIN_PALETTE: &str = r#"
@define-color bg rgba(30, 29, 47, 0.94);
@define-color bg-alt rgba(40, 40, 57, 0.75);
@define-color fg #d9e0ee;
@define-color fg-muted #cdd6f4;
@define-color selected #cba6f7;
@define-color active #abe9b3;
@define-color urgent #f28fad;
@define-color border-color rgba(88, 91, 112, 0.55);
@define-color card-bg rgba(40, 40, 57, 0.65);
@define-color card-hover rgba(69, 71, 90, 0.85);
"#;

pub const NORD_PALETTE: &str = r#"
@define-color bg rgba(46, 52, 64, 0.94);
@define-color bg-alt rgba(56, 62, 74, 0.75);
@define-color fg #eceff4;
@define-color fg-muted #d8dee9;
@define-color selected #88c0d0;
@define-color active #a3be8c;
@define-color urgent #bf616a;
@define-color border-color rgba(76, 86, 106, 0.55);
@define-color card-bg rgba(56, 62, 74, 0.65);
@define-color card-hover rgba(76, 86, 106, 0.85);
"#;

pub const DRACULA_PALETTE: &str = r#"
@define-color bg rgba(30, 31, 41, 0.94);
@define-color bg-alt rgba(40, 42, 54, 0.75);
@define-color fg #f8f8f2;
@define-color fg-muted #6272a4;
@define-color selected #bd93f9;
@define-color active #50fa7b;
@define-color urgent #ff5555;
@define-color border-color rgba(98, 114, 164, 0.55);
@define-color card-bg rgba(40, 42, 54, 0.65);
@define-color card-hover rgba(68, 71, 90, 0.85);
"#;

pub const GRUVBOX_PALETTE: &str = r#"
@define-color bg rgba(40, 40, 40, 0.94);
@define-color bg-alt rgba(53, 53, 53, 0.75);
@define-color fg #ebdbb2;
@define-color fg-muted #a89984;
@define-color selected #d79921;
@define-color active #b8bb26;
@define-color urgent #fb4934;
@define-color border-color rgba(102, 92, 84, 0.55);
@define-color card-bg rgba(53, 53, 53, 0.65);
@define-color card-hover rgba(80, 73, 69, 0.85);
"#;

pub const EVERFOREST_PALETTE: &str = r#"
@define-color bg rgba(50, 61, 67, 0.94);
@define-color bg-alt rgba(60, 71, 77, 0.75);
@define-color fg #dad1be;
@define-color fg-muted #a7c080;
@define-color selected #7fbbb3;
@define-color active #a7c080;
@define-color urgent #e67e80;
@define-color border-color rgba(79, 90, 96, 0.55);
@define-color card-bg rgba(60, 71, 77, 0.65);
@define-color card-hover rgba(79, 90, 96, 0.85);
"#;

pub const ROSE_PINE_PALETTE: &str = r#"
@define-color bg rgba(25, 23, 36, 0.94);
@define-color bg-alt rgba(31, 29, 46, 0.75);
@define-color fg #e0def4;
@define-color fg-muted #908caa;
@define-color selected #eb6f92;
@define-color active #c4a7e7;
@define-color urgent #eb6f92;
@define-color border-color rgba(110, 106, 134, 0.55);
@define-color card-bg rgba(31, 29, 46, 0.65);
@define-color card-hover rgba(64, 61, 82, 0.85);
"#;

pub const BASE_RULES_CSS: &str = r#"
.quick-shelf-window {
    background-color: @bg;
    border: 1px solid @border-color;
    border-radius: 18px;
    padding: 14px;
    box-shadow: 0 10px 30px rgba(0, 0, 0, 0.45);
}

.qs-header {
    margin-bottom: 10px;
    padding-bottom: 8px;
    border-bottom: 1px solid rgba(255, 255, 255, 0.08);
}

.qs-title {
    font-size: 14px;
    font-weight: bold;
    color: @fg;
}

.qs-badge {
    background: @active;
    color: @selected;
    border-radius: 10px;
    padding: 2px 8px;
    font-size: 11px;
    font-weight: 600;
}

.qs-btn {
    background: @active;
    color: @fg;
    border: 1px solid @border-color;
    border-radius: 8px;
    padding: 4px 8px;
}

.qs-btn:hover {
    background: @selected;
    color: @bg;
}

.qs-section-title {
    font-size: 12px;
    font-weight: bold;
    color: @fg-muted;
    margin-top: 6px;
    margin-bottom: 6px;
}

.qs-app-card {
    background: @card-bg;
    border: 1px solid @border-color;
    border-radius: 12px;
    padding: 8px 6px;
    min-width: 68px;
    min-height: 68px;
    transition: all 150ms ease;
}

.qs-app-card:hover {
    background: @card-hover;
    border-color: @selected;
}

.qs-app-label {
    font-size: 11px;
    color: @fg;
    margin-top: 4px;
}

.qs-image-card {
    background: @card-bg;
    border: 1px solid @border-color;
    border-radius: 10px;
    padding: 4px;
    transition: all 150ms ease;
}

.qs-image-card:hover {
    border-color: @selected;
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.25);
}

.qs-dropzone {
    border: 2px dashed @border-color;
    border-radius: 12px;
    padding: 18px 12px;
    background: @card-bg;
}

.qs-dropzone:hover {
    border-color: @selected;
    background: @active;
}

.qs-preview-overlay {
    background: @bg;
    border-radius: 16px;
    padding: 12px;
    border: 1px solid @border-color;
}

.qs-tab-active {
    background: @selected;
    color: @bg;
    font-weight: bold;
}
"#;

pub fn builtin_theme_palette(name: &str) -> Option<&'static str> {
    match name.to_lowercase().as_str() {
        "default" => Some(DEFAULT_PALETTE),
        "tokyonight" | "tokyo-night" => Some(TOKYONIGHT_PALETTE),
        "catppuccin" | "catppuccin-mocha" => Some(CATPPUCCIN_PALETTE),
        "nord" => Some(NORD_PALETTE),
        "dracula" => Some(DRACULA_PALETTE),
        "gruvbox" => Some(GRUVBOX_PALETTE),
        "everforest" => Some(EVERFOREST_PALETTE),
        "rose-pine" | "rosepine" => Some(ROSE_PINE_PALETTE),
        _ => None,
    }
}

pub fn list_available_themes() -> Vec<String> {
    let mut names = vec![
        "default".to_string(),
        "tokyonight".to_string(),
        "catppuccin".to_string(),
        "nord".to_string(),
        "dracula".to_string(),
        "gruvbox".to_string(),
        "everforest".to_string(),
        "rose-pine".to_string(),
    ];

    let t_dir = themes_dir();
    if let Ok(entries) = fs::read_dir(t_dir) {
        for entry in entries.flatten() {
            let p = entry.path();
            if p.extension().and_then(|e| e.to_str()) == Some("css") {
                if let Some(stem) = p.file_stem().and_then(|s| s.to_str()) {
                    let s_str = stem.to_string();
                    if !names.contains(&s_str) {
                        names.push(s_str);
                    }
                }
            }
        }
    }

    names
}

pub fn load_effective_css(theme_name: &str) -> String {
    let mut css = String::new();

    // 1. Initial palette definition
    let user_theme_file = themes_dir().join(format!("{}.css", theme_name));
    if user_theme_file.is_file() {
        if let Ok(content) = fs::read_to_string(&user_theme_file) {
            css.push_str(&content);
            css.push('\n');
        }
    } else if let Some(builtin) = builtin_theme_palette(theme_name) {
        css.push_str(builtin);
        css.push('\n');
    } else {
        css.push_str(DEFAULT_PALETTE);
        css.push('\n');
    }

    // 2. Optional dotfile colors override (e.g. from Matugen / Pywal)
    let c_file = colors_file();
    if c_file.is_file() {
        if let Ok(content) = fs::read_to_string(c_file) {
            css.push_str(&content);
            css.push('\n');
        }
    }

    // 3. Base widget structure rules using the palette variables
    css.push_str(BASE_RULES_CSS);
    css.push('\n');

    // 4. Custom user style overrides
    let s_file = style_file();
    if s_file.is_file() {
        if let Ok(content) = fs::read_to_string(s_file) {
            css.push_str(&content);
            css.push('\n');
        }
    }

    css
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_builtin_themes() {
        assert!(builtin_theme_palette("default").is_some());
        assert!(builtin_theme_palette("tokyonight").is_some());
        assert!(builtin_theme_palette("catppuccin").is_some());
        assert!(builtin_theme_palette("nord").is_some());
        assert!(builtin_theme_palette("dracula").is_some());
        assert!(builtin_theme_palette("gruvbox").is_some());
        assert!(builtin_theme_palette("everforest").is_some());
        assert!(builtin_theme_palette("rose-pine").is_some());
        assert!(builtin_theme_palette("unknown-theme").is_none());
    }

    #[test]
    fn generates_complete_css_with_theme_variables() {
        let css = load_effective_css("tokyonight");
        assert!(css.contains("@define-color bg"));
        assert!(css.contains(".quick-shelf-window"));
        assert!(css.contains(".qs-title"));
        assert!(css.contains(".qs-app-card"));
    }

    #[test]
    fn lists_default_and_builtin_theme_names() {
        let list = list_available_themes();
        assert!(list.contains(&"default".to_string()));
        assert!(list.contains(&"tokyonight".to_string()));
        assert!(list.contains(&"catppuccin".to_string()));
    }
}
