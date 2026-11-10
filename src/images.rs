use crate::config::{images_dir, images_file};
use gdk_pixbuf::Pixbuf;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
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

    pub fn move_item(&mut self, from: usize, to: usize) -> bool {
        if from >= self.images.len() || to >= self.images.len() || from == to {
            return false;
        }
        let item = self.images.remove(from);
        self.images.insert(to, item);
        let _ = self.save();
        true
    }

    pub fn clear_all(&mut self) {
        for img in &self.images {
            let _ = fs::remove_file(&img.path);
        }
        self.images.clear();
        let _ = self.save();
    }

    pub fn copy_image_to_clipboard(path: &Path) {
        let _ = Command::new("wl-copy")
            .arg("--type")
            .arg("image/png")
            .stdin(fs::File::open(path).unwrap_or_else(|_| fs::File::open("/dev/null").unwrap()))
            .spawn();
    }

    pub fn copy_path_to_clipboard(path: &Path) {
        let p_str = path.to_string_lossy().to_string();
        let _ = Command::new("wl-copy").arg(&p_str).spawn();
    }

    pub fn open_image(path: &Path) {
        let _ = open::that(path);
    }

    pub fn start_external_drag(path: &Path, is_image: bool) -> String {
        let abs_path = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
        let uri = format!("file://{}", abs_path.to_string_lossy());

        let _ = Command::new("wl-copy")
            .arg("--type")
            .arg("text/uri-list")
            .arg(&uri)
            .spawn();

        if is_image {
            Self::copy_image_to_clipboard(&abs_path);
        }

        let drag_tools = ["ripdrag", "dragon", "dragon-drop"];
        for tool in drag_tools {
            if let Ok(path_var) = std::env::var("PATH") {
                for dir in path_var.split(':') {
                    let bin = Path::new(dir).join(tool);
                    if bin.is_file() {
                        let _ = Command::new(tool)
                            .arg("-x")
                            .arg("-A")
                            .arg(&abs_path)
                            .spawn();
                        return format!("Dragging via {} (or paste with Ctrl+V)", tool);
                    }
                }
            }
        }

        "Copied to clipboard (Ctrl+V into any app, install ripdrag for direct drag)".to_string()
    }

    pub fn generate_halfblock_thumbnail(
        path: &Path,
        cols: u16,
        rows: u16,
    ) -> Vec<Vec<(char, ratatui::style::Color, ratatui::style::Color)>> {
        let px_w = cols as i32;
        let px_h = (rows * 2) as i32;
        let Ok(pixbuf) = Pixbuf::from_file_at_scale(path, px_w, px_h, false) else {
            return Vec::new();
        };
        let w = pixbuf.width() as usize;
        let h = pixbuf.height() as usize;
        let rowstride = pixbuf.rowstride() as usize;
        let n_channels = pixbuf.n_channels() as usize;
        let bytes = pixbuf.read_pixel_bytes();
        let data: &[u8] = bytes.as_ref();

        let mut result = Vec::with_capacity(rows as usize);
        for row in 0..rows as usize {
            let mut line = Vec::with_capacity(cols as usize);
            let y_top = row * 2;
            let y_bot = y_top + 1;
            for col in 0..cols as usize {
                let (r_t, g_t, b_t) = if col < w && y_top < h {
                    let offset = y_top * rowstride + col * n_channels;
                    if offset + 2 < data.len() {
                        (data[offset], data[offset + 1], data[offset + 2])
                    } else {
                        (0, 0, 0)
                    }
                } else {
                    (0, 0, 0)
                };
                let (r_b, g_b, b_b) = if col < w && y_bot < h {
                    let offset = y_bot * rowstride + col * n_channels;
                    if offset + 2 < data.len() {
                        (data[offset], data[offset + 1], data[offset + 2])
                    } else {
                        (0, 0, 0)
                    }
                } else {
                    (0, 0, 0)
                };
                line.push((
                    '▀',
                    ratatui::style::Color::Rgb(r_t, g_t, b_t),
                    ratatui::style::Color::Rgb(r_b, g_b, b_b),
                ));
            }
            result.push(line);
        }
        result
    }
}

fn sanitize_filename(name: &str) -> String {
    name.chars()
        .filter(|c| c.is_alphanumeric() || *c == '-' || *c == '_')
        .collect::<String>()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recognizes_supported_image_extensions() {
        assert!(ImageManager::is_image_path(Path::new("photo.png")));
        assert!(ImageManager::is_image_path(Path::new("photo.JPG")));
        assert!(ImageManager::is_image_path(Path::new("image.webp")));
        assert!(ImageManager::is_image_path(Path::new("vector.svg")));
        assert!(!ImageManager::is_image_path(Path::new("document.pdf")));
        assert!(!ImageManager::is_image_path(Path::new("script.sh")));
    }

    #[test]
    fn sanitizes_file_names() {
        assert_eq!(sanitize_filename("my photo (1)!.png"), "myphoto1png");
        assert_eq!(sanitize_filename("valid-name_123"), "valid-name_123");
    }
}
