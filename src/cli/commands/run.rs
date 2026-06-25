use crate::error::{CNodeError, Result};
use std::fs;
use std::path::Path;
use std::process::Command;

pub async fn handle(script: String, args: Vec<String>) -> Result<()> {
    let package_json_path = Path::new("package.json");

    if !package_json_path.exists() {
        return Err(CNodeError::Custom("package.json not found".to_string()));
    }

    let content = fs::read_to_string(package_json_path)?;
    let package_json: serde_json::Value = serde_json::from_str(&content)?;

    let scripts = package_json.get("scripts")
        .and_then(|s| s.as_object())
        .ok_or_else(|| CNodeError::Custom("No scripts in package.json".to_string()))?;

    let script_cmd = scripts.get(&script)
        .and_then(|s| s.as_str())
        .ok_or_else(|| CNodeError::Custom(format!("Script '{}' not found", script)))?;

    let shell = if cfg!(target_os = "windows") { "cmd" } else { "sh" };
    let shell_arg = if cfg!(target_os = "windows") { "/C" } else { "-c" };

    let full_cmd = if args.is_empty() {
        script_cmd.to_string()
    } else {
        format!("{} {}", script_cmd, args.join(" "))
    };

    let status = Command::new(shell)
        .arg(shell_arg)
        .arg(&full_cmd)
        .status()?;

    if !status.success() {
        std::process::exit(status.code().unwrap_or(1));
    }

    Ok(())
}
