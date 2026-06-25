use thiserror::Error;

#[derive(Error, Debug)]
pub enum CNodeError {
    #[error("Network error: {0}")]
    NetworkError(String),

    #[error("Package not found: {0}")]
    PackageNotFound(String),

    #[error("Invalid version: {0}")]
    InvalidVersion(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("Dependency resolution failed: {0}")]
    ResolutionFailed(String),

    #[error("Download failed: {0}")]
    DownloadFailed(String),

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("Archive error: {0}")]
    ArchiveError(String),

    #[error("TOML parse error: {0}")]
    TomlError(String),

    #[error("{0}")]
    Custom(String),
}

pub type Result<T> = std::result::Result<T, CNodeError>;

impl From<reqwest::Error> for CNodeError {
    fn from(err: reqwest::Error) -> Self {
        CNodeError::NetworkError(err.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display_network() {
        let err = CNodeError::NetworkError("connection refused".to_string());
        assert_eq!(err.to_string(), "Network error: connection refused");
    }

    #[test]
    fn test_error_display_package_not_found() {
        let err = CNodeError::PackageNotFound("express".to_string());
        assert_eq!(err.to_string(), "Package not found: express");
    }

    #[test]
    fn test_error_display_invalid_version() {
        let err = CNodeError::InvalidVersion("abc".to_string());
        assert_eq!(err.to_string(), "Invalid version: abc");
    }

    #[test]
    fn test_error_display_custom() {
        let err = CNodeError::Custom("something went wrong".to_string());
        assert_eq!(err.to_string(), "something went wrong");
    }

    #[test]
    fn test_error_from_io() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let err: CNodeError = io_err.into();
        assert!(matches!(err, CNodeError::IoError(_)));
    }

    #[test]
    fn test_result_type() {
        let ok: Result<i32> = Ok(42);
        assert_eq!(ok.unwrap(), 42);

        let err: Result<i32> = Err(CNodeError::Custom("fail".to_string()));
        assert!(err.is_err());
    }
}
