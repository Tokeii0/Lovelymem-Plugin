use thiserror::Error;

/// Custom error types for memstrap
#[derive(Error, Debug)]
pub enum MemstrapError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("CSV error: {0}")]
    Csv(#[from] csv::Error),
    
    #[error("Regex error: {0}")]
    Regex(#[from] regex::Error),
    
    #[error("Memory mapping error: {0}")]
    Mmap(String),
    
    #[error("Invalid encoding: {0}")]
    InvalidEncoding(String),
    
    #[error("Configuration error: {0}")]
    Config(String),
}

/// Result type alias for memstrap operations
pub type Result<T> = std::result::Result<T, MemstrapError>;
