use crate::config::Config;
use crate::cli::ConfigCommand;
use crate::error::Result;
use std::fs;
use std::path::Path;

pub async fn handle(subcommand: ConfigCommand) -> Result<()> {
    let config_path = Config::default_path();
    let mut config = if config_path.exists() {
        Config::load(&config_path)?
    } else {
        Config::new()
    };

    match subcommand {
        ConfigCommand::Get { key } => {
            let value = match key.as_str() {
                "registry" => config.get_registry(),
                "cache-dir" => config.get_cache_dir().to_string_lossy().to_string(),
                "parallel-downloads" => config.get_parallel_downloads().to_string(),
                "strict" => config.is_strict().to_string(),
                _ => return Err(crate::error::CNodeError::ConfigError(format!("Unknown key: {}", key))),
            };
            println!("{}", value);
        }
        ConfigCommand::Set { key, value } => {
            match key.as_str() {
                "registry" => config.registry = Some(value),
                "cache-dir" => config.cache_dir = Some(Path::new(&value).to_path_buf()),
                "parallel-downloads" => {
                    let n: usize = value.parse()
                        .map_err(|_| crate::error::CNodeError::ConfigError("Invalid number".to_string()))?;
                    config.parallel_downloads = Some(n);
                }
                "strict" => {
                    let b = value == "true";
                    config.strict = Some(b);
                }
                _ => return Err(crate::error::CNodeError::ConfigError(format!("Unknown key: {}", key))),
            }
            config.save(&config_path)?;
        }
        ConfigCommand::List => {
            if !config_path.exists() {
                println!("(empty)");
                return Ok(());
            }
            let content = fs::read_to_string(&config_path)
                .unwrap_or_default();
            if content.is_empty() {
                println!("(empty)");
            } else {
                print!("{}", content);
            }
        }
        ConfigCommand::Delete { key } => {
            match key.as_str() {
                "registry" => config.registry = None,
                "cache-dir" => config.cache_dir = None,
                "parallel-downloads" => config.parallel_downloads = None,
                "strict" => config.strict = None,
                _ => return Err(crate::error::CNodeError::ConfigError(format!("Unknown key: {}", key))),
            }
            config.save(&config_path)?;
        }
    }

    Ok(())
}
