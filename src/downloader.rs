use crate::error::{CNodeError, Result};
use crate::package::PackageVersion;
use base64::Engine;
use reqwest::Client;
use sha2::{Digest, Sha256};
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use tar::Archive;

const PROGRESS_BAR_WIDTH: usize = 30;

static PROGRESS_ENABLED: AtomicBool = AtomicBool::new(true);

pub fn enable_progress(enabled: bool) {
    PROGRESS_ENABLED.store(enabled, Ordering::Relaxed);
}

pub struct Downloader {
    client: Client,
    cache_dir: PathBuf,
    timeout_secs: u64,
}

impl Downloader {
    pub fn new(cache_dir: Option<PathBuf>) -> Result<Self> {
        let cache = cache_dir.unwrap_or_else(|| {
            dirs::cache_dir()
                .map(|d| d.join("cnpm"))
                .unwrap_or_else(|| PathBuf::from(".cnpm_cache"))
        });
        fs::create_dir_all(&cache)?;
        Ok(Downloader {
            client: Client::new(),
            cache_dir: cache,
            timeout_secs: 300,
        })
    }

    pub fn cache_dir(&self) -> &Path {
        &self.cache_dir
    }

    pub async fn download_package(
        &self,
        package: &PackageVersion,
        target_dir: &Path,
    ) -> Result<PathBuf> {
        let cache_path = self.get_cache_path(&package.name, &package.version);
        if cache_path.exists() {
            if PROGRESS_ENABLED.load(Ordering::Relaxed) {
                eprint!("\rcached {}/{}   ", &package.name, &package.version);
                io::stderr().flush().ok();
            }
            return self.extract_to_target(&cache_path, target_dir);
        }
        if PROGRESS_ENABLED.load(Ordering::Relaxed) {
            eprint!("\rdownloading {}/{}", &package.name, &package.version);
            io::stderr().flush().ok();
        }
        let tarball_data = self.fetch_tarball(&package.dist.tarball).await?;
        if let Some(integrity) = &package.dist.integrity {
            self.verify_integrity(&tarball_data, integrity)?;
        }
        fs::create_dir_all(cache_path.parent().unwrap())?;
        fs::write(&cache_path, &tarball_data)?;
        if PROGRESS_ENABLED.load(Ordering::Relaxed) {
            eprint!("\rextracting {}/{}", &package.name, &package.version);
            io::stderr().flush().ok();
        }
        let result = self.extract_to_target(&cache_path, target_dir);
        if PROGRESS_ENABLED.load(Ordering::Relaxed) {
            eprintln!("\rdone {}/{}     ", &package.name, &package.version);
        }
        result
    }

    pub fn compute_tarball_integrity(&self, name: &str, version: &str) -> Result<Option<String>> {
        let cache_path = self.get_cache_path(name, version);
        if !cache_path.exists() {
            return Ok(None);
        }
        let data = fs::read(&cache_path)?;
        Ok(Some(compute_integrity(&data)))
    }

    async fn fetch_tarball(&self, url: &str) -> Result<Vec<u8>> {
        let mut response = self.client
            .get(url)
            .timeout(std::time::Duration::from_secs(self.timeout_secs))
            .send()
            .await?;
        if !response.status().is_success() {
            return Err(CNodeError::DownloadFailed(format!(
                "Failed to download: {}",
                response.status()
            )));
        }

        let total = response.content_length().unwrap_or(0);
        let mut downloaded: u64 = 0;
        let mut data = Vec::with_capacity(total as usize);

        while let Some(chunk) = response.chunk().await? {
            downloaded += chunk.len() as u64;
            data.extend_from_slice(&chunk);
            if PROGRESS_ENABLED.load(Ordering::Relaxed) && total > 0 {
                let pct = (downloaded as f64 / total as f64) * 100.0;
                let filled = ((pct / 100.0) * PROGRESS_BAR_WIDTH as f64) as usize;
                let filled = filled.min(PROGRESS_BAR_WIDTH);
                let bar: String = (0..PROGRESS_BAR_WIDTH)
                    .map(|i| if i < filled { '=' } else if i == filled && i < PROGRESS_BAR_WIDTH - 1 { '>' } else { ' ' })
                    .collect();
                write!(io::stderr(), "\r[{}] {:.0}%", bar, pct).ok();
                io::stderr().flush().ok();
            }
        }

        Ok(data)
    }

    fn extract_to_target(&self, cache_path: &Path, target_dir: &Path) -> Result<PathBuf> {
        let file = fs::File::open(cache_path)
            .map_err(|e| CNodeError::ArchiveError(e.to_string()))?;
        let decoder = flate2::read::GzDecoder::new(file);
        let mut archive = Archive::new(decoder);

        let temp_dir = tempfile::tempdir()?;
        archive
            .unpack(temp_dir.path())
            .map_err(|e| CNodeError::ArchiveError(e.to_string()))?;

        let entries = fs::read_dir(temp_dir.path())?;
        let mut top_dirs: Vec<PathBuf> = entries
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| p.is_dir())
            .collect();
        top_dirs.sort();

        if let Some(package_dir) = top_dirs.first() {
            copy_dir_all(package_dir, target_dir)?;
        } else {
            copy_dir_all(temp_dir.path(), target_dir)?;
        }

        Ok(target_dir.to_path_buf())
    }

    fn get_cache_path(&self, name: &str, version: &str) -> PathBuf {
        let safe_name = name.replace('/', "_").replace('@', "");
        self.cache_dir
            .join(format!("{}-{}.tar.gz", safe_name, version))
    }

    fn verify_integrity(&self, data: &[u8], integrity: &str) -> Result<()> {
        if integrity.starts_with("sha512-") {
            let hash_part = &integrity[7..];
            let mut hasher = sha2::Sha512::new();
            hasher.update(data);
            let result = hasher.finalize();
            let computed = base64::engine::general_purpose::STANDARD.encode(result);
            if computed == hash_part {
                Ok(())
            } else {
                Err(CNodeError::DownloadFailed("Integrity check failed (sha512)".to_string()))
            }
        } else if integrity.starts_with("sha256-") {
            let hash_part = &integrity[7..];
            let mut hasher = Sha256::new();
            hasher.update(data);
            let result = hasher.finalize();
            let computed = base64::engine::general_purpose::STANDARD.encode(result);
            if computed == hash_part {
                Ok(())
            } else {
                Err(CNodeError::DownloadFailed("Integrity check failed (sha256)".to_string()))
            }
        } else {
            Ok(())
        }
    }
}

pub fn compute_integrity(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    let result = hasher.finalize();
    let encoded = base64::engine::general_purpose::STANDARD.encode(result);
    format!("sha256-{}", encoded)
}

fn copy_dir_all(src: &Path, dst: &Path) -> std::io::Result<()> {
    if !dst.exists() {
        fs::create_dir_all(dst)?;
    }
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        let src_path = entry.path();
        let file_name = entry.file_name();
        let dst_path = dst.join(&file_name);
        if file_type.is_dir() {
            copy_dir_all(&src_path, &dst_path)?;
        } else {
            fs::copy(&src_path, &dst_path)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_integrity_empty() {
        let result = compute_integrity(b"");
        assert!(result.starts_with("sha256-"));
    }

    #[test]
    fn test_compute_integrity_deterministic() {
        let data = b"hello world";
        let result1 = compute_integrity(data);
        let result2 = compute_integrity(data);
        assert_eq!(result1, result2);
    }

    #[test]
    fn test_compute_integrity_different() {
        let result1 = compute_integrity(b"data1");
        let result2 = compute_integrity(b"data2");
        assert_ne!(result1, result2);
    }

    #[test]
    fn test_compute_integrity_format() {
        let result = compute_integrity(b"test");
        assert!(result.starts_with("sha256-"));
        assert!(result.len() > 7);
    }

    #[test]
    fn test_new_downloader_creates_cache_dir() {
        let tmp = tempfile::tempdir().unwrap();
        let cache_path = tmp.path().join("cnpm-cache");
        let downloader = Downloader::new(Some(cache_path.clone())).unwrap();
        assert!(cache_path.exists());
        assert!(downloader.cache_dir().exists());
    }

    #[test]
    fn test_get_cache_path() {
        let tmp = tempfile::tempdir().unwrap();
        let downloader = Downloader::new(Some(tmp.path().join("cache"))).unwrap();
        let path = downloader.get_cache_path("express", "4.18.2");
        assert!(path.to_string_lossy().contains("express"));
        assert!(path.to_string_lossy().contains("4.18.2"));
        assert!(path.to_string_lossy().ends_with(".tar.gz"));
    }

    #[test]
    fn test_get_cache_path_scoped() {
        let tmp = tempfile::tempdir().unwrap();
        let downloader = Downloader::new(Some(tmp.path().join("cache"))).unwrap();
        let path = downloader.get_cache_path("@angular/core", "16.0.0");
        let name = path.to_string_lossy();
        assert!(name.contains("angular_core"));
    }
}
