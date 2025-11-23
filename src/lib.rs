//! OTLP Arrow Flight Library
//!
//! A cross-platform Rust library for receiving OpenTelemetry Protocol (OTLP) messages
//! via gRPC and writing them to local files in Arrow IPC Streaming format.
//!
//! # Features
//!
//! - OTLP gRPC reception
//! - Arrow IPC file storage
//! - Public API for embedded usage
//! - Configurable via YAML, environment variables, or programmatic API
//! - Optional remote forwarding
//! - Mock service for testing
//!
//! # Example
//!
//! ```no_run
//! use otlp_arrow_library::{OtlpLibrary, Config};
//!
//! # async fn example() -> Result<(), otlp_arrow_library::OtlpError> {
//! let config = Config::default();
//! let library = OtlpLibrary::new(config).await?;
//!
//! // Export traces
//! // library.export_trace(span).await?;
//! # Ok(())
//! # }
//! ```

#![warn(missing_docs)]
#![warn(clippy::all)]

pub mod api;
pub mod config;
pub mod error;
pub mod mock;
pub mod otlp;

pub mod python;

// Re-export public API
pub use api::public::OtlpLibrary;
pub use config::{AuthConfig, Config, ConfigBuilder, ForwardingConfig, ForwardingProtocol};
pub use error::{OtlpConfigError, OtlpError, OtlpExportError, OtlpServerError};
pub use mock::service::MockOtlpService;

// Initialize tracing subscriber for structured logging
use tracing_subscriber::EnvFilter;

/// Initialize structured logging
pub fn init_logging() {
    let _ = tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .with_target(false)
        .with_thread_ids(false)
        .with_thread_names(false)
        .init();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_library_initialization() {
        init_logging();
        // Basic smoke test
        assert!(true);
    }
}
