use crate::cli::CacheCommand;
use crate::error::Result;
use std::fs;
use std::path::PathBuf;

pub async fn handle(subcommand: CacheCommand, cache_dir: Option<PathBuf>) -> Result<()> {
    let cache_dir = cache_dir.unwrap_or_else(|| {
        dirs::cache_dir()
            .map(|d| d.join("cnpm"))
            .unwrap_or_else(|| PathBuf::from(".cnpm_cache"))
    });

    match subcommand {
        CacheCommand::Clean => {
            if cache_dir.exists() {
                for entry in fs::read_dir(&cache_dir)? {
                    let entry = entry?;
                    let path = entry.path();
                    if entry.file_type()?.is_dir() {
                        fs::remove_dir_all(&path)?;
                    } else {
                        fs::remove_file(&path)?;
                    }
                }
            }
        }
        CacheCommand::Verify => {
            if !cache_dir.exists() {
                println!("(empty)");
                return Ok(());
            }
            let mut ok = 0u32;
            let mut failed = 0u32;
            for entry in fs::read_dir(&cache_dir)? {
                let entry = entry?;
                let path = entry.path();
                if path.extension().and_then(|e| e.to_str()) == Some("gz") {
                    match fs::read(&path) {
                        Ok(_data) => ok += 1,
                        Err(_) => {
                            failed += 1;
                        }
                    }
                }
            }
            println!("{} valid, {} corrupted", ok, failed);
        }
        CacheCommand::List => {
            if !cache_dir.exists() {
                println!("(empty)");
                return Ok(());
            }
            let mut entries: Vec<_> = fs::read_dir(&cache_dir)?
                .filter_map(|e| e.ok())
                .filter(|e| e.path().extension().and_then(|x| x.to_str()) == Some("gz"))
                .map(|e| e.file_name().to_string_lossy().to_string())
                .collect();
            entries.sort();
            for entry in entries {
                println!("{}", entry);
            }
        }
    }

    Ok(())
}
