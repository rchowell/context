use thiserror::Error;

/// Result type alias for Context operations
pub type Result<T> = std::result::Result<T, ContextError>;

/// Unified error types for Context operations
#[derive(Error, Debug)]
pub enum ContextError {
    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("YAML error: {0}")]
    YamlError(#[from] serde_yaml::Error),

    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("Invalid document: {0}")]
    InvalidDocument(String),

    #[error("Document not found: {0}")]
    DocumentNotFound(String),

    #[error("Invalid hash format: {0}")]
    InvalidHashFormat(String),

    #[error("Validation failed: {0}")]
    ValidationFailed(String),

    #[error("Directory not initialized: {0}")]
    NotInitialized(String),

    #[error("fatal: not a context repository (or any parent directories): .context")]
    NotARepository,

    #[error("Cache error: {0}")]
    CacheError(String),

    #[error("Search error: {0}")]
    SearchError(String),

    #[error("Sync error: {0}")]
    SyncError(String),

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("{0}")]
    Other(String),
}
