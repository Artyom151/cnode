use crate::error::{CNodeError, Result};
use crate::package::{AuditResponse, PackageMetadata, PackageVersion, SearchPackage, SearchResult};
use reqwest::Client;
use std::collections::HashMap;

pub struct Registry {
    client: Client,
    registry_url: String,
    timeout_secs: u64,
}

impl Registry {
    pub fn new(registry_url: Option<String>) -> Self {
        let url = registry_url.unwrap_or_else(|| "https://registry.npmjs.org".to_string());
        Registry {
            client: Client::new(),
            registry_url: url,
            timeout_secs: 30,
        }
    }

    pub fn registry_url(&self) -> &str {
        &self.registry_url
    }

    pub async fn fetch_package_metadata(&self, package_name: &str) -> Result<PackageMetadata> {
        let url = format!("{}/{}", self.registry_url, package_name);
        let response = self.client
            .get(&url)
            .timeout(std::time::Duration::from_secs(self.timeout_secs))
            .send()
            .await?;
        if !response.status().is_success() {
            return Err(CNodeError::PackageNotFound(package_name.to_string()));
        }
        let metadata: PackageMetadata = response.json().await?;
        Ok(metadata)
    }

    pub async fn fetch_version(
        &self,
        package_name: &str,
        version: &str,
    ) -> Result<PackageVersion> {
        let url = format!("{}/{}/{}", self.registry_url, package_name, version);
        let response = self.client
            .get(&url)
            .timeout(std::time::Duration::from_secs(self.timeout_secs))
            .send()
            .await?;
        if !response.status().is_success() {
            return Err(CNodeError::InvalidVersion(format!(
                "{}@{}",
                package_name, version
            )));
        }
        let version_info: PackageVersion = response.json().await?;
        Ok(version_info)
    }

    pub async fn search(&self, query: &str, limit: usize) -> Result<Vec<SearchPackage>> {
        let url = format!(
            "{}/-/v1/search?text={}&size={}",
            self.registry_url, query, limit
        );
        let response = self.client
            .get(&url)
            .timeout(std::time::Duration::from_secs(self.timeout_secs))
            .send()
            .await?;
        if !response.status().is_success() {
            return Err(CNodeError::Custom("Search failed".to_string()));
        }
        let result: SearchResult = response.json().await?;
        Ok(result.objects.into_iter().map(|o| o.package).collect())
    }

    pub async fn audit(
        &self,
        packages: &HashMap<String, String>,
    ) -> Result<AuditResponse> {
        let url = format!("{}/-/npm/v1/security/advisories/bulk", self.registry_url);
        let mut body = HashMap::new();
        for (name, version) in packages {
            body.insert(format!("{}@{}", name, version), serde_json::json!({
                "name": name,
                "version": version,
            }));
        }
        let response = self.client
            .post(&url)
            .json(&body)
            .timeout(std::time::Duration::from_secs(self.timeout_secs))
            .send()
            .await?;
        if response.status().is_success() {
            let advisories: AuditResponse = response.json().await?;
            Ok(advisories)
        } else {
            let _status = response.status();
            Err(CNodeError::Custom(format!("Audit request failed ({})", _status)))
        }
    }

    pub fn resolve_version_spec(&self, metadata: &PackageMetadata, spec: &str) -> Result<String> {
        if spec == "*" || spec == "latest" {
            return Ok(metadata.dist_tags.latest.clone());
        }
        if metadata.versions.contains_key(spec) {
            return Ok(spec.to_string());
        }
        if spec.starts_with('^') {
            let version = &spec[1..];
            return self.find_compatible_version(&metadata.versions, version, true);
        }
        if spec.starts_with('~') {
            let version = &spec[1..];
            return self.find_compatible_version(&metadata.versions, version, false);
        }
        if spec.starts_with(">=") {
            let version = &spec[2..];
            return self.find_minimum_version(&metadata.versions, version);
        }
        if let Some(version) = metadata.versions.keys().find(|v| v.as_str() == spec) {
            return Ok(version.clone());
        }
        if spec.chars().all(|c| c.is_ascii_digit() || c == '.' || c == 'x' || c == 'X') {
            let parts: Vec<&str> = spec.split('.').filter(|p| *p != "x" && *p != "X").collect();
            if !parts.is_empty() && parts.len() < 3 {
                let major = parts[0];
                let minor = parts.get(1).map(|s| *s).unwrap_or("0");
                let base = format!("{}.{}.0", major, minor);
                let allow_minor = parts.len() == 1;
                return self.find_compatible_version(&metadata.versions, &base, allow_minor);
            }
        }
        Err(CNodeError::InvalidVersion(format!(
            "Cannot resolve version {} for {}",
            spec, metadata.name
        )))
    }

    fn find_compatible_version(
        &self,
        versions: &HashMap<String, PackageVersion>,
        base: &str,
        allow_minor: bool,
    ) -> Result<String> {
        let mut compatible = Vec::new();
        for (v, _) in versions.iter() {
            if self.is_compatible(v, base, allow_minor) {
                compatible.push(v.clone());
            }
        }
        compatible.sort();
        compatible
            .last()
            .cloned()
            .ok_or_else(|| CNodeError::InvalidVersion(base.to_string()))
    }

    fn find_minimum_version(
        &self,
        versions: &HashMap<String, PackageVersion>,
        base: &str,
    ) -> Result<String> {
        let mut matching = Vec::new();
        for (v, _) in versions.iter() {
            if self.compare_versions(v, base) >= 0 {
                matching.push(v.clone());
            }
        }
        matching.sort();
        matching
            .first()
            .cloned()
            .ok_or_else(|| CNodeError::InvalidVersion(base.to_string()))
    }

    fn is_compatible(&self, version: &str, base: &str, allow_minor: bool) -> bool {
        let v_parts: Vec<&str> = version.split('.').collect();
        let b_parts: Vec<&str> = base.split('.').collect();
        if v_parts.is_empty() || b_parts.is_empty() {
            return false;
        }
        let v_major = v_parts[0].parse::<u32>().unwrap_or(0);
        let b_major = b_parts[0].parse::<u32>().unwrap_or(0);
        if v_major != b_major {
            return false;
        }
        if allow_minor {
            return true;
        }
        let v_minor = v_parts.get(1).and_then(|v| v.parse::<u32>().ok()).unwrap_or(0);
        let b_minor = b_parts.get(1).and_then(|v| v.parse::<u32>().ok()).unwrap_or(0);
        v_minor == b_minor
    }

    fn compare_versions(&self, a: &str, b: &str) -> i32 {
        let a_parts: Vec<u32> = a.split('.').filter_map(|p| p.parse().ok()).collect();
        let b_parts: Vec<u32> = b.split('.').filter_map(|p| p.parse().ok()).collect();
        for i in 0..std::cmp::max(a_parts.len(), b_parts.len()) {
            let a_v = a_parts.get(i).copied().unwrap_or(0);
            let b_v = b_parts.get(i).copied().unwrap_or(0);
            if a_v > b_v {
                return 1;
            } else if a_v < b_v {
                return -1;
            }
        }
        0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::package::DistTags;
    use std::collections::HashMap;

    fn make_test_metadata() -> PackageMetadata {
        let mut versions = HashMap::new();
        for v in &["1.0.0", "1.1.0", "1.2.0", "2.0.0", "2.1.0", "3.0.0"] {
            versions.insert(
                v.to_string(),
                PackageVersion {
                    version: v.to_string(),
                    name: "test-pkg".to_string(),
                    description: None,
                    dependencies: None,
                    dist: crate::package::DistInfo {
                        tarball: "".to_string(),
                        shasum: "".to_string(),
                        integrity: None,
                    },
                },
            );
        }
        PackageMetadata {
            name: "test-pkg".to_string(),
            versions,
            dist_tags: DistTags {
                latest: "3.0.0".to_string(),
            },
            time: HashMap::new(),
        }
    }

    #[test]
    fn test_compare_versions_equal() {
        let r = Registry::new(None);
        assert_eq!(r.compare_versions("1.0.0", "1.0.0"), 0);
    }

    #[test]
    fn test_compare_versions_greater() {
        let r = Registry::new(None);
        assert_eq!(r.compare_versions("2.0.0", "1.0.0"), 1);
    }

    #[test]
    fn test_compare_versions_less() {
        let r = Registry::new(None);
        assert_eq!(r.compare_versions("1.0.0", "2.0.0"), -1);
    }

    #[test]
    fn test_compare_versions_patch() {
        let r = Registry::new(None);
        assert_eq!(r.compare_versions("1.0.1", "1.0.0"), 1);
        assert_eq!(r.compare_versions("1.0.0", "1.0.1"), -1);
    }

    #[test]
    fn test_compare_versions_different_length() {
        let r = Registry::new(None);
        assert_eq!(r.compare_versions("1.0", "1.0.0"), 0);
        assert_eq!(r.compare_versions("2.0", "1.0.0"), 1);
    }

    #[test]
    fn test_is_compatible_caret() {
        let r = Registry::new(None);
        assert!(r.is_compatible("1.5.0", "1.0.0", true));
        assert!(!r.is_compatible("2.0.0", "1.0.0", true));
        assert!(!r.is_compatible("1.5.0", "2.0.0", true));
    }

    #[test]
    fn test_is_compatible_tilde() {
        let r = Registry::new(None);
        assert!(r.is_compatible("1.0.5", "1.0.0", false));
        assert!(!r.is_compatible("1.1.0", "1.0.0", false));
        assert!(r.is_compatible("1.1.0", "1.1.0", false));
    }

    #[test]
    fn test_resolve_latest() {
        let r = Registry::new(None);
        let metadata = make_test_metadata();
        let result = r.resolve_version_spec(&metadata, "latest").unwrap();
        assert_eq!(result, "3.0.0");
    }

    #[test]
    fn test_resolve_exact() {
        let r = Registry::new(None);
        let metadata = make_test_metadata();
        let result = r.resolve_version_spec(&metadata, "1.0.0").unwrap();
        assert_eq!(result, "1.0.0");
    }

    #[test]
    fn test_resolve_caret() {
        let r = Registry::new(None);
        let metadata = make_test_metadata();
        let result = r.resolve_version_spec(&metadata, "^1.0.0").unwrap();
        assert_eq!(result, "1.2.0");
    }

    #[test]
    fn test_resolve_tilde() {
        let r = Registry::new(None);
        let metadata = make_test_metadata();
        let result = r.resolve_version_spec(&metadata, "~1.0.0").unwrap();
        assert_eq!(result, "1.0.0");
    }

    #[test]
    fn test_resolve_minimum() {
        let r = Registry::new(None);
        let metadata = make_test_metadata();
        let result = r.resolve_version_spec(&metadata, ">=2.0.0").unwrap();
        assert_eq!(result, "2.0.0");
    }

    #[test]
    fn test_resolve_invalid() {
        let r = Registry::new(None);
        let metadata = make_test_metadata();
        let result = r.resolve_version_spec(&metadata, "999.0.0");
        assert!(result.is_err());
    }
}
