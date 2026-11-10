mod config;
mod images;
mod ipc;
mod shortcuts;
mod themes;
mod tui;

use images::ImageManager;
use ipc::send_command;
use shortcuts::ShortcutManager;
use std::env;
use std::path::Path;
use std::process::Command;
use tui::run_tui;

fn print_help() {
    println!(
        r#"quick-shelf v{}
Mouse-enabled terminal shelf for app shortcuts and image drop previews

USAGE:
    quick-shelf [OPTIONS]

OPTIONS:
    --homescreen, --desktop   Run as persistent homescreen widget on desktop background
    --toggle                  Toggle popup shelf visibility (or launch popup if not running)
    --popup                   Show popup shelf window
    --tui                     Run the mouse-enabled terminal shelf directly
    --hide                    Hide running popup shelf window
    --status                  Output Waybar-compatible JSON status
    --theme <NAME>            Apply theme (switches live if running, or updates config)
    --list-themes             List available built-in and user themes
    --reload-style            Reload CSS styling in running instance
    --add-shortcut <PATH>     Add an application .desktop file or executable
    --add-image <PATH>        Add an image to the shelf
    --clear-images            Clear all images from the shelf
    --version, -v             Print version information
    --help, -h                Print this help message
"#,
        env!("CARGO_PKG_VERSION")
    );
}
fn launch_tui_terminal() {
    let executable = env::current_exe().unwrap_or_else(|_| Path::new("quick-shelf").to_path_buf());
    let terminal = env::var("QUICK_SHELF_TERMINAL").ok();
    let candidates = terminal
        .as_deref()
        .map(|value| vec![value.to_string()])
        .unwrap_or_else(|| {
            ["foot", "kitty", "alacritty", "wezterm"]
                .iter()
                .map(|value| value.to_string())
                .collect()
        });

    for candidate in candidates {
        let mut parts = candidate.split_whitespace();
        let Some(program) = parts.next() else { continue };
        let mut command = Command::new(program);
        command.args(parts);
        if program == "foot" {
            command.args(["--app-id", "quick-shelf"]);
        } else if program == "kitty" {
            command.args(["--class", "quick-shelf"]);
        } else if program == "alacritty" {
            command.args(["--class", "quick-shelf", "-e"]);
        } else if program == "wezterm" {
            command.args(["start", "--class", "quick-shelf", "--"]);
        }
        if command.arg(&executable).arg("--tui").spawn().is_ok() {
            return;
        }
    }

    eprintln!(
        "No supported terminal found. Run `quick-shelf --tui` directly or set QUICK_SHELF_TERMINAL."
    );
}

fn print_status() {
    let shortcuts = ShortcutManager::load();
    let images = ImageManager::load();
    let s_count = shortcuts.count();
    let i_count = images.count();
    let total = s_count + i_count;

    let class = if total > 0 { "has-items" } else { "empty" };
    let tooltip = format!(
        "Quick Shelf\n  • Shortcuts: {}\n  • Images: {}\n\nLeft-click: Toggle shelf\nRight-click: Add shortcut",
        s_count, i_count
    );

    let json = serde_json::json!({
        "text": "󱂬",
        "alt": "shelf",
        "tooltip": tooltip,
        "class": class,
        "count": total
    });

    println!("{}", json);
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let action = args.get(1).map(|s| s.as_str()).unwrap_or("--toggle");

    match action {
        "--help" | "-h" => {
            print_help();
            return;
        }
        "--version" | "-v" => {
            println!("quick-shelf {}", env!("CARGO_PKG_VERSION"));
            return;
        }
        "--status" => {
            print_status();
            return;
        }
        "--clear-images" => {
            let mut images = ImageManager::load();
            images.clear_all();
            let _ = send_command("refresh");
            println!("Cleared all images from shelf.");
            return;
        }
        "--reload-style" => {
            if let Ok(resp) = send_command("reload-style") {
                println!("Reload style: {}", resp);
            } else {
                println!("quick-shelf is not running.");
            }
            return;
        }
        "--list-themes" => {
            let current = config::ShelfConfig::load().theme;
            println!("Available themes:");
            for t in themes::list_available_themes() {
                if t == current {
                    println!("* {} (active)", t);
                } else {
                    println!("  {}", t);
                }
            }
            return;
        }
        "--theme" => {
            if let Some(theme_name) = args.get(2) {
                if let Ok(resp) = send_command(&format!("set-theme {}", theme_name)) {
                    println!("Applied theme '{}' via IPC: {}", theme_name, resp);
                } else {
                    let mut cfg = config::ShelfConfig::load();
                    cfg.theme = theme_name.clone();
                    if let Err(e) = cfg.save() {
                        eprintln!("Failed to save theme to config: {}", e);
                    } else {
                        println!("Theme set to '{}' in config.", theme_name);
                    }
                }
            } else {
                eprintln!("Error: Missing theme name. Usage: quick-shelf --theme <NAME>");
            }
            return;
        }
        "--hide" => {
            let _ = send_command("hide");
            return;
        }
        "--add-shortcut" => {
            if let Some(target) = args.get(2) {
                let path = Path::new(target);
                if path.extension().and_then(|e| e.to_str()) == Some("desktop") {
                    if let Some(sc) = ShortcutManager::from_desktop_file(path) {
                        let mut mgr = ShortcutManager::load();
                        mgr.add(sc);
                        let _ = send_command("refresh");
                        println!("Shortcut added: {}", target);
                        return;
                    }
                }
                let mut mgr = ShortcutManager::load();
                let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or(target);
                mgr.add(shortcuts::Shortcut {
                    id: stem.to_string(),
                    name: stem.to_string(),
                    exec: target.to_string(),
                    icon: "application-x-executable".to_string(),
                });
                let _ = send_command("refresh");
                println!("Shortcut added: {}", target);
            } else {
                eprintln!("Error: Missing path for --add-shortcut");
            }
            return;
        }
        "--add-image" => {
            if let Some(target) = args.get(2) {
                let path = Path::new(target);
                let mut mgr = ImageManager::load();
                match mgr.add_from_path(path) {
                    Ok(item) => {
                        let _ = send_command("refresh");
                        println!("Image added to shelf: {}", item.name);
                    }
                    Err(e) => eprintln!("Failed to add image: {}", e),
                }
            } else {
                eprintln!("Error: Missing image path for --add-image");
            }
            return;
        }
        "--toggle" => {
            if let Ok(resp) = send_command("toggle") {
                if resp == "ok" {
                    return;
                }
            }
            launch_tui_terminal();
        }
        "--popup" => {
            if let Ok(resp) = send_command("show") {
                if resp == "ok" {
                    return;
                }
            }
            launch_tui_terminal();
        }
        "--tui" => {
            if let Err(error) = run_tui() {
                eprintln!("Quick Shelf TUI failed: {}", error);
                std::process::exit(1);
            }
        }
        "--homescreen" | "--desktop" => {
            if let Err(error) = run_tui() {
                eprintln!("Quick Shelf TUI failed: {}", error);
                std::process::exit(1);
            }
        }
        other => {
            eprintln!("Unknown option: {}", other);
            print_help();
            std::process::exit(1);
        }
    }
}
