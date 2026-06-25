use std::process::{Command, Stdio};

fn get_node_version() -> String {
    Command::new("node")
        .arg("--version")
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "not found".to_string())
}

fn get_npm_version() -> String {
    Command::new("npm")
        .arg("--version")
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "not found".to_string())
}

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() > 1 && args[1] == "--version" {
        println!("cnode 0.1.0");
        return;
    }

    if args.len() > 1 && args[1] == "info" {
        println!("cnode 0.1.0 - Rust FFI bindings for Node.js");
        println!("node: {}", get_node_version());
        println!("npm:  {}", get_npm_version());
        println!();
        println!("usage: cnpm <command>  - Node.js package manager");
        println!("       cnode info      - system information");
        println!("       cnode --version - version info");
        return;
    }

    println!("cnode 0.1.0 - Rust FFI bindings for Node.js");
    println!("node: {}", get_node_version());
    println!("npm:  {}", get_npm_version());
    println!();
    println!("Use 'cnpm' for package management.");
}
