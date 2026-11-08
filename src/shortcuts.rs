use crate::config::shortcuts_file;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Shortcut {
    pub id: String,
    pub name: String,
    pub exec: String,
    pub icon: String,
}

#[derive(Debug, Clone, Default)]
pub struct ShortcutManager {
    shortcuts: Vec<Shortcut>,
}

impl ShortcutManager {
    pub fn load() -> Self {
        let path = shortcuts_file();
        if path.exists() {
            if let Ok(content) = fs::read_to_string(&path) {
                if let Ok(list) = serde_json::from_str::<Vec<Shortcut>>(&content) {
                    return Self { shortcuts: list };
                }
            }
        }
        let defaults = Self::default_shortcuts();
        let mgr = Self {
            shortcuts: defaults,
        };
        let _ = mgr.save();
        mgr
    }

    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let path = shortcuts_file();
        let json = serde_json::to_string_pretty(&self.shortcuts)?;
        fs::write(path, json)?;
        Ok(())
    }

    pub fn list(&self) -> &[Shortcut] {
        &self.shortcuts
    }

    pub fn count(&self) -> usize {
        self.shortcuts.len()
    }

    pub fn add(&mut self, shortcut: Shortcut) -> bool {
        if self.shortcuts.iter().any(|s| s.exec == shortcut.exec || s.id == shortcut.id) {
            return false;
        }
        self.shortcuts.push(shortcut);
        let _ = self.save();
        true
    }

    pub fn remove(&mut self, id: &str) -> bool {
        let len = self.shortcuts.len();
        self.shortcuts.retain(|s| s.id != id);
        let removed = self.shortcuts.len() < len;
        if removed {
            let _ = self.save();
        }
        removed
    }

    #[allow(dead_code)]
    pub fn launch(&self, id: &str) -> Result<(), std::io::Error> {
        if let Some(s) = self.shortcuts.iter().find(|s| s.id == id) {
            Self::launch_exec(&s.exec)?;
        }
        Ok(())
    }

    pub fn launch_exec(exec: &str) -> Result<(), std::io::Error> {
        let cleaned = clean_exec(exec);
        Command::new("sh")
            .arg("-c")
            .arg(format!("{} &", cleaned))
            .spawn()?;
        Ok(())
    }

    pub fn default_shortcuts() -> Vec<Shortcut> {
        let mut list = Vec::new();
        let candidates = [
            ("kitty", "Kitty Terminal", "kitty", "kitty"),
            ("brave", "Brave Browser", "brave-origin", "brave-browser"),
            ("chromium", "Chromium", "chromium --force-dark-mode", "chromium"),
            ("files", "Files", "nautilus", "org.gnome.Nautilus"),
            ("code", "VS Code", "code", "com.visualstudio.code"),
            ("music", "Spotify", "spotify", "spotify-client"),
        ];

        for (id, name, exec, icon) in candidates {
            let bin = exec.split_whitespace().next().unwrap_or(exec);
            if is_command_available(bin) {
                list.push(Shortcut {
                    id: id.to_string(),
                    name: name.to_string(),
                    exec: exec.to_string(),
                    icon: icon.to_string(),
                });
            }
        }

        if list.is_empty() {
            list.push(Shortcut {
                id: "terminal".to_string(),
                name: "Terminal".to_string(),
                exec: "kitty".to_string(),
                icon: "utilities-terminal".to_string(),
            });
            list.push(Shortcut {
                id: "files".to_string(),
                name: "Files".to_string(),
                exec: "nautilus".to_string(),
                icon: "system-file-manager".to_string(),
            });
        }

        list
    }

    pub fn from_desktop_file(path: &Path) -> Option<Shortcut> {
        let content = fs::read_to_string(path).ok()?;
        let mut in_desktop_entry = false;
        let mut name = None;
        let mut exec = None;
        let mut icon = None;
        let mut no_display = false;

        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with('[') && trimmed.ends_with(']') {
                in_desktop_entry = trimmed == "[Desktop Entry]";
                continue;
            }
            if !in_desktop_entry {
                continue;
            }

            if let Some((k, v)) = trimmed.split_once('=') {
                let key = k.trim();
                let val = v.trim();
                match key {
                    "Name" if name.is_none() => name = Some(val.to_string()),
                    "Exec" if exec.is_none() => exec = Some(clean_exec(val)),
                    "Icon" if icon.is_none() => icon = Some(val.to_string()),
                    "NoDisplay" => no_display = val.eq_ignore_ascii_case("true"),
                    _ => {}
                }
            }
        }

        if no_display {
            return None;
        }

        let name = name?;
        let exec = exec?;
        let icon = icon.unwrap_or_else(|| "application-x-executable".to_string());
        let stem = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("app")
            .to_string();

        Some(Shortcut {
            id: stem,
            name,
            exec,
            icon,
        })
    }

    pub fn list_system_apps() -> Vec<Shortcut> {
        let mut apps = Vec::new();
        let mut dirs = vec![PathBuf::from("/usr/share/applications")];
        if let Ok(data_dir) = std::env::var("XDG_DATA_HOME") {
            dirs.push(PathBuf::from(data_dir).join("applications"));
        } else if let Ok(home) = std::env::var("HOME") {
            dirs.push(PathBuf::from(home).join(".local/share/applications"));
        }

        for dir in dirs {
            if let Ok(entries) = fs::read_dir(dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.extension().and_then(|e| e.to_str()) == Some("desktop") {
                        if let Some(app) = Self::from_desktop_file(&path) {
                            if !apps.iter().any(|a: &Shortcut| a.name == app.name || a.exec == app.exec) {
                                apps.push(app);
                            }
                        }
                    }
                }
            }
        }

        apps.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
        apps
    }
}

fn clean_exec(raw: &str) -> String {
    let mut parts = Vec::new();
    for part in raw.split_whitespace() {
        if part.starts_with('%') {
            continue;
        }
        parts.push(part);
    }
    parts.join(" ")
}

fn is_command_available(cmd: &str) -> bool {
    if let Ok(path) = std::env::var("PATH") {
        for dir in path.split(':') {
            let bin = Path::new(dir).join(cmd);
            if bin.is_file() {
                return true;
            }
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cleans_field_codes_from_exec_command() {
        assert_eq!(clean_exec("firefox %u"), "firefox");
        assert_eq!(clean_exec("gimp-2.10 %U %f"), "gimp-2.10");
        assert_eq!(clean_exec("code --new-window %F"), "code --new-window");
    }

    #[test]
    fn parses_desktop_entry_fields() {
        let content = "[Desktop Entry]\nName=Test App\nExec=test-bin %u\nIcon=test-icon\nType=Application\n";
        let temp_dir = std::env::temp_dir();
        let path = temp_dir.join("test_app.desktop");
        fs::write(&path, content).unwrap();

        let shortcut = ShortcutManager::from_desktop_file(&path);
        let _ = fs::remove_file(&path);

        assert!(shortcut.is_some());
        let s = shortcut.unwrap();
        assert_eq!(s.name, "Test App");
        assert_eq!(s.exec, "test-bin");
        assert_eq!(s.icon, "test-icon");
    }

    #[test]
    fn skips_nodisplay_desktop_entries() {
        let content = "[Desktop Entry]\nName=Hidden App\nExec=hidden\nIcon=hidden\nNoDisplay=true\n";
        let temp_dir = std::env::temp_dir();
        let path = temp_dir.join("hidden_app.desktop");
        fs::write(&path, content).unwrap();

        let shortcut = ShortcutManager::from_desktop_file(&path);
        let _ = fs::remove_file(&path);

        assert!(shortcut.is_none());
    }
}
