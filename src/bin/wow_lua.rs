//! Lua REPL client for wow-ui-sim.
//!
//! Connects to a running wow-ui-sim instance and executes Lua code.
//!
//! Usage:
//!   wow-lua                     # Interactive REPL
//!   wow-lua -e "print('hi')"    # Execute code and exit
//!   wow-lua script.lua          # Execute file and exit
//!   wow-lua --list              # List running servers

use std::io::{self, BufRead, Write};
use std::path::PathBuf;
use wow_ui_sim::lua_server::client;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    // Parse arguments
    if args.len() > 1 {
        match args[1].as_str() {
            "--help" | "-h" => {
                print_help();
                return;
            }
            "--list" | "-l" => {
                list_servers();
                return;
            }
            "-e" => {
                if args.len() < 3 {
                    eprintln!("Error: -e requires code argument");
                    std::process::exit(1);
                }
                let code = args[2..].join(" ");
                execute_and_exit(&code);
                return;
            }
            arg if !arg.starts_with('-') => {
                // Treat as file path
                execute_file_and_exit(arg);
                return;
            }
            _ => {
                eprintln!("Unknown option: {}", args[1]);
                print_help();
                std::process::exit(1);
            }
        }
    }

    // Interactive REPL
    run_repl();
}

fn print_help() {
    println!("wow-lua - Lua REPL for wow-ui-sim");
    println!();
    println!("Usage:");
    println!("  wow-lua                     Interactive REPL");
    println!("  wow-lua -e <code>           Execute code and exit");
    println!("  wow-lua <file.lua>          Execute file and exit");
    println!("  wow-lua --list              List running servers");
    println!();
    println!("REPL Commands:");
    println!("  .exit, .quit                Exit the REPL");
    println!("  .servers                    List running servers");
    println!("  .connect <path>             Connect to specific server");
}

fn list_servers() {
    let servers = client::find_servers();
    if servers.is_empty() {
        println!("No wow-ui-sim servers found.");
        println!("Start wow-ui-sim first, then run wow-lua.");
    } else {
        println!("Running servers:");
        for server in &servers {
            let status = match client::ping(server) {
                Ok(()) => "OK",
                Err(_) => "ERROR",
            };
            println!("  {} [{}]", server.display(), status);
        }
    }
}

fn find_server() -> Option<PathBuf> {
    let servers = client::find_servers();
    if servers.is_empty() {
        eprintln!("Error: No wow-ui-sim server found.");
        eprintln!("Start wow-ui-sim first, then run wow-lua.");
        return None;
    }
    if servers.len() > 1 {
        eprintln!("Multiple servers found. Using first one.");
        eprintln!("Use --list to see all, or set WOW_LUA_SOCKET to specify.");
    }
    Some(servers.into_iter().next().unwrap())
}

fn execute_and_exit(code: &str) {
    let socket = match std::env::var("WOW_LUA_SOCKET") {
        Ok(s) => PathBuf::from(s),
        Err(_) => match find_server() {
            Some(s) => s,
            None => std::process::exit(1),
        },
    };

    match client::exec(&socket, code) {
        Ok(output) => {
            if !output.is_empty() {
                println!("{}", output);
            }
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }
}

fn execute_file_and_exit(path: &str) {
    let code = match std::fs::read_to_string(path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error reading {}: {}", path, e);
            std::process::exit(1);
        }
    };
    execute_and_exit(&code);
}

fn run_repl() {
    let mut socket = match std::env::var("WOW_LUA_SOCKET") {
        Ok(s) => PathBuf::from(s),
        Err(_) => match find_server() {
            Some(s) => s,
            None => std::process::exit(1),
        },
    };

    println!("Connected to {}", socket.display());
    println!("Type Lua code to execute. Use .exit to quit.");
    println!();

    let stdin = io::stdin();
    let mut stdout = io::stdout();

    loop {
        print!("> ");
        stdout.flush().unwrap();

        let mut line = String::new();
        match stdin.lock().read_line(&mut line) {
            Ok(0) => break, // EOF
            Ok(_) => {}
            Err(e) => {
                eprintln!("Read error: {}", e);
                break;
            }
        }

        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        // Handle REPL commands
        if line.starts_with('.') {
            match line {
                ".exit" | ".quit" | ".q" => break,
                ".servers" => {
                    list_servers();
                    continue;
                }
                cmd if cmd.starts_with(".connect ") => {
                    let path = cmd.strip_prefix(".connect ").unwrap().trim();
                    socket = PathBuf::from(path);
                    println!("Switched to {}", socket.display());
                    continue;
                }
                _ => {
                    eprintln!("Unknown command: {}", line);
                    continue;
                }
            }
        }

        // Execute Lua code
        match client::exec(&socket, line) {
            Ok(output) => {
                if !output.is_empty() {
                    println!("{}", output);
                }
            }
            Err(e) => {
                eprintln!("Error: {}", e);
            }
        }
    }

    println!("Goodbye!");
}
