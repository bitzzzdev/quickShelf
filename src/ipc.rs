use crate::config::ipc_socket_path;
use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::sync::mpsc::Sender;
use std::thread;

#[derive(Debug, Clone)]
pub enum IpcMessage {
    Toggle,
    Show,
    Hide,
    ReloadStyle,
    Refresh,
    AddImage(String),
    AddShortcut(String),
}

pub fn send_command(cmd: &str) -> Result<String, std::io::Error> {
    let socket_path = ipc_socket_path();
    let mut stream = UnixStream::connect(socket_path)?;
    writeln!(stream, "{}", cmd)?;
    stream.flush()?;

    let mut reader = BufReader::new(stream);
    let mut response = String::new();
    reader.read_line(&mut response)?;
    Ok(response.trim().to_string())
}

#[allow(dead_code)]
pub fn is_running() -> bool {
    let socket_path = ipc_socket_path();
    if !socket_path.exists() {
        return false;
    }
    match UnixStream::connect(socket_path) {
        Ok(mut stream) => {
            let _ = writeln!(stream, "ping");
            let _ = stream.flush();
            true
        }
        Err(_) => {
            let _ = fs::remove_file(ipc_socket_path());
            false
        }
    }
}

pub fn start_server(sender: Sender<IpcMessage>) -> Result<(), std::io::Error> {
    let socket_path = ipc_socket_path();
    if socket_path.exists() {
        let _ = fs::remove_file(&socket_path);
    }

    let listener = UnixListener::bind(&socket_path)?;

    thread::spawn(move || {
        for stream in listener.incoming() {
            if let Ok(mut stream) = stream {
                let mut reader = BufReader::new(stream.try_clone().unwrap());
                let mut line = String::new();
                if reader.read_line(&mut line).is_ok() {
                    let trimmed = line.trim();
                    let response = match trimmed {
                        "ping" => "pong\n".to_string(),
                        "toggle" => {
                            let _ = sender.send(IpcMessage::Toggle);
                            "ok\n".to_string()
                        }
                        "show" => {
                            let _ = sender.send(IpcMessage::Show);
                            "ok\n".to_string()
                        }
                        "hide" => {
                            let _ = sender.send(IpcMessage::Hide);
                            "ok\n".to_string()
                        }
                        "reload-style" => {
                            let _ = sender.send(IpcMessage::ReloadStyle);
                            "ok\n".to_string()
                        }
                        "refresh" => {
                            let _ = sender.send(IpcMessage::Refresh);
                            "ok\n".to_string()
                        }
                        s if s.starts_with("add-image ") => {
                            let path = s.trim_start_matches("add-image ").trim().to_string();
                            let _ = sender.send(IpcMessage::AddImage(path));
                            "ok\n".to_string()
                        }
                        s if s.starts_with("add-shortcut ") => {
                            let path = s.trim_start_matches("add-shortcut ").trim().to_string();
                            let _ = sender.send(IpcMessage::AddShortcut(path));
                            "ok\n".to_string()
                        }
                        _ => "unknown\n".to_string(),
                    };
                    let _ = stream.write_all(response.as_bytes());
                }
            }
        }
    });

    Ok(())
}
