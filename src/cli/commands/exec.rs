use crate::error::Result;
use std::process::Command;
use std::path::PathBuf;

pub async fn handle(command: String, args: Vec<String>) -> Result<()> {
    let bin_dir = PathBuf::from("node_modules/.bin");

    let cmd_path = if cfg!(windows) {
        let exts = [".cmd", ".ps1", ""];
        let mut found = None;
        for ext in &exts {
            let candidate = bin_dir.join(format!("{}{}", command, ext));
            if candidate.exists() {
                found = Some(candidate);
                break;
            }
        }
        found.map(|p| p.to_path_buf())
    } else {
        let candidate = bin_dir.join(&command);
        if candidate.exists() {
            Some(candidate)
        } else {
            None
        }
    };

    match cmd_path {
        Some(path) => {
            let status = if cfg!(windows) {
                Command::new("cmd")
                    .arg("/c")
                    .arg(path.to_string_lossy().as_ref())
                    .args(&args)
                    .spawn()
            } else {
                Command::new(&path)
                    .args(&args)
                    .spawn()
            };

            match status {
                Ok(mut child) => {
                    child.wait()?;
                }
                Err(e) => {
                    return Err(crate::error::CNodeError::Custom(format!(
                        "failed to execute {}: {}",
                        command, e
                    )));
                }
            }
            Ok(())
        }
        None => {
            return Err(crate::error::CNodeError::Custom(format!(
                "cannot find command `{}`. Make sure it is installed in node_modules/.bin",
                command
            )));
        }
    }
}
