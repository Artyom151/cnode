use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Package {
    pub name: String,
    pub version: String,
    pub description: Option<String>,
    pub main: Option<String>,
    pub types: Option<String>,
    pub keywords: Option<Vec<String>>,
    pub author: Option<String>,
    pub license: Option<String>,
    pub repository: Option<String>,
    pub homepage: Option<String>,
    pub bugs: Option<String>,
    pub dependencies: Option<HashMap<String, String>>,
    pub dev_dependencies: Option<HashMap<String, String>>,
    pub peer_dependencies: Option<HashMap<String, String>>,
    pub optional_dependencies: Option<HashMap<String, String>>,
    pub engines: Option<HashMap<String, String>>,
    pub bin: Option<HashMap<String, String>>,
    pub scripts: Option<HashMap<String, String>>,
}

impl Package {
    pub fn new(name: String, version: String) -> Self {
        Package {
            name,
            version,
            description: None,
            main: None,
            types: None,
            keywords: None,
            author: None,
            license: None,
            repository: None,
            homepage: None,
            bugs: None,
            dependencies: None,
            dev_dependencies: None,
            peer_dependencies: None,
            optional_dependencies: None,
            engines: None,
            bin: None,
            scripts: None,
        }
    }

    pub fn set_description(mut self, desc: String) -> Self {
        self.description = Some(desc);
        self
    }

    pub fn set_main(mut self, main: String) -> Self {
        self.main = Some(main);
        self
    }

    pub fn set_dependencies(mut self, deps: HashMap<String, String>) -> Self {
        self.dependencies = Some(deps);
        self
    }

    pub fn add_dependency(&mut self, name: String, version: String) {
        self.dependencies
            .get_or_insert_with(HashMap::new)
            .insert(name, version);
    }

    pub fn add_dev_dependency(&mut self, name: String, version: String) {
        self.dev_dependencies
            .get_or_insert_with(HashMap::new)
            .insert(name, version);
    }

    pub fn all_dependencies(&self) -> HashMap<String, String> {
        let mut all = HashMap::new();
        if let Some(deps) = &self.dependencies {
            all.extend(deps.clone());
        }
        if let Some(peer_deps) = &self.peer_dependencies {
            all.extend(peer_deps.clone());
        }
        if let Some(opt_deps) = &self.optional_dependencies {
            all.extend(opt_deps.clone());
        }
        all
    }

    pub fn identifier(&self) -> String {
        format!("{}@{}", self.name, self.version)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageMetadata {
    pub name: String,
    pub versions: HashMap<String, PackageVersion>,
    #[serde(rename = "dist-tags")]
    pub dist_tags: DistTags,
    pub time: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DistTags {
    pub latest: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageVersion {
    pub version: String,
    pub name: String,
    pub description: Option<String>,
    pub dependencies: Option<HashMap<String, String>>,
    pub dist: DistInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DistInfo {
    pub tarball: String,
    pub shasum: String,
    pub integrity: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchPackage {
    pub name: String,
    pub version: String,
    pub description: Option<String>,
    pub keywords: Option<Vec<String>>,
    pub date: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub objects: Vec<SearchObject>,
    pub total: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchObject {
    pub package: SearchPackage,
    pub score: Option<serde_json::Value>,
    #[serde(rename = "searchScore")]
    pub search_score: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Advisory {
    #[serde(default)]
    pub id: u32,
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub severity: String,
    #[serde(default)]
    pub package_name: Option<String>,
    #[serde(default)]
    pub vulnerable_versions: Option<String>,
    #[serde(default)]
    pub patched_versions: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AuditResponse {
    #[serde(default)]
    pub advisories: HashMap<String, Advisory>,
    #[serde(default)]
    pub metadata: AuditMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AuditMetadata {
    #[serde(default)]
    pub vulnerabilities: VulnerabilityCounts,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct VulnerabilityCounts {
    #[serde(default)]
    pub info: u32,
    #[serde(default)]
    pub low: u32,
    #[serde(default)]
    pub moderate: u32,
    #[serde(default)]
    pub high: u32,
    #[serde(default)]
    pub critical: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_package_new() {
        let pkg = Package::new("test".to_string(), "1.0.0".to_string());
        assert_eq!(pkg.name, "test");
        assert_eq!(pkg.version, "1.0.0");
        assert!(pkg.dependencies.is_none());
    }

    #[test]
    fn test_package_identifier() {
        let pkg = Package::new("lodash".to_string(), "4.17.21".to_string());
        assert_eq!(pkg.identifier(), "lodash@4.17.21");
    }

    #[test]
    fn test_package_add_dependency() {
        let mut pkg = Package::new("root".to_string(), "1.0.0".to_string());
        pkg.add_dependency("express".to_string(), "^4.0.0".to_string());
        pkg.add_dependency("lodash".to_string(), "^4.0.0".to_string());
        let deps = pkg.dependencies.unwrap();
        assert_eq!(deps.get("express"), Some(&"^4.0.0".to_string()));
        assert_eq!(deps.get("lodash"), Some(&"^4.0.0".to_string()));
    }

    #[test]
    fn test_package_all_dependencies() {
        let mut pkg = Package::new("root".to_string(), "1.0.0".to_string());
        pkg.add_dependency("express".to_string(), "^4.0.0".to_string());
        pkg.add_dev_dependency("mocha".to_string(), "^10.0.0".to_string());
        let all = pkg.all_dependencies();
        assert!(all.contains_key("express"));
        assert!(!all.contains_key("mocha"));
    }

    #[test]
    fn test_package_serde_roundtrip() {
        let mut pkg = Package::new("test".to_string(), "1.0.0".to_string());
        pkg.add_dependency("dep1".to_string(), "^1.0.0".to_string());
        let json = serde_json::to_string(&pkg).unwrap();
        let deserialized: Package = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.name, "test");
        assert_eq!(deserialized.version, "1.0.0");
    }

    #[test]
    fn test_dist_tags_deserialize() {
        let json = r#"{"latest": "4.18.2"}"#;
        let tags: DistTags = serde_json::from_str(json).unwrap();
        assert_eq!(tags.latest, "4.18.2");
    }

    #[test]
    fn test_package_version_deserialize() {
        let json = r#"{
            "version": "4.18.2",
            "name": "express",
            "description": "web framework",
            "dist": {
                "tarball": "https://registry.npmjs.org/express/-/express-4.18.2.tgz",
                "shasum": "abc123",
                "integrity": "sha512-abc=="
            }
        }"#;
        let version: PackageVersion = serde_json::from_str(json).unwrap();
        assert_eq!(version.version, "4.18.2");
        assert_eq!(version.name, "express");
        assert_eq!(version.dist.tarball, "https://registry.npmjs.org/express/-/express-4.18.2.tgz");
    }

    #[test]
    fn test_search_package_deserialize() {
        let json = r#"{
            "name": "express",
            "version": "4.18.2",
            "description": "web framework",
            "keywords": ["web"]
        }"#;
        let sp: SearchPackage = serde_json::from_str(json).unwrap();
        assert_eq!(sp.name, "express");
        assert_eq!(sp.keywords.unwrap(), vec!["web"]);
    }

    #[test]
    fn test_advisory_default() {
        let advisory = Advisory::default();
        assert_eq!(advisory.id, 0);
        assert!(advisory.title.is_empty());
        assert!(advisory.severity.is_empty());
    }

    #[test]
    fn test_audit_response_deserialize() {
        let json = r#"{
            "advisories": {
                "1523": {
                    "id": 1523,
                    "title": "Prototype Pollution",
                    "severity": "high",
                    "package_name": "lodash"
                }
            },
            "metadata": {
                "vulnerabilities": {
                    "high": 1,
                    "low": 0
                }
            }
        }"#;
        let response: AuditResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.advisories.len(), 1);
        let adv = response.advisories.get("1523").unwrap();
        assert_eq!(adv.title, "Prototype Pollution");
        assert_eq!(response.metadata.vulnerabilities.high, 1);
    }
}
