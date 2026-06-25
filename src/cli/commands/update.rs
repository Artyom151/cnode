use crate::downloader::Downloader;
use crate::error::{CNodeError, Result};
use crate::registry::Registry;
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

fn safe_package_path(name: &str) -> PathBuf {
    if name.starts_with('@') {
        let mut parts = name.splitn(2, '/');
        let scope = parts.next().unwrap_or("@");
        let pkg = parts.next().unwrap_or("");
        Path::new("node_modules").join(scope).join(pkg)
    } else {
        Path::new("node_modules").join(name)
    }
}

pub async fn handle(
    packages: Option<Vec<String>>,
    all: bool,
    registry_url: Option<String>,
) -> Result<()> {
    let registry = Registry::new(registry_url);
    let downloader = Downloader::new(None)?;
    let package_json_path = Path::new("package.json");

    if !package_json_path.exists() {
        return Err(CNodeError::Custom("package.json not found".to_string()));
    }

    let content = fs::read_to_string(package_json_path)?;
    let mut package_json: serde_json::Value = serde_json::from_str(&content)?;

    let deps = package_json["dependencies"].as_object()
        .ok_or_else(|| CNodeError::Custom("No dependencies in package.json".to_string()))?
        .clone();

    let update_list: Vec<(String, String)> = if let Some(pkgs) = packages {
        pkgs.into_iter().filter_map(|p| {
            deps.get(&p).map(|v| (p, v.as_str().unwrap_or("latest").to_string()))
        }).collect()
    } else if all {
        deps.iter().map(|(k, v)| (k.clone(), v.as_str().unwrap_or("latest").to_string())).collect()
    } else {
        return Err(CNodeError::Custom("Specify packages or use --all".to_string()));
    };

    if update_list.is_empty() {
        return Err(CNodeError::Custom("No packages to update".to_string()));
    }

    let mut updated_deps = BTreeMap::new();

    for (name, _version_spec) in &update_list {
        let metadata = registry.fetch_package_metadata(name).await?;
        let new_version = metadata.dist_tags.latest.clone();

        let version_info = registry.fetch_version(name, &new_version).await?;

        let pkg_dir = safe_package_path(name);
        if pkg_dir.exists() {
            fs::remove_dir_all(&pkg_dir)?;
        }
        fs::create_dir_all(&pkg_dir)?;
        downloader.download_package(&version_info, &pkg_dir).await?;

        updated_deps.insert(name.clone(), new_version.clone());
    }

    if let Some(deps_obj) = package_json["dependencies"].as_object_mut() {
        for (name, version) in &updated_deps {
            deps_obj.insert(name.clone(), serde_json::Value::String(version.clone()));
        }
    }

    fs::write(package_json_path, serde_json::to_string_pretty(&package_json)?)?;

    Ok(())
}
