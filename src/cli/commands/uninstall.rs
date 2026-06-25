use crate::error::Result;
use std::fs;
use std::path::Path;

pub async fn handle(packages: Vec<String>, save: bool) -> Result<()> {
    if packages.is_empty() {
        return Err(crate::error::CNodeError::Custom("No packages specified".to_string()));
    }

    let package_json_path = Path::new("package.json");
    let mut package_json: serde_json::Value = if package_json_path.exists() {
        let content = fs::read_to_string(package_json_path)?;
        serde_json::from_str(&content)?
    } else {
        return Err(crate::error::CNodeError::Custom("package.json not found".to_string()));
    };

    let nm_path = Path::new("node_modules");

    for pkg in &packages {
        let pkg_name = if let Some(idx) = pkg.find('@') {
            if idx == 0 { pkg.as_str() } else { &pkg[..idx] }
        } else {
            pkg.as_str()
        };

        let pkg_dir = if pkg_name.starts_with('@') {
            let mut parts = pkg_name.splitn(2, '/');
            let scope = parts.next().unwrap();
            let name = parts.next().unwrap_or("");
            nm_path.join(scope).join(name)
        } else {
            nm_path.join(pkg_name)
        };

        if pkg_dir.exists() {
            fs::remove_dir_all(&pkg_dir)?;
        }

        if save {
            if let Some(deps) = package_json["dependencies"].as_object_mut() {
                deps.remove(pkg_name);
            }
            if let Some(dev_deps) = package_json["devDependencies"].as_object_mut() {
                dev_deps.remove(pkg_name);
            }
        }
    }

    if save {
        fs::write(package_json_path, serde_json::to_string_pretty(&package_json)?)?;
    }

    Ok(())
}
