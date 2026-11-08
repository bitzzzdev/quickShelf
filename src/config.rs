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
        }
    }
}

pub fn config_dir() -> PathBuf {
    let p = glib::user_config_dir().join("quick-shelf");
    let _ = fs::create_dir_all(&p);
    p
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
                    return cfg;
                }
            }
        }
        let cfg = ShelfConfig::default();
        let _ = cfg.save();
        cfg
    }

    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let path = config_file();
        let content = serde_json::to_string_pretty(self)?;
        fs::write(path, content)?;
        Ok(())
    }
}
