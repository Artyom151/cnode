use crate::downloader::Downloader;
use crate::error::{CNodeError, Result};
use crate::package::PackageVersion;
use crate::registry::Registry;
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

fn parse_dependency(package: &str) -> (String, String) {
    if let Some(index) = package.rfind('@') {
        if index == 0 {
            return (package.to_string(), "latest".to_string());
        }
        let name = package[..index].to_string();
        let version = package[index + 1..].to_string();
        return (name, if version.is_empty() { "latest".to_string() } else { version });
    }
    (package.to_string(), "latest".to_string())
}

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

fn ensure_object(val: &mut serde_json::Value, key: &str) {
    if !val.get(key).map(|v| v.is_object()).unwrap_or(false) {
        val[key] = serde_json::json!({});
    }
}

async fn resolve_all_deps(
    registry: &Registry,
    name: &str,
    spec: &str,
    resolved: &mut BTreeMap<String, String>,
    seen: &mut std::collections::HashSet<String>,
) -> Result<()> {
    let key = format!("{}@{}", name, spec);
    if seen.contains(&key) {
        return Ok(());
    }
    seen.insert(key);

    let metadata = registry.fetch_package_metadata(name).await?;
    let version = if spec == "latest" {
        metadata.dist_tags.latest.clone()
    } else {
        registry.resolve_version_spec(&metadata, spec)?
    };

    if resolved.contains_key(name) {
        if resolved.get(name) != Some(&version) {
            let existing = resolved.get(name).cloned().unwrap_or_default();
            if version != existing {
            }
        }
        return Ok(());
    }

    resolved.insert(name.to_string(), version.clone());

    if let Some(version_info) = metadata.versions.get(&version) {
        if let Some(deps) = &version_info.dependencies {
            for (dep_name, dep_spec) in deps {
                Box::pin(resolve_all_deps(registry, dep_name, dep_spec, resolved, seen)).await?;
            }
        }
    }

    Ok(())
}

pub async fn handle(
    packages: Vec<String>,
    save: bool,
    registry_url: Option<String>,
    cache_dir: Option<PathBuf>,
) -> Result<()> {
    let registry = Registry::new(registry_url);
    let downloader = Downloader::new(cache_dir)?;
    let package_json_path = PathBuf::from("package.json");

    let mut package_json = if package_json_path.exists() {
        let content = fs::read_to_string(&package_json_path)?;
        serde_json::from_str(&content)?
    } else {
        serde_json::json!({
            "name": "project",
            "version": "0.1.0",
            "dependencies": {},
            "devDependencies": {}
        })
    };

    ensure_object(&mut package_json, "dependencies");
    ensure_object(&mut package_json, "devDependencies");

    let dependencies = package_json["dependencies"].as_object_mut().ok_or_else(|| {
        CNodeError::Custom("package.json dependencies is not an object".to_string())
    })?;

    let install_list: Vec<(String, String)> = if packages.is_empty() {
        if dependencies.is_empty() {
            return Err(CNodeError::Custom(
                "No packages specified and package.json has no dependencies".to_string(),
            ));
        }
        dependencies
            .iter()
            .map(|(name, version)| (name.clone(), version.as_str().unwrap_or("latest").to_string()))
            .collect()
    } else {
        packages
            .into_iter()
            .map(|pkg| parse_dependency(&pkg))
            .collect()
    };

    let mut flattened = BTreeMap::new();
    let mut seen = std::collections::HashSet::new();

    for (name, version_spec) in &install_list {
        resolve_all_deps(&registry, name, version_spec, &mut flattened, &mut seen).await?;
    }

    let mut version_info_map: BTreeMap<String, PackageVersion> = BTreeMap::new();
    for (name, version) in &flattened {
        let version_info = registry.fetch_version(name, version).await?;
        version_info_map.insert(name.clone(), version_info);
    }

    for (name, version_spec) in &install_list {
        if let Some(version) = flattened.get(name) {
            if let Some(version_info) = version_info_map.get(name) {
                let display = if version_spec == "latest" {
                    format!("{}@latest", name)
                } else {
                    format!("{}@{}", name, version_spec)
                };
                println!("Installing {}", display);
                let pkg_dir = safe_package_path(name);
                fs::create_dir_all(&pkg_dir)?;
                downloader.download_package(version_info, &pkg_dir).await?;
                println!("installed {}@{}", name, version);
            }
        }
    }

    for (name, version_info) in &version_info_map {
        if install_list.iter().any(|(n, _)| n == name) {
            continue;
        }
        let pkg_dir = safe_package_path(name);
        fs::create_dir_all(&pkg_dir)?;
        downloader.download_package(version_info, &pkg_dir).await?;
    }

    if save {
        for (name, version_spec) in &install_list {
            let resolved_version = flattened.get(name).cloned().unwrap_or_else(|| version_spec.clone());
            dependencies.insert(name.clone(), serde_json::Value::String(resolved_version));
        }
        fs::write(&package_json_path, serde_json::to_string_pretty(&package_json)?)?;
    }

    let package_lock = build_package_lock(&package_json, &flattened, &version_info_map, &downloader);
    fs::write("package-lock.json", serde_json::to_string_pretty(&package_lock)?)?;

    Ok(())
}

fn build_package_lock(
    package_json: &serde_json::Value,
    resolved: &BTreeMap<String, String>,
    metadata: &BTreeMap<String, PackageVersion>,
    downloader: &Downloader,
) -> serde_json::Value {
    let mut packages_map = serde_json::Map::new();
    let mut dependencies_map = serde_json::Map::new();

    for (name, version) in resolved.iter() {
        let mut package_entry = serde_json::Map::new();
        package_entry.insert("version".to_string(), serde_json::Value::String(version.clone()));

        let resolved_url = metadata
            .get(name)
            .map(|m| m.dist.tarball.clone())
            .unwrap_or_else(|| format!(
                "https://registry.npmjs.org/{}/-/{}-{}.tgz",
                name,
                name.replace('/', "-"),
                version
            ));

        package_entry.insert("resolved".to_string(), serde_json::Value::String(resolved_url));

        let integrity = downloader
            .compute_tarball_integrity(name, version)
            .unwrap_or(None)
            .unwrap_or_else(|| "sha512-unknown".to_string());

        package_entry.insert("integrity".to_string(), serde_json::Value::String(integrity));

        packages_map.insert(format!("node_modules/{}", name), serde_json::Value::Object(package_entry));

        let mut dep_entry = serde_json::Map::new();
        dep_entry.insert("version".to_string(), serde_json::Value::String(version.clone()));
        dependencies_map.insert(name.clone(), serde_json::Value::Object(dep_entry));
    }

    serde_json::json!({
        "name": package_json["name"].as_str().unwrap_or("project"),
        "version": package_json["version"].as_str().unwrap_or("0.1.0"),
        "lockfileVersion": 2,
        "requires": true,
        "packages": serde_json::Value::Object(packages_map),
        "dependencies": serde_json::Value::Object(dependencies_map),
    })
}
