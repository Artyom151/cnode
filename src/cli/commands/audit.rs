use crate::error::{CNodeError, Result};
use crate::registry::Registry;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

pub async fn handle() -> Result<()> {
    let package_json_path = Path::new("package.json");
    if !package_json_path.exists() {
        return Err(CNodeError::Custom("package.json not found".to_string()));
    }

    let content = fs::read_to_string(package_json_path)?;
    let package_json: serde_json::Value = serde_json::from_str(&content)?;

    let mut deps_to_check = HashMap::new();

    let dep_types = ["dependencies", "devDependencies"];
    for dep_type in &dep_types {
        if let Some(deps) = package_json.get(*dep_type).and_then(|d| d.as_object()) {
            for (name, version_val) in deps {
                let version = version_val.as_str().unwrap_or("latest");
                let clean_version = version.trim_start_matches('^').trim_start_matches('~').trim_start_matches('>').trim_start_matches('<').trim_start_matches('=');
                deps_to_check.insert(name.clone(), clean_version.to_string());
            }
        }
    }

    if deps_to_check.is_empty() {
        println!("No dependencies to audit");
        return Ok(());
    }

    let registry = Registry::new(None);
    let audit_result = registry.audit(&deps_to_check).await?;

    if audit_result.advisories.is_empty() {
        println!("No known vulnerabilities found");
        return Ok(());
    }

    println!("{:<10} {:<30} {}", "SEVERITY", "PACKAGE", "TITLE");
    println!("{}", "-".repeat(80));

    for (adv_id, advisory) in &audit_result.advisories {
        let sev = advisory.severity.to_uppercase();
        let pkg_name = advisory.package_name.as_deref().unwrap_or(adv_id);
        println!("{:<10} {:<30} {}", sev, pkg_name, advisory.title);
    }

    println!();
    let total = &audit_result.metadata.vulnerabilities;
    let total_count = total.info + total.low + total.moderate + total.high + total.critical;
    println!("{} vulnerabilities found", total_count);
    if total.critical > 0 { println!("  critical: {}", total.critical); }
    if total.high > 0 { println!("  high: {}", total.high); }
    if total.moderate > 0 { println!("  moderate: {}", total.moderate); }
    if total.low > 0 { println!("  low: {}", total.low); }
    if total.info > 0 { println!("  info: {}", total.info); }

    Ok(())
}
