use crate::error::{CNodeError, Result};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

pub async fn handle(depth: Option<usize>, global: bool) -> Result<()> {
    let max_depth = depth.unwrap_or(usize::MAX);
    let base_path = if global {
        return Err(CNodeError::Custom("Global mode not supported yet".to_string()));
    } else {
        Path::new("node_modules").to_path_buf()
    };

    if !base_path.exists() {
        println!("(empty)");
        return Ok(());
    }

    let packages = scan_node_modules(&base_path, 0, max_depth);

    if packages.is_empty() {
        println!("(empty)");
        return Ok(());
    }

    for (i, (name, version, indent)) in packages.iter().enumerate() {
        let padding = "  ".repeat(*indent);
        let is_last = i == packages.len() - 1
            || packages.get(i + 1).map(|(_, _, d)| *d <= *indent).unwrap_or(true);
        let prefix = if is_last { "\\-- " } else { "|-- " };
        let v = version.as_deref().unwrap_or("?");
        println!("{}{}{}@{}", padding, prefix, name, v);
    }

    Ok(())
}

fn scan_node_modules(
    dir: &Path,
    current_depth: usize,
    max_depth: usize,
) -> Vec<(String, Option<String>, usize)> {
    let mut result = Vec::new();
    if current_depth > max_depth {
        return result;
    }
    if !dir.exists() {
        return result;
    }

    let entries = match fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return result,
    };

    let mut dirs: BTreeMap<String, (Option<String>, Option<PathBuf>)> = BTreeMap::new();

    for entry in entries.flatten() {
        let path = entry.path();
        let name = entry.file_name();
        let name_str = name.to_string_lossy().to_string();

        if name_str.starts_with('.') || !entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
            continue;
        }

        let pkg_json = path.join("package.json");
        let version = if pkg_json.exists() {
            if let Ok(content) = fs::read_to_string(&pkg_json) {
                if let Ok(val) = serde_json::from_str::<serde_json::Value>(&content) {
                    val.get("version").and_then(|v| v.as_str()).map(|s| s.to_string())
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        };

        dirs.insert(name_str, (version, Some(path)));
    }

    for (name, (version, path_opt)) in &dirs {
        result.push((name.clone(), version.clone(), current_depth));
        if let Some(path) = path_opt {
            let nested = path.join("node_modules");
            if nested.exists() {
                let sub = scan_node_modules(&nested, current_depth + 1, max_depth);
                result.extend(sub);
            }
        }
    }

    result
}
