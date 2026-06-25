use crate::error::{CNodeError, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub registry: Option<String>,
    pub cache_dir: Option<PathBuf>,
    pub node_modules_dir: Option<PathBuf>,
    pub lock_file: Option<String>,
    pub strict: Option<bool>,
    pub parallel_downloads: Option<usize>,
}

impl Config {
    pub fn new() -> Self {
        Config {
            registry: None,
            cache_dir: None,
            node_modules_dir: None,
            lock_file: None,
            strict: None,
            parallel_downloads: None,
        }
    }

    pub fn default_path() -> PathBuf {
        dirs::home_dir()
            .map(|d| d.join(".cnpmrc"))
            .unwrap_or_else(|| PathBuf::from(".cnpmrc"))
    }

    pub fn load(path: &Path) -> Result<Self> {
        if !path.exists() {
            return Ok(Config::new());
        }
        let content = fs::read_to_string(path)
            .map_err(|e| CNodeError::ConfigError(e.to_string()))?;
        let config: Config = toml::from_str(&content)
            .map_err(|e| CNodeError::TomlError(e.to_string()))?;
        Ok(config)
    }

    pub fn load_default() -> Result<Self> {
        Self::load(&Self::default_path())
    }

    pub fn save(&self, path: &Path) -> Result<()> {
        let content = toml::to_string_pretty(self)
            .map_err(|e| CNodeError::TomlError(e.to_string()))?;
        fs::write(path, content)
            .map_err(|e| CNodeError::ConfigError(e.to_string()))?;
        Ok(())
    }

    pub fn get_registry(&self) -> String {
        self.registry
            .clone()
            .unwrap_or_else(|| "https://registry.npmjs.org".to_string())
    }

    pub fn get_cache_dir(&self) -> PathBuf {
        self.cache_dir.clone().unwrap_or_else(|| {
            dirs::cache_dir()
                .map(|d| d.join("cnpm"))
                .unwrap_or_else(|| PathBuf::from(".cnpm_cache"))
        })
    }

    pub fn get_node_modules_dir(&self) -> PathBuf {
        self.node_modules_dir
            .clone()
            .unwrap_or_else(|| PathBuf::from("node_modules"))
    }

    pub fn get_parallel_downloads(&self) -> usize {
        self.parallel_downloads.unwrap_or(4)
    }

    pub fn is_strict(&self) -> bool {
        self.strict.unwrap_or(false)
    }
}

impl Default for Config {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_new() {
        let config = Config::new();
        assert!(config.registry.is_none());
        assert!(config.cache_dir.is_none());
    }

    #[test]
    fn test_config_default_registry() {
        let config = Config::new();
        assert_eq!(config.get_registry(), "https://registry.npmjs.org");
    }

    #[test]
    fn test_config_custom_registry() {
        let mut config = Config::new();
        config.registry = Some("https://registry.npmmirror.com".to_string());
        assert_eq!(config.get_registry(), "https://registry.npmmirror.com");
    }

    #[test]
    fn test_config_parallel_downloads_default() {
        let config = Config::new();
        assert_eq!(config.get_parallel_downloads(), 4);
    }

    #[test]
    fn test_config_strict_default() {
        let config = Config::new();
        assert!(!config.is_strict());
    }

    #[test]
    fn test_config_save_and_load() {
        let tmp = tempfile::tempdir().unwrap();
        let config_path = tmp.path().join("config.toml");

        let mut config = Config::new();
        config.registry = Some("https://example.com".to_string());
        config.parallel_downloads = Some(8);
        config.save(&config_path).unwrap();

        let loaded = Config::load(&config_path).unwrap();
        assert_eq!(loaded.get_registry(), "https://example.com");
        assert_eq!(loaded.get_parallel_downloads(), 8);
    }

    #[test]
    fn test_config_load_nonexistent() {
        let tmp = tempfile::tempdir().unwrap();
        let config_path = tmp.path().join("nonexistent.toml");
        let config = Config::load(&config_path).unwrap();
        assert!(config.registry.is_none());
    }

    #[test]
    fn test_config_get_cache_dir() {
        let config = Config::new();
        let cache_dir = config.get_cache_dir();
        assert!(!cache_dir.as_os_str().is_empty());
    }

    #[test]
    fn test_config_default_path() {
        let path = Config::default_path();
        assert!(path.to_string_lossy().contains(".cnpmrc"));
    }
}
