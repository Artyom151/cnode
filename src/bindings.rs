use napi_derive::napi;
use std::sync::Arc;

#[napi(object)]
pub struct JsPackageInfo {
    pub name: String,
    pub version: String,
    pub description: Option<String>,
}

#[napi]
pub struct CNode {
    registry_url: String,
    runtime: tokio::runtime::Runtime,
}

#[napi]
impl CNode {
    #[napi(constructor)]
    pub fn new(registry_url: Option<String>) -> Self {
        CNode {
            registry_url: registry_url.unwrap_or_else(|| "https://registry.npmjs.org".to_string()),
            runtime: tokio::runtime::Runtime::new().expect("failed to create tokio runtime"),
        }
    }

    #[napi]
    pub fn get_registry_url(&self) -> String {
        self.registry_url.clone()
    }

    #[napi]
    pub fn set_registry_url(&mut self, url: String) {
        self.registry_url = url;
    }

    #[napi]
    pub fn fetch_package_info(&self, package_name: String) -> serde_json::Value {
        let registry = cnode_lib::Registry::new(Some(self.registry_url.clone()));
        match self.runtime.block_on(registry.fetch_package_metadata(&package_name)) {
            Ok(metadata) => {
                let latest = &metadata.dist_tags.latest;
                let version_info = metadata.versions.get(latest);
                serde_json::json!({
                    "name": metadata.name,
                    "latest": latest,
                    "versions": metadata.versions.len(),
                    "description": version_info.and_then(|v| v.description.as_deref()),
                    "hasDependencies": version_info.map(|v| v.dependencies.is_some()).unwrap_or(false),
                })
            }
            Err(e) => serde_json::json!({ "error": e.to_string() }),
        }
    }

    #[napi]
    pub fn search_packages(&self, query: String, limit: Option<u32>) -> Vec<serde_json::Value> {
        let limit = limit.unwrap_or(10) as usize;
        let registry = cnode_lib::Registry::new(Some(self.registry_url.clone()));
        match self.runtime.block_on(registry.search(&query, limit)) {
            Ok(results) => {
                results.into_iter().map(|p| {
                    serde_json::json!({
                        "name": p.name,
                        "version": p.version,
                        "description": p.description,
                    })
                }).collect()
            }
            Err(_) => vec![],
        }
    }
}
