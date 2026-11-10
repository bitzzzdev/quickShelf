use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShelfConfig {
    pub homescreen_anchor: String,
    pub margin_top: i32,
    pub margin_right: i32,
    pub margin_bottom: i32,
    pub margin_left: i32,
    pub width: i32,
    pub max_height: i32,
    pub default_view: String,
    pub layer: String,
    #[serde(default = "default_theme")]
    pub theme: String,
    #[serde(default)]
    pub tui: TuiStyle,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TuiStyle {
    #[serde(default = "default_accent")]
    pub accent: String,
    #[serde(default = "default_background")]
    pub background: String,
    #[serde(default = "default_foreground")]
    pub foreground: String,
    #[serde(default = "default_muted")]
    pub muted: String,
    #[serde(default = "default_selected")]
    pub selected: String,
    #[serde(default = "default_card")]
    pub card: String,
    #[serde(default)]
    pub layout: TuiLayout,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TuiLayout {
    #[serde(default = "default_width_percent")]
    pub width_percent: u16,
    #[serde(default = "default_height_percent")]
    pub height_percent: u16,
    #[serde(default = "default_header_height")]
    pub header_height: u16,
    #[serde(default = "default_footer_height")]
    pub footer_height: u16,
    #[serde(default = "default_content_padding")]
    pub content_padding: u16,
    #[serde(default = "default_detail_width")]
    pub detail_width: u16,
    #[serde(default = "default_title")]
    pub title: String,
    #[serde(default = "default_subtitle")]
    pub subtitle: String,
    #[serde(default = "default_footer_hint")]
    pub footer_hint: String,
    #[serde(default = "default_section_order")]
    pub section_order: Vec<String>,
    #[serde(default = "default_show_header")]
    pub show_header: bool,
    #[serde(default = "default_show_footer")]
    pub show_footer: bool,
    #[serde(default = "default_show_details")]
    pub show_details: bool,
}

fn default_accent() -> String { "#d46478".to_string() }
fn default_background() -> String { "#160d11".to_string() }
fn default_foreground() -> String { "#f8e4df".to_string() }
fn default_muted() -> String { "#d4a8a2".to_string() }
fn default_selected() -> String { "#35182c".to_string() }
fn default_card() -> String { "#261420".to_string() }
fn default_width_percent() -> u16 { 92 }
fn default_height_percent() -> u16 { 82 }
fn default_header_height() -> u16 { 3 }
fn default_footer_height() -> u16 { 2 }
fn default_content_padding() -> u16 { 1 }
fn default_detail_width() -> u16 { 42 }
fn default_title() -> String { "QUICK SHELF".to_string() }
fn default_subtitle() -> String {
    "Keyboard-first, mouse-enabled, and drop-friendly".to_string()
}
fn default_footer_hint() -> String {
    "q quit   enter open   d delete   drag reorder   paste path".to_string()
}
fn default_section_order() -> Vec<String> {
    vec!["apps".to_string(), "images".to_string()]
}
fn default_show_header() -> bool { true }
fn default_show_footer() -> bool { true }
fn default_show_details() -> bool { true }

impl Default for TuiStyle {
    fn default() -> Self {
        Self {
            accent: default_accent(),
            background: default_background(),
            foreground: default_foreground(),
            muted: default_muted(),
            selected: default_selected(),
            card: default_card(),
            layout: TuiLayout::default(),
        }
    }
}

impl Default for TuiLayout {
    fn default() -> Self {
        Self {
            width_percent: default_width_percent(),
            height_percent: default_height_percent(),
            header_height: default_header_height(),
            footer_height: default_footer_height(),
            content_padding: default_content_padding(),
            detail_width: default_detail_width(),
            title: default_title(),
            subtitle: default_subtitle(),
            footer_hint: default_footer_hint(),
            section_order: default_section_order(),
            show_header: default_show_header(),
            show_footer: default_show_footer(),
            show_details: default_show_details(),
        }
    }
}

fn default_theme() -> String {
    "bitzdots".to_string()
}

impl Default for ShelfConfig {
    fn default() -> Self {
        Self {
            homescreen_anchor: "top-right".to_string(),
            margin_top: 40,
            margin_right: 24,
            margin_bottom: 24,
            margin_left: 24,
            width: 380,
            max_height: 620,
            default_view: "both".to_string(),
            layer: "bottom".to_string(),
            theme: default_theme(),
            tui: TuiStyle::default(),
        }
    }
}

pub fn config_dir() -> PathBuf {
    let p = glib::user_config_dir().join("quick").join("shelf");
    let _ = fs::create_dir_all(&p);
    migrate_legacy_config(&p);
    p
}

fn migrate_legacy_config(target: &PathBuf) {
    if target.join("config.json").exists() {
        return;
    }

    let config_root = glib::user_config_dir();
    for legacy in [config_root.join("qshelf"), config_root.join("quick-shelf")] {
        if !legacy.is_dir() {
            continue;
        }

        if let Ok(entries) = fs::read_dir(&legacy) {
            for entry in entries.flatten() {
                let source = entry.path();
                let destination = target.join(entry.file_name());
                if source.is_file() && !destination.exists() {
                    if let Err(error) = fs::copy(&source, &destination) {
                        eprintln!(
                            "Failed to migrate Quick Shelf config {}: {}",
                            source.display(),
                            error
                        );
                    }
                }
            }
        }

        if target.join("config.json").exists() {
            break;
        }
    }
}

pub fn data_dir() -> PathBuf {
    let p = glib::user_data_dir().join("quick-shelf");
    let _ = fs::create_dir_all(&p);
    p
}

pub fn images_dir() -> PathBuf {
    let p = data_dir().join("images");
    let _ = fs::create_dir_all(&p);
    p
}

pub fn shortcuts_file() -> PathBuf {
    data_dir().join("shortcuts.json")
}

pub fn images_file() -> PathBuf {
    data_dir().join("images.json")
}

pub fn config_file() -> PathBuf {
    config_dir().join("config.json")
}

pub fn style_file() -> PathBuf {
    config_dir().join("style.css")
}

pub fn colors_file() -> PathBuf {
    config_dir().join("colors.css")
}

pub fn themes_dir() -> PathBuf {
    let p = config_dir().join("themes");
    let _ = fs::create_dir_all(&p);
    p
}

pub fn ipc_socket_path() -> PathBuf {
    if let Ok(runtime_dir) = std::env::var("XDG_RUNTIME_DIR") {
        PathBuf::from(runtime_dir).join("quick-shelf.sock")
    } else {
        let user = glib::user_name().to_string_lossy().to_string();
        PathBuf::from(format!("/tmp/quick-shelf-{}.sock", user))
    }
}

impl ShelfConfig {
    pub fn load() -> Self {
        let path = config_file();
        if path.exists() {
            if let Ok(content) = fs::read_to_string(&path) {
                if let Ok(cfg) = serde_json::from_str::<ShelfConfig>(&content) {
                    return cfg.with_generated_palette();
                }
            }
        }
        let cfg = ShelfConfig::default();
        let _ = cfg.save();
        cfg.with_generated_palette()
    }

    fn with_generated_palette(mut self) -> Self {
        let palette_path = colors_file();
        let Ok(content) = fs::read_to_string(palette_path) else {
            return self;
        };

        for line in content.lines() {
            let Some((name, value)) = line
                .trim()
                .strip_prefix("@define-color ")
                .and_then(|line| line.split_once(' '))
            else {
                continue;
            };
            let value = value.trim().trim_end_matches(';');
            match name {
                "bg" => self.tui.background = value.to_string(),
                "fg" => self.tui.foreground = value.to_string(),
                "fg-muted" => self.tui.muted = value.to_string(),
                "selected" => self.tui.selected = value.to_string(),
                "border-color" => self.tui.accent = value.to_string(),
                "card-bg" => self.tui.card = value.to_string(),
                _ => {}
            }
        }
        self
    }

    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let path = config_file();
        let content = serde_json::to_string_pretty(self)?;
        fs::write(path, content)?;
        Ok(())
    }
}
