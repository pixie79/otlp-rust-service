//! Error types for OTLP Arrow Library
//!
//! Defines all error types used throughout the library with clear error messages
//! and context for debugging.

use thiserror::Error;

/// Main error type for the OTLP Arrow Library
#[derive(Error, Debug)]
pub enum OtlpError {
    /// Configuration-related errors
    #[error("Configuration error: {0}")]
    Config(#[from] OtlpConfigError),

    /// Export/processing errors
    #[error("Export error: {0}")]
    Export(#[from] OtlpExportError),

    /// I/O errors
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Server-related errors
    #[error("Server error: {0}")]
    Server(#[from] OtlpServerError),
}

/// Configuration-related errors
#[derive(Error, Debug)]
pub enum OtlpConfigError {
    /// Invalid output directory path
    #[error("Invalid output directory: {0}")]
    InvalidOutputDir(String),

    /// Invalid interval value
    #[error("Invalid interval: {0}")]
    InvalidInterval(String),

    /// Missing required configuration field
    #[error("Missing required field: {0}")]
    MissingRequiredField(String),

    /// Invalid URL format
    #[error("Invalid URL format: {0}")]
    InvalidUrl(String),

    /// Configuration validation failed
    #[error("Configuration validation failed: {0}")]
    ValidationFailed(String),
}

/// Export/processing errors
#[derive(Error, Debug)]
pub enum OtlpExportError {
    /// Message buffer is full
    #[error("Message buffer is full")]
    BufferFull,

    /// Serialization error
    #[error("Serialization error: {0}")]
    SerializationError(String),

    /// Remote forwarding error
    #[error("Remote forwarding failed: {0}")]
    ForwardingError(String),

    /// Arrow IPC conversion error
    #[error("Arrow IPC conversion error: {0}")]
    ArrowConversionError(String),

    /// Cleanup error
    #[error("Cleanup error: {0}")]
    CleanupError(String),

    /// Format conversion error
    #[error("Format conversion error: {0}")]
    FormatConversionError(String),
}

/// Server-related errors
#[derive(Error, Debug)]
pub enum OtlpServerError {
    /// Failed to bind server address
    #[error("Failed to bind server address: {0}")]
    BindError(String),

    /// Failed to start server
    #[error("Failed to start server: {0}")]
    StartupError(String),

    /// Server shutdown error
    #[error("Server shutdown error: {0}")]
    ShutdownError(String),
}

impl From<anyhow::Error> for OtlpError {
    fn from(err: anyhow::Error) -> Self {
        OtlpError::Io(std::io::Error::new(
            std::io::ErrorKind::Other,
            err.to_string(),
        ))
    }
}

