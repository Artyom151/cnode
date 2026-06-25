use crate::error::Result;
use std::fs;
use std::path::PathBuf;

pub async fn handle(cache_dir: Option<PathBuf>) -> Result<()> {
    let cache_dir = cache_dir.unwrap_or_else(|| {
        dirs::cache_dir()
            .map(|d| d.join("cnpm"))
            .unwrap_or_else(|| PathBuf::from(".cnpm_cache"))
    });

    if !cache_dir.exists() {
        return Ok(());
    }

    for entry in fs::read_dir(&cache_dir)? {
        let entry = entry?;
        let path = entry.path();
        if entry.file_type()?.is_dir() {
            fs::remove_dir_all(&path)?;
        } else {
            fs::remove_file(&path)?;
        }
    }

    Ok(())
}
