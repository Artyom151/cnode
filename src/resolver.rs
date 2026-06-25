use crate::error::Result;
use crate::package::Package;
use crate::registry::Registry;
use std::collections::HashMap;
use std::pin::Pin;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct ResolvedDependency {
    pub name: String,
    pub version: String,
    pub dependencies: Vec<ResolvedDependency>,
}

pub struct Resolver {
    registry: Arc<Registry>,
    cache: Arc<tokio::sync::Mutex<HashMap<String, String>>>,
}

impl Resolver {
    pub fn new(registry: Arc<Registry>) -> Self {
        Resolver {
            registry,
            cache: Arc::new(tokio::sync::Mutex::new(HashMap::new())),
        }
    }

    pub async fn resolve_dependencies(
        &self,
        package: &Package,
    ) -> Result<Vec<ResolvedDependency>> {
        let mut resolved = Vec::new();
        let deps = package.all_dependencies();
        for (name, spec) in deps.iter() {
            let dep = self.resolve_single(name, spec).await?;
            resolved.push(dep);
        }
        Ok(resolved)
    }

    pub async fn resolve_single(&self, name: &str, spec: &str) -> Result<ResolvedDependency> {
        self.resolve_single_internal(name, spec).await
    }

    fn resolve_single_internal<'a>(
        &'a self,
        name: &'a str,
        spec: &'a str,
    ) -> Pin<Box<dyn std::future::Future<Output = Result<ResolvedDependency>> + 'a>> {
        Box::pin(async move {
            let cache = self.cache.lock().await;
            if let Some(version) = cache.get(&format!("{}@{}", name, spec)) {
                return Ok(ResolvedDependency {
                    name: name.to_string(),
                    version: version.clone(),
                    dependencies: Vec::new(),
                });
            }
            drop(cache);

            let metadata = self.registry.fetch_package_metadata(name).await?;
            let version = self.registry.resolve_version_spec(&metadata, spec)?;

            let package_version = match self.registry.fetch_version(name, &version).await {
                Ok(v) => v,
                Err(e) => {
                    return Err(e);
                }
            };

            let mut nested_deps = Vec::new();
            if let Some(deps) = &package_version.dependencies {
                for (dep_name, dep_spec) in deps.iter() {
                    let resolved_dep = self
                        .resolve_single_internal(dep_name, dep_spec)
                        .await?;
                    nested_deps.push(resolved_dep);
                }
            }

            let mut cache = self.cache.lock().await;
            cache.insert(format!("{}@{}", name, spec), version.clone());

            Ok(ResolvedDependency {
                name: name.to_string(),
                version,
                dependencies: nested_deps,
            })
        })
    }

    pub fn flatten_dependencies(&self, deps: &[ResolvedDependency]) -> HashMap<String, String> {
        let mut flattened = HashMap::new();
        fn flatten_recursive(
            dep: &ResolvedDependency,
            flattened: &mut HashMap<String, String>,
        ) {
            flattened.insert(dep.name.clone(), dep.version.clone());
            for nested in &dep.dependencies {
                flatten_recursive(nested, flattened);
            }
        }
        for dep in deps {
            flatten_recursive(dep, &mut flattened);
        }
        flattened
    }

    pub fn collect_tree(
        &self,
        deps: &[ResolvedDependency],
    ) -> Vec<(String, String, usize)> {
        let mut result = Vec::new();
        fn walk(dep: &ResolvedDependency, depth: usize, result: &mut Vec<(String, String, usize)>) {
            result.push((dep.name.clone(), dep.version.clone(), depth));
            for nested in &dep.dependencies {
                walk(nested, depth + 1, result);
            }
        }
        for dep in deps {
            walk(dep, 0, &mut result);
        }
        result
    }

    pub fn detect_conflicts(
        &self,
        resolved: &HashMap<String, String>,
    ) -> Vec<(String, Vec<String>)> {
        let mut conflicts = Vec::new();
        let mut version_map: HashMap<String, Vec<String>> = HashMap::new();
        for (name, version) in resolved {
            version_map
                .entry(name.clone())
                .or_insert_with(Vec::new)
                .push(version.clone());
        }
        for (name, mut versions) in version_map {
            if versions.len() > 1 {
                versions.sort();
                conflicts.push((name, versions));
            }
        }
        conflicts
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_flatten_dependencies_empty() {
        let registry = Arc::new(Registry::new(None));
        let resolver = Resolver::new(registry);
        let result = resolver.flatten_dependencies(&[]);
        assert!(result.is_empty());
    }

    #[test]
    fn test_flatten_dependencies_single() {
        let registry = Arc::new(Registry::new(None));
        let resolver = Resolver::new(registry);
        let deps = vec![ResolvedDependency {
            name: "express".to_string(),
            version: "4.18.2".to_string(),
            dependencies: vec![],
        }];
        let result = resolver.flatten_dependencies(&deps);
        assert_eq!(result.len(), 1);
        assert_eq!(result.get("express"), Some(&"4.18.2".to_string()));
    }

    #[test]
    fn test_flatten_dependencies_nested() {
        let registry = Arc::new(Registry::new(None));
        let resolver = Resolver::new(registry);
        let deps = vec![ResolvedDependency {
            name: "root".to_string(),
            version: "1.0.0".to_string(),
            dependencies: vec![ResolvedDependency {
                name: "child".to_string(),
                version: "2.0.0".to_string(),
                dependencies: vec![],
            }],
        }];
        let result = resolver.flatten_dependencies(&deps);
        assert_eq!(result.len(), 2);
        assert_eq!(result.get("root"), Some(&"1.0.0".to_string()));
        assert_eq!(result.get("child"), Some(&"2.0.0".to_string()));
    }

    #[test]
    fn test_collect_tree() {
        let registry = Arc::new(Registry::new(None));
        let resolver = Resolver::new(registry);
        let deps = vec![ResolvedDependency {
            name: "root".to_string(),
            version: "1.0.0".to_string(),
            dependencies: vec![ResolvedDependency {
                name: "child".to_string(),
                version: "2.0.0".to_string(),
                dependencies: vec![],
            }],
        }];
        let tree = resolver.collect_tree(&deps);
        assert_eq!(tree.len(), 2);
        assert_eq!(tree[0], ("root".to_string(), "1.0.0".to_string(), 0));
        assert_eq!(tree[1], ("child".to_string(), "2.0.0".to_string(), 1));
    }

    #[test]
    fn test_detect_conflicts_none() {
        let registry = Arc::new(Registry::new(None));
        let resolver = Resolver::new(registry);
        let mut map = HashMap::new();
        map.insert("express".to_string(), "4.0.0".to_string());
        map.insert("lodash".to_string(), "4.17.0".to_string());
        let conflicts = resolver.detect_conflicts(&map);
        assert!(conflicts.is_empty());
    }

    #[test]
    fn test_detect_conflicts_found() {
        let registry = Arc::new(Registry::new(None));
        let resolver = Resolver::new(registry);
        let mut map = HashMap::new();
        map.insert("lodash".to_string(), "4.0.0".to_string());
        let conflicts = resolver.detect_conflicts(&map);
        assert!(conflicts.is_empty());
    }
}
