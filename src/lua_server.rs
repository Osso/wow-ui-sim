//! Lua execution server for wow-ui-sim.
//!
//! Provides a Unix socket server that accepts Lua code and returns results.

use serde::{Deserialize, Serialize};
use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::PathBuf;
use std::sync::mpsc;
use std::thread;

/// Request sent to the Lua server.
#[derive(Debug, Serialize, Deserialize)]
pub enum Request {
    /// Execute Lua code
    Exec { code: String },
    /// Ping to check if server is alive
    Ping,
    /// Dump the frame tree
    DumpTree {
        /// Filter by name (substring match)
        filter: Option<String>,
        /// Only show visible frames
        visible_only: bool,
    },
}

/// Response from the Lua server.
#[derive(Debug, Serialize, Deserialize)]
pub enum Response {
    /// Execution result (captured print output)
    Output(String),
    /// Error message
    Error(String),
    /// Pong response
    Pong,
    /// Frame tree dump
    Tree(String),
}

/// Command sent to the app from the Lua server.
pub enum LuaCommand {
    Exec {
        code: String,
        respond: mpsc::Sender<Response>,
    },
    DumpTree {
        filter: Option<String>,
        visible_only: bool,
        respond: mpsc::Sender<Response>,
    },
}

/// Get the socket path for Lua REPL.
pub fn socket_path() -> PathBuf {
    PathBuf::from(format!(
        "/tmp/wow-lua-{}.sock",
        std::process::id()
    ))
}

/// Initialize the Lua server.
/// Returns a receiver for commands. The socket is cleaned up on drop.
pub fn init() -> mpsc::Receiver<LuaCommand> {
    // Clean up stale sockets from dead processes
    cleanup_stale_sockets();

    let (tx, rx) = mpsc::channel();
    let path = socket_path();

    // Clean up our own stale socket if it exists
    let _ = std::fs::remove_file(&path);

    thread::spawn(move || {
        run_server(tx, path);
    });

    rx
}

/// Clean up stale sockets from dead processes.
fn cleanup_stale_sockets() {
    let pattern = "/tmp/wow-lua-*.sock";
    if let Ok(entries) = glob::glob(pattern) {
        for entry in entries.flatten() {
            // Extract PID from filename: /tmp/wow-lua-{pid}.sock
            if let Some(filename) = entry.file_name().and_then(|f| f.to_str()) {
                if let Some(pid_str) = filename
                    .strip_prefix("wow-lua-")
                    .and_then(|s| s.strip_suffix(".sock"))
                {
                    if let Ok(pid) = pid_str.parse::<i32>() {
                        // Check if process is still alive using kill(pid, 0)
                        let exists = unsafe { libc::kill(pid, 0) } == 0;
                        if !exists {
                            if std::fs::remove_file(&entry).is_ok() {
                                eprintln!("[wow-lua] Cleaned up stale socket: {}", entry.display());
                            }
                        }
                    }
                }
            }
        }
    }
}

fn run_server(cmd_tx: mpsc::Sender<LuaCommand>, path: PathBuf) {
    let listener = match UnixListener::bind(&path) {
        Ok(l) => {
            eprintln!("[wow-lua] Listening on {}", path.display());
            l
        }
        Err(e) => {
            eprintln!("[wow-lua] Failed to bind: {}", e);
            return;
        }
    };

    // Clean up socket on exit
    struct SocketGuard(PathBuf);
    impl Drop for SocketGuard {
        fn drop(&mut self) {
            let _ = std::fs::remove_file(&self.0);
        }
    }
    let _guard = SocketGuard(path);

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                if let Err(e) = handle_connection(stream, &cmd_tx) {
                    eprintln!("[wow-lua] Connection error: {}", e);
                }
            }
            Err(e) => {
                eprintln!("[wow-lua] Accept error: {}", e);
            }
        }
    }
}

/// Send a command and wait for a response with timeout.
fn send_command(
    cmd_tx: &mpsc::Sender<LuaCommand>,
    build: impl FnOnce(mpsc::Sender<Response>) -> LuaCommand,
) -> Response {
    let (resp_tx, resp_rx) = mpsc::channel();
    if cmd_tx.send(build(resp_tx)).is_err() {
        return Response::Error("App closed".into());
    }
    match resp_rx.recv_timeout(std::time::Duration::from_secs(30)) {
        Ok(r) => r,
        Err(_) => Response::Error("Timeout".into()),
    }
}

fn handle_connection(
    mut stream: UnixStream,
    cmd_tx: &mpsc::Sender<LuaCommand>,
) -> std::io::Result<()> {
    let reader = BufReader::new(stream.try_clone()?);

    for line in reader.lines() {
        let line = line?;
        if line.is_empty() {
            continue;
        }

        let request: Request = match serde_json::from_str(&line) {
            Ok(r) => r,
            Err(e) => {
                let resp = Response::Error(format!("Invalid request: {}", e));
                writeln!(stream, "{}", serde_json::to_string(&resp).unwrap())?;
                continue;
            }
        };

        let response = match request {
            Request::Ping => Response::Pong,
            Request::Exec { code } => {
                send_command(cmd_tx, |respond| LuaCommand::Exec { code, respond })
            }
            Request::DumpTree { filter, visible_only } => {
                send_command(cmd_tx, |respond| LuaCommand::DumpTree { filter, visible_only, respond })
            }
        };

        writeln!(stream, "{}", serde_json::to_string(&response).unwrap())?;
    }

    Ok(())
}

/// Client module for connecting to the Lua server.
pub mod client {
    use super::*;
    use std::io::{BufRead, BufReader, Write};
    use std::os::unix::net::UnixStream;
    use std::path::Path;

    /// Connect to a Lua server and execute code.
    pub fn exec<P: AsRef<Path>>(socket: P, code: &str) -> Result<String, String> {
        let mut stream =
            UnixStream::connect(socket).map_err(|e| format!("Connect failed: {}", e))?;

        let request = Request::Exec {
            code: code.to_string(),
        };
        writeln!(stream, "{}", serde_json::to_string(&request).unwrap())
            .map_err(|e| format!("Write failed: {}", e))?;

        let mut reader = BufReader::new(stream);
        let mut line = String::new();
        reader
            .read_line(&mut line)
            .map_err(|e| format!("Read failed: {}", e))?;

        let response: Response =
            serde_json::from_str(&line).map_err(|e| format!("Invalid response: {}", e))?;

        match response {
            Response::Output(s) => Ok(s),
            Response::Error(e) => Err(e),
            Response::Pong => Err("Unexpected pong".into()),
            Response::Tree(_) => Err("Unexpected tree".into()),
        }
    }

    /// Ping the server.
    pub fn ping<P: AsRef<Path>>(socket: P) -> Result<(), String> {
        let mut stream =
            UnixStream::connect(socket).map_err(|e| format!("Connect failed: {}", e))?;

        let request = Request::Ping;
        writeln!(stream, "{}", serde_json::to_string(&request).unwrap())
            .map_err(|e| format!("Write failed: {}", e))?;

        let mut reader = BufReader::new(stream);
        let mut line = String::new();
        reader
            .read_line(&mut line)
            .map_err(|e| format!("Read failed: {}", e))?;

        let response: Response =
            serde_json::from_str(&line).map_err(|e| format!("Invalid response: {}", e))?;

        match response {
            Response::Pong => Ok(()),
            Response::Error(e) => Err(e),
            _ => Err("Unexpected response".into()),
        }
    }

    /// Find running wow-lua servers.
    pub fn find_servers() -> Vec<PathBuf> {
        glob::glob("/tmp/wow-lua-*.sock")
            .map(|paths| paths.filter_map(Result::ok).collect())
            .unwrap_or_default()
    }

    /// Dump the frame tree.
    pub fn dump_tree<P: AsRef<Path>>(
        socket: P,
        filter: Option<String>,
        visible_only: bool,
    ) -> Result<String, String> {
        let mut stream =
            UnixStream::connect(socket).map_err(|e| format!("Connect failed: {}", e))?;

        let request = Request::DumpTree { filter, visible_only };
        writeln!(stream, "{}", serde_json::to_string(&request).unwrap())
            .map_err(|e| format!("Write failed: {}", e))?;

        let mut reader = BufReader::new(stream);
        let mut line = String::new();
        reader
            .read_line(&mut line)
            .map_err(|e| format!("Read failed: {}", e))?;

        let response: Response =
            serde_json::from_str(&line).map_err(|e| format!("Invalid response: {}", e))?;

        match response {
            Response::Tree(s) => Ok(s),
            Response::Error(e) => Err(e),
            _ => Err("Unexpected response".into()),
        }
    }
}
