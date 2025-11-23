//! Configuration type definitions
//!
//! Defines all configuration structures for the OTLP Arrow Library.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

use crate::error::OtlpConfigError;

/// Protocol to use for forwarding messages to remote endpoints
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ForwardingProtocol {
    /// Standard OTLP gRPC with Protobuf
    Protobuf,
    /// OpenTelemetry Protocol with Apache Arrow (OTAP)
    ArrowFlight,
}

impl Default for ForwardingProtocol {
    fn default() -> Self {
        Self::Protobuf
    }
}

/// Configuration for gRPC protocol support (Protobuf and Arrow Flight)
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ProtocolConfig {
    /// Whether gRPC with Protobuf protocol is enabled (default: true)
    #[serde(default = "default_protobuf_enabled")]
    pub protobuf_enabled: bool,

    /// Port for gRPC with Protobuf (default: 4317, standard OTLP port)
    #[serde(default = "default_protobuf_port")]
    pub protobuf_port: u16,

    /// Whether gRPC with Arrow Flight IPC protocol is enabled (default: true)
    #[serde(default = "default_arrow_flight_enabled")]
    pub arrow_flight_enabled: bool,

    /// Port for gRPC with Arrow Flight IPC (default: 4318, configurable)
    #[serde(default = "default_arrow_flight_port")]
    pub arrow_flight_port: u16,
}

impl Default for ProtocolConfig {
    fn default() -> Self {
        Self {
            protobuf_enabled: default_protobuf_enabled(),
            protobuf_port: default_protobuf_port(),
            arrow_flight_enabled: default_arrow_flight_enabled(),
            arrow_flight_port: default_arrow_flight_port(),
        }
    }
}

impl ProtocolConfig {
    /// Validate protocol configuration
    pub fn validate(&self) -> Result<(), OtlpConfigError> {
        // At least one protocol must be enabled
        if !self.protobuf_enabled && !self.arrow_flight_enabled {
            return Err(OtlpConfigError::ValidationFailed(
                "At least one protocol must be enabled".to_string(),
            ));
        }

        // Ports must be valid (1-65535)
        if self.protobuf_port == 0 || self.protobuf_port > 65535 {
            return Err(OtlpConfigError::ValidationFailed(
                "Protobuf port must be between 1 and 65535".to_string(),
            ));
        }

        if self.arrow_flight_port == 0 || self.arrow_flight_port > 65535 {
            return Err(OtlpConfigError::ValidationFailed(
                "Arrow Flight port must be between 1 and 65535".to_string(),
            ));
        }

        // Ports must be different if both protocols are enabled
        if self.protobuf_enabled && self.arrow_flight_enabled {
            if self.protobuf_port == self.arrow_flight_port {
                return Err(OtlpConfigError::ValidationFailed(
                    "Protobuf and Arrow Flight ports must be different when both protocols are enabled".to_string(),
                ));
            }
        }

        Ok(())
    }
}

fn default_protobuf_enabled() -> bool {
    true
}

fn default_protobuf_port() -> u16 {
    4317
}

fn default_arrow_flight_enabled() -> bool {
    true
}

fn default_arrow_flight_port() -> u16 {
    4318
}

/// Main configuration structure for the OTLP Arrow Library
///
/// This structure contains all configuration options for the library, including
/// output directory, write intervals, cleanup schedules, protocol settings, and
/// optional remote forwarding.
///
/// # Configuration Sources
///
/// Configuration can be loaded from:
/// - YAML files
/// - Environment variables (with `OTLP_*` prefix)
/// - Programmatic API (using `ConfigBuilder`)
///
/// # Default Values
///
/// - `output_dir`: `./output_dir`
/// - `write_interval_secs`: `5`
/// - `trace_cleanup_interval_secs`: `600` (10 minutes)
/// - `metric_cleanup_interval_secs`: `3600` (1 hour)
/// - `protocols`: Both Protobuf and Arrow Flight enabled by default
/// - `forwarding`: Disabled by default
///
/// # Example
///
/// ```no_run
/// use otlp_arrow_library::Config;
///
/// // Use defaults
/// let config = Config::default();
///
/// // Or use builder
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let config = otlp_arrow_library::ConfigBuilder::new()
///     .output_dir("./custom_output")
///     .write_interval_secs(10)
///     .build()?;
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Config {
    /// Output directory for Arrow IPC files (default: ./output_dir)
    #[serde(default = "default_output_dir")]
    pub output_dir: PathBuf,

    /// How frequently to write batches to disk in seconds (default: 5)
    #[serde(default = "default_write_interval_secs")]
    pub write_interval_secs: u64,

    /// How frequently to clean old trace files in seconds (default: 600)
    #[serde(default = "default_trace_cleanup_interval_secs")]
    pub trace_cleanup_interval_secs: u64,

    /// How frequently to clean old metric files in seconds (default: 3600)
    #[serde(default = "default_metric_cleanup_interval_secs")]
    pub metric_cleanup_interval_secs: u64,

    /// Protocol configuration (Protobuf and Arrow Flight)
    #[serde(default)]
    pub protocols: ProtocolConfig,

    /// Optional remote forwarding configuration
    #[serde(default)]
    pub forwarding: Option<ForwardingConfig>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            output_dir: default_output_dir(),
            write_interval_secs: default_write_interval_secs(),
            trace_cleanup_interval_secs: default_trace_cleanup_interval_secs(),
            metric_cleanup_interval_secs: default_metric_cleanup_interval_secs(),
            protocols: ProtocolConfig::default(),
            forwarding: None,
        }
    }
}

impl Config {
    /// Validate configuration values
    pub fn validate(&self) -> Result<(), OtlpConfigError> {
        // Validate output directory path
        if self.output_dir.to_string_lossy().is_empty() {
            return Err(OtlpConfigError::InvalidOutputDir(
                "Output directory cannot be empty".to_string(),
            ));
        }

        // Validate path is not too long (platform-specific, but 4096 is safe for most)
        let path_str = self.output_dir.to_string_lossy();
        if path_str.len() > 4096 {
            return Err(OtlpConfigError::InvalidOutputDir(format!(
                "Output directory path is too long ({} characters, max 4096)",
                path_str.len()
            )));
        }

        // Validate path doesn't contain null bytes (invalid on most systems)
        if path_str.contains('\0') {
            return Err(OtlpConfigError::InvalidOutputDir(
                "Output directory path cannot contain null bytes".to_string(),
            ));
        }

        // Validate write interval
        if self.write_interval_secs == 0 {
            return Err(OtlpConfigError::InvalidInterval(
                "Write interval must be greater than 0".to_string(),
            ));
        }

        // Validate write interval is reasonable (not too large - max 1 hour)
        if self.write_interval_secs > 3600 {
            return Err(OtlpConfigError::InvalidInterval(
                "Write interval must be less than 3600 seconds (1 hour)".to_string(),
            ));
        }

        // Validate cleanup intervals
        if self.trace_cleanup_interval_secs == 0 {
            return Err(OtlpConfigError::InvalidInterval(
                "Trace cleanup interval must be greater than 0".to_string(),
            ));
        }

        if self.metric_cleanup_interval_secs == 0 {
            return Err(OtlpConfigError::InvalidInterval(
                "Metric cleanup interval must be greater than 0".to_string(),
            ));
        }

        // Validate cleanup intervals are reasonable (< 1 day)
        if self.trace_cleanup_interval_secs > 86400 {
            return Err(OtlpConfigError::InvalidInterval(
                "Trace cleanup interval must be less than 86400 seconds (1 day)".to_string(),
            ));
        }

        if self.metric_cleanup_interval_secs > 86400 {
            return Err(OtlpConfigError::InvalidInterval(
                "Metric cleanup interval must be less than 86400 seconds (1 day)".to_string(),
            ));
        }

        // Validate cleanup intervals are not too small (minimum 60 seconds for cleanup)
        if self.trace_cleanup_interval_secs < 60 {
            return Err(OtlpConfigError::InvalidInterval(
                "Trace cleanup interval must be at least 60 seconds".to_string(),
            ));
        }

        if self.metric_cleanup_interval_secs < 60 {
            return Err(OtlpConfigError::InvalidInterval(
                "Metric cleanup interval must be at least 60 seconds".to_string(),
            ));
        }

        // Validate protocol configuration
        self.protocols.validate()?;

        // Validate forwarding configuration if enabled
        if let Some(ref forwarding) = self.forwarding {
            forwarding.validate()?;
        }

        Ok(())
    }
}

/// Configuration for optional remote OTLP endpoint forwarding
///
/// When enabled, the library will forward all received OTLP messages to a remote
/// endpoint in addition to storing them locally. Forwarding supports automatic format
/// conversion (Protobuf ↔ Arrow Flight) and various authentication methods.
///
/// # Features
///
/// - **Format Conversion**: Automatically converts between Protobuf and Arrow Flight formats
/// - **Authentication**: Supports API key, bearer token, and basic authentication
/// - **Resilience**: Forwarding failures do not affect local storage
/// - **Circuit Breaker**: Automatically stops forwarding after repeated failures
///
/// # Example
///
/// ```no_run
/// use otlp_arrow_library::config::{ForwardingConfig, ForwardingProtocol};
///
/// let forwarding = ForwardingConfig {
///     enabled: true,
///     endpoint_url: Some("https://collector.example.com:4317".to_string()),
///     protocol: ForwardingProtocol::Protobuf,
///     authentication: None,
/// };
/// ```
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ForwardingConfig {
    /// Whether forwarding is enabled (default: false)
    #[serde(default)]
    pub enabled: bool,

    /// Remote OTLP endpoint URL (required if enabled)
    pub endpoint_url: Option<String>,

    /// Protocol to use for forwarding (Protobuf or Arrow Flight, default: Protobuf)
    #[serde(default)]
    pub protocol: ForwardingProtocol,

    /// Authentication configuration (optional)
    #[serde(default)]
    pub authentication: Option<AuthConfig>,
}

impl Default for ForwardingConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            endpoint_url: None,
            protocol: ForwardingProtocol::default(),
            authentication: None,
        }
    }
}

impl ForwardingConfig {
    /// Validate forwarding configuration
    pub fn validate(&self) -> Result<(), OtlpConfigError> {
        if self.enabled {
            if let Some(ref url) = self.endpoint_url {
                if url.is_empty() {
                    return Err(OtlpConfigError::InvalidUrl(
                        "Endpoint URL cannot be empty when forwarding is enabled".to_string(),
                    ));
                }

                // Validate URL format
                if !url.starts_with("http://") && !url.starts_with("https://") {
                    return Err(OtlpConfigError::InvalidUrl(
                        "Endpoint URL must use http:// or https:// scheme".to_string(),
                    ));
                }
            } else {
                return Err(OtlpConfigError::MissingRequiredField(
                    "endpoint_url is required when forwarding is enabled".to_string(),
                ));
            }
        }

        Ok(())
    }
}

/// Authentication configuration for remote forwarding
///
/// Specifies the authentication method and credentials to use when forwarding
/// messages to remote OTLP endpoints.
///
/// # Supported Authentication Types
///
/// - **`api_key`**: API key authentication with custom header name
///   - Required credentials: `key`
///   - Optional credentials: `header_name` (default: `X-API-Key`)
///
/// - **`bearer_token`**: Bearer token authentication
///   - Required credentials: `token`
///
/// - **`basic`**: HTTP Basic authentication
///   - Required credentials: `username`, `password`
///
/// # Example
///
/// ```no_run
/// use otlp_arrow_library::AuthConfig;
/// use std::collections::HashMap;
///
/// let mut credentials = HashMap::new();
/// credentials.insert("token".to_string(), "my-bearer-token".to_string());
///
/// let auth = AuthConfig {
///     auth_type: "bearer_token".to_string(),
///     credentials,
/// };
/// ```
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AuthConfig {
    /// Type of authentication (e.g., "api_key", "bearer_token", "basic")
    pub auth_type: String,

    /// Authentication parameters (e.g., token, key, username, password)
    pub credentials: HashMap<String, String>,
}

impl AuthConfig {
    /// Validate authentication configuration
    pub fn validate(&self) -> Result<(), OtlpConfigError> {
        if self.auth_type.is_empty() {
            return Err(OtlpConfigError::ValidationFailed(
                "Authentication type cannot be empty".to_string(),
            ));
        }

        // Validate required credentials based on auth type
        match self.auth_type.as_str() {
            "api_key" | "bearer_token" => {
                if !self.credentials.contains_key("token")
                    && !self.credentials.contains_key("api_key")
                {
                    return Err(OtlpConfigError::MissingRequiredField(format!(
                        "token or api_key required for {}",
                        self.auth_type
                    )));
                }
            }
            "basic" => {
                if !self.credentials.contains_key("username")
                    || !self.credentials.contains_key("password")
                {
                    return Err(OtlpConfigError::MissingRequiredField(
                        "username and password required for basic auth".to_string(),
                    ));
                }
            }
            _ => {
                return Err(OtlpConfigError::ValidationFailed(format!(
                    "Unsupported authentication type: {}",
                    self.auth_type
                )));
            }
        }

        Ok(())
    }
}

/// Builder for creating configurations programmatically
#[derive(Debug, Default)]
pub struct ConfigBuilder {
    config: Config,
}

impl ConfigBuilder {
    /// Create a new configuration builder
    pub fn new() -> Self {
        Self {
            config: Config::default(),
        }
    }

    /// Set output directory
    pub fn output_dir(mut self, dir: impl Into<PathBuf>) -> Self {
        self.config.output_dir = dir.into();
        self
    }

    /// Set write interval in seconds
    pub fn write_interval_secs(mut self, secs: u64) -> Self {
        self.config.write_interval_secs = secs;
        self
    }

    /// Set trace cleanup interval in seconds
    pub fn trace_cleanup_interval_secs(mut self, secs: u64) -> Self {
        self.config.trace_cleanup_interval_secs = secs;
        self
    }

    /// Set metric cleanup interval in seconds
    pub fn metric_cleanup_interval_secs(mut self, secs: u64) -> Self {
        self.config.metric_cleanup_interval_secs = secs;
        self
    }

    /// Set protocol configuration
    pub fn protocols(mut self, protocols: ProtocolConfig) -> Self {
        self.config.protocols = protocols;
        self
    }

    /// Enable or disable Protobuf protocol
    pub fn protobuf_enabled(mut self, enabled: bool) -> Self {
        self.config.protocols.protobuf_enabled = enabled;
        self
    }

    /// Set Protobuf port
    pub fn protobuf_port(mut self, port: u16) -> Self {
        self.config.protocols.protobuf_port = port;
        self
    }

    /// Enable or disable Arrow Flight protocol
    pub fn arrow_flight_enabled(mut self, enabled: bool) -> Self {
        self.config.protocols.arrow_flight_enabled = enabled;
        self
    }

    /// Set Arrow Flight port
    pub fn arrow_flight_port(mut self, port: u16) -> Self {
        self.config.protocols.arrow_flight_port = port;
        self
    }

    /// Enable forwarding with configuration
    pub fn enable_forwarding(mut self, forwarding: ForwardingConfig) -> Self {
        self.config.forwarding = Some(forwarding);
        self
    }

    /// Set forwarding configuration (convenience method)
    pub fn forwarding(mut self, forwarding: Option<ForwardingConfig>) -> Self {
        self.config.forwarding = forwarding;
        self
    }

    /// Build the configuration with validation
    pub fn build(self) -> Result<Config, OtlpConfigError> {
        self.config.validate()?;
        Ok(self.config)
    }
}

// Default value functions
fn default_output_dir() -> PathBuf {
    PathBuf::from("./output_dir")
}

fn default_write_interval_secs() -> u64 {
    5
}

fn default_trace_cleanup_interval_secs() -> u64 {
    600
}

fn default_metric_cleanup_interval_secs() -> u64 {
    3600
}
