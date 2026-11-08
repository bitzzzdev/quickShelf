use crate::config::{images_dir, images_file};
use gdk_pixbuf::Pixbuf;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ImageItem {
    pub id: String,
    pub name: String,
    pub path: String,
    pub added_at: u64,
    pub width: i32,
    pub height: i32,
    pub size_bytes: u64,
}

#[derive(Debug, Clone, Default)]
pub struct ImageManager {
    images: Vec<ImageItem>,
}

impl ImageManager {
    pub fn load() -> Self {
        let path = images_file();
        if path.exists() {
            if let Ok(content) = fs::read_to_string(&path) {
                if let Ok(mut list) = serde_json::from_str::<Vec<ImageItem>>(&content) {
                    list.retain(|img| Path::new(&img.path).exists());
                    return Self { images: list };
                }
            }
        }
        let mgr = Self { images: Vec::new() };
        let _ = mgr.save();
        mgr
    }

    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let path = images_file();
        let json = serde_json::to_string_pretty(&self.images)?;
        fs::write(path, json)?;
        Ok(())
    }

    pub fn list(&self) -> &[ImageItem] {
        &self.images
    }

    pub fn count(&self) -> usize {
        self.images.len()
    }

    pub fn is_image_path(path: &Path) -> bool {
        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            let lower = ext.to_lowercase();
            matches!(
                lower.as_str(),
                "png" | "jpg" | "jpeg" | "webp" | "gif" | "svg" | "bmp" | "avif" | "ico"
            )
        } else {
            false
        }
    }

    pub fn add_from_path(&mut self, source_path: &Path) -> Result<ImageItem, Box<dyn std::error::Error>> {
        if !source_path.exists() {
            return Err(format!("File does not exist: {}", source_path.display()).into());
        }

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let filename = source_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("image.png");

        let ext = source_path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("png");

        let dest_filename = format!("{}_{}.{}", now, sanitize_filename(filename), ext);
        let dest_path = images_dir().join(&dest_filename);

        fs::copy(source_path, &dest_path)?;

        let size_bytes = fs::metadata(&dest_path).map(|m| m.len()).unwrap_or(0);
        let (width, height) = match Pixbuf::file_info(&dest_path) {
            Some((info, w, h)) => {
                let _ = info;
                (w, h)
            }
            None => (0, 0),
        };

        let item = ImageItem {
            id: format!("img_{}", now),
            name: filename.to_string(),
            path: dest_path.to_string_lossy().to_string(),
            added_at: now,
            width,
            height,
            size_bytes,
        };

        self.images.insert(0, item.clone());
        let _ = self.save();
        Ok(item)
    }

    pub fn remove(&mut self, id: &str) -> bool {
        if let Some(pos) = self.images.iter().position(|img| img.id == id) {
            let removed = self.images.remove(pos);
            let _ = fs::remove_file(&removed.path);
            let _ = self.save();
            return true;
        }
        false
    }

    pub fn clear_all(&mut self) {
        for img in &self.images {
            let _ = fs::remove_file(&img.path);
        }
        self.images.clear();
        let _ = self.save();
    }

    pub fn copy_image_to_clipboard(path: &Path) {
        if let Ok(pixbuf) = Pixbuf::from_file(path) {
            let clipboard = gtk::Clipboard::get(&gdk::SELECTION_CLIPBOARD);
            clipboard.set_image(&pixbuf);
        }
        let _ = Command::new("wl-copy")
            .arg("--type")
            .arg("image/png")
            .stdin(fs::File::open(path).unwrap_or_else(|_| fs::File::open("/dev/null").unwrap()))
            .spawn();
    }

    pub fn copy_path_to_clipboard(path: &Path) {
        let p_str = path.to_string_lossy().to_string();
        let clipboard = gtk::Clipboard::get(&gdk::SELECTION_CLIPBOARD);
        clipboard.set_text(&p_str);
        let _ = Command::new("wl-copy").arg(&p_str).spawn();
    }

    pub fn open_image(path: &Path) {
        let _ = open::that(path);
    }
}

fn sanitize_filename(name: &str) -> String {
    name.chars()
        .filter(|c| c.is_alphanumeric() || *c == '-' || *c == '_')
        .collect::<String>()
}
