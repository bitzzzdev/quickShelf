mod config;
mod images;
mod ipc;
mod shortcuts;
mod ui;

use config::ShelfConfig;
use images::ImageManager;
use ipc::{IpcMessage, send_command, start_server};
use shortcuts::ShortcutManager;
use std::env;
use std::path::{Path, PathBuf};
use std::sync::mpsc::channel;
use std::time::Duration;
use ui::MainWindow;

fn print_help() {
    println!(
        r#"quick-shelf v{}
Lightweight desktop shelf widget & Waybar module for app shortcuts and image drop previews

USAGE:
    quick-shelf [OPTIONS]

OPTIONS:
    --homescreen, --desktop   Run as persistent homescreen widget on desktop background
    --toggle                  Toggle popup shelf visibility (or launch popup if not running)
    --popup                   Show popup shelf window
    --hide                    Hide running popup shelf window
    --status                  Output Waybar-compatible JSON status
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
            // Not running: launch popup GUI
            run_gui(false);
        }
        "--popup" => {
            if let Ok(resp) = send_command("show") {
                if resp == "ok" {
                    return;
                }
            }
            run_gui(false);
        }
        "--homescreen" | "--desktop" => {
            run_gui(true);
        }
        other => {
            eprintln!("Unknown option: {}", other);
            print_help();
            std::process::exit(1);
        }
    }
}

fn run_gui(is_homescreen: bool) {
    if gtk::init().is_err() {
        eprintln!("Failed to initialize GTK.");
        std::process::exit(1);
    }

    let (sender, receiver) = channel::<IpcMessage>();
    if let Err(e) = start_server(sender) {
        eprintln!("Warning: Failed to start IPC server: {}", e);
    }

    let main_window = MainWindow::new(is_homescreen);
    main_window.show();

    // Poll IPC messages in GTK main thread
    let win_weak = main_window.window().downgrade();
    glib::timeout_add_local(Duration::from_millis(50), move || {
        while let Ok(msg) = receiver.try_recv() {
            match msg {
                IpcMessage::Toggle => {
                    if let Some(w) = win_weak.upgrade() {
                        if w.is_visible() {
                            w.hide();
                        } else {
                            w.show_all();
                            w.present();
                        }
                    }
                }
                IpcMessage::Show => {
                    if let Some(w) = win_weak.upgrade() {
                        w.show_all();
                        w.present();
                    }
                }
                IpcMessage::Hide => {
                    if let Some(w) = win_weak.upgrade() {
                        w.hide();
                    }
                }
                IpcMessage::ReloadStyle => {
                    main_window.reload_style();
                }
                IpcMessage::Refresh => {
                    main_window.refresh();
                }
                IpcMessage::AddImage(path) => {
                    let mut mgr = ImageManager::load();
                    let _ = mgr.add_from_path(Path::new(&path));
                    main_window.refresh();
                }
                IpcMessage::AddShortcut(path) => {
                    let p = Path::new(&path);
                    if let Some(sc) = ShortcutManager::from_desktop_file(p) {
                        let mut mgr = ShortcutManager::load();
                        mgr.add(sc);
                        main_window.refresh();
                    }
                }
            }
        }
        glib::ControlFlow::Continue
    });

    gtk::main();
}
