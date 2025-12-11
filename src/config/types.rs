//! Configuration type definitions
//!
//! Defines all configuration structures for the OTLP Arrow Library.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

use crate::error::OtlpConfigError;

// Import SecretString for secure credential storage
use secrecy::SecretString;

// Import url crate for comprehensive URL validation
use url::Url;

/// Protocol to use for forwarding messages to remote endpoints
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ForwardingProtocol {
    /// Standard OTLP gRPC with Protobuf
    #[default]
    Protobuf,
    /// OpenTelemetry Protocol with Apache Arrow (OTAP)
    ArrowFlight,
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

    /// Whether SDK ResourceMetrics extraction is enabled (default: true)
    ///
    /// When enabled, allows extraction of metric data from SDK ResourceMetrics
    /// via opentelemetry-otlp exporter. This requires creating a temporary gRPC
    /// server and adds overhead. Disable if you only use gRPC ingestion path
    /// which already preserves protobuf format.
    #[serde(default = "default_sdk_extraction_enabled")]
    pub sdk_extraction_enabled: bool,
}

impl Default for ProtocolConfig {
    fn default() -> Self {
        Self {
            protobuf_enabled: default_protobuf_enabled(),
            protobuf_port: default_protobuf_port(),
            arrow_flight_enabled: default_arrow_flight_enabled(),
            arrow_flight_port: default_arrow_flight_port(),
            sdk_extraction_enabled: default_sdk_extraction_enabled(),
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
        if self.protobuf_port == 0 {
            return Err(OtlpConfigError::ValidationFailed(
                "Protobuf port must be between 1 and 65535".to_string(),
            ));
        }

        if self.arrow_flight_port == 0 {
            return Err(OtlpConfigError::ValidationFailed(
                "Arrow Flight port must be between 1 and 65535".to_string(),
            ));
        }

        // Ports must be different if both protocols are enabled
        if self.protobuf_enabled
            && self.arrow_flight_enabled
            && self.protobuf_port == self.arrow_flight_port
        {
            return Err(OtlpConfigError::ValidationFailed(
                "Protobuf and Arrow Flight ports must be different when both protocols are enabled"
                    .to_string(),
            ));
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

fn default_sdk_extraction_enabled() -> bool {
    true
}

/// Configuration for dashboard HTTP server
///
/// Controls whether the Rust service serves the dashboard static files via HTTP.
/// When enabled, the service starts an HTTP server on the specified port
/// to serve static files from the configured directory.
///
/// # Default Values
///
/// - `enabled`: `false` (disabled by default)
/// - `port`: `8080` (when enabled)
/// - `static_dir`: `"./dashboard/dist"`
///
/// # Example
///
/// ```no_run
/// use otlp_arrow_library::config::DashboardConfig;
/// use std::path::PathBuf;
///
/// let dashboard = DashboardConfig {
///     enabled: true,
///     port: 8080,
///     static_dir: PathBuf::from("./dashboard/dist"),
///     bind_address: "127.0.0.1".to_string(), // or "0.0.0.0" for network access
///     x_frame_options: None, // Optional: Some("DENY".to_string()) or Some("SAMEORIGIN".to_string())
/// };
/// ```
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DashboardConfig {
    /// Whether dashboard HTTP server is enabled (default: false)
    #[serde(default = "default_dashboard_enabled")]
    pub enabled: bool,

    /// Port for dashboard HTTP server (default: 8080)
    #[serde(default = "default_dashboard_port")]
    pub port: u16,

    /// Directory containing dashboard static files (default: ./dashboard/dist)
    #[serde(default = "default_dashboard_static_dir")]
    pub static_dir: PathBuf,

    /// Bind address for dashboard HTTP server (default: 127.0.0.1 for local-only access)
    /// Use 0.0.0.0 to allow network access from other machines
    #[serde(default = "default_dashboard_bind_address")]
    pub bind_address: String,

    /// X-Frame-Options header value (default: "DENY")
    /// Set to "SAMEORIGIN" to allow embedding in iframes from same origin
    /// Only used if Some, otherwise defaults to "DENY" in server
    #[serde(default)]
    pub x_frame_options: Option<String>,
}

impl Default for DashboardConfig {
    fn default() -> Self {
        Self {
            enabled: default_dashboard_enabled(),
            port: default_dashboard_port(),
            static_dir: default_dashboard_static_dir(),
            bind_address: default_dashboard_bind_address(),
            x_frame_options: None, // Default to None, server will use "DENY"
        }
    }
}

impl DashboardConfig {
    /// Validate dashboard configuration
    pub fn validate(&self) -> Result<(), OtlpConfigError> {
        if self.enabled {
            // Validate port (u16 is already 0-65535, so just check for 0)
            if self.port == 0 {
                return Err(OtlpConfigError::ValidationFailed(
                    "Dashboard port must be between 1 and 65535".to_string(),
                ));
            }

            // Validate port doesn't conflict with gRPC ports
            if self.port == 4317 || self.port == 4318 {
                return Err(OtlpConfigError::ValidationFailed(
                    "Dashboard port conflicts with gRPC port (4317 or 4318)".to_string(),
                ));
            }

            // Validate bind address is a valid IP address format
            if !self.bind_address.is_empty() {
                // Basic validation: should be a valid IP address (127.0.0.1, 0.0.0.0, or IPv6)
                if self.bind_address.parse::<std::net::IpAddr>().is_err() {
                    return Err(OtlpConfigError::ValidationFailed(format!(
                        "Dashboard bind_address must be a valid IP address: {}",
                        self.bind_address
                    )));
                }
            }

            // Validate static directory exists when enabled
            if !self.static_dir.exists() {
                return Err(OtlpConfigError::InvalidOutputDir(format!(
                    "Dashboard static directory does not exist: {}",
                    self.static_dir.display()
                )));
            }

            if !self.static_dir.is_dir() {
                return Err(OtlpConfigError::InvalidOutputDir(format!(
                    "Dashboard static directory is not a directory: {}",
                    self.static_dir.display()
                )));
            }

            // Validate x_frame_options if provided
            if let Some(ref xfo) = self.x_frame_options
                && xfo != "DENY"
                && xfo != "SAMEORIGIN"
            {
                return Err(OtlpConfigError::ValidationFailed(format!(
                    "x_frame_options must be 'DENY' or 'SAMEORIGIN' (got: {})",
                    xfo
                )));
            }
        }

        Ok(())
    }
}

fn default_dashboard_enabled() -> bool {
    false
}

fn default_dashboard_port() -> u16 {
    8080
}

fn default_dashboard_static_dir() -> PathBuf {
    PathBuf::from("./dashboard/dist")
}

fn default_dashboard_bind_address() -> String {
    "127.0.0.1".to_string()
}

/// Main configuration structure for the OTLP Arrow Library
///
/// This structure contains all configuration options for the library, including
/// output directory, write intervals, cleanup schedules, protocol settings,
/// optional remote forwarding, and dashboard HTTP server.
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
/// - `dashboard`: Disabled by default
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

    /// Maximum number of trace spans to buffer in memory (default: 10000)
    #[serde(default = "default_max_trace_buffer_size")]
    pub max_trace_buffer_size: usize,

    /// Maximum number of metric requests to buffer in memory (default: 10000)
    #[serde(default = "default_max_metric_buffer_size")]
    pub max_metric_buffer_size: usize,

    /// Protocol configuration (Protobuf and Arrow Flight)
    #[serde(default)]
    pub protocols: ProtocolConfig,

    /// Optional remote forwarding configuration
    #[serde(default)]
    pub forwarding: Option<ForwardingConfig>,

    /// Dashboard HTTP server configuration
    #[serde(default)]
    pub dashboard: DashboardConfig,

    /// Temporality mode for metric exporters (default: Cumulative)
    ///
    /// This setting controls how metrics are aggregated:
    /// - Cumulative: Metrics accumulate values over time (default, backward compatible)
    /// - Delta: Metrics represent changes since last export
    ///
    /// Note: Temporality is not serializable, so this field is skipped during serialization.
    /// It can only be set programmatically via ConfigBuilder.
    #[serde(skip)]
    pub metric_temporality: Option<opentelemetry_sdk::metrics::Temporality>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            output_dir: default_output_dir(),
            write_interval_secs: default_write_interval_secs(),
            trace_cleanup_interval_secs: default_trace_cleanup_interval_secs(),
            metric_cleanup_interval_secs: default_metric_cleanup_interval_secs(),
            max_trace_buffer_size: default_max_trace_buffer_size(),
            max_metric_buffer_size: default_max_metric_buffer_size(),
            protocols: ProtocolConfig::default(),
            forwarding: None,
            dashboard: DashboardConfig::default(),
            metric_temporality: None, // Defaults to Cumulative (handled in exporter)
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

        // Validate buffer size limits
        if self.max_trace_buffer_size == 0 || self.max_trace_buffer_size > 1_000_000 {
            return Err(OtlpConfigError::ValidationFailed(format!(
                "max_trace_buffer_size must be between 1 and 1,000,000 (got {})",
                self.max_trace_buffer_size
            )));
        }
        if self.max_metric_buffer_size == 0 || self.max_metric_buffer_size > 1_000_000 {
            return Err(OtlpConfigError::ValidationFailed(format!(
                "max_metric_buffer_size must be between 1 and 1,000,000 (got {})",
                self.max_metric_buffer_size
            )));
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

        // Validate dashboard configuration
        self.dashboard.validate()?;

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
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
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

                // Comprehensive URL validation using url crate
                let parsed_url = Url::parse(url).map_err(|e| {
                    OtlpConfigError::InvalidUrl(format!(
                        "Invalid endpoint URL format: {} (error: {})",
                        url, e
                    ))
                })?;

                // Validate scheme (must be http or https)
                match parsed_url.scheme() {
                    "http" | "https" => {}
                    scheme => {
                        return Err(OtlpConfigError::InvalidUrl(format!(
                            "Endpoint URL must use http or https scheme (got: {}): {}",
                            scheme, url
                        )));
                    }
                }

                // Validate host is present
                if parsed_url.host().is_none() {
                    return Err(OtlpConfigError::InvalidUrl(format!(
                        "Endpoint URL must include a host: {}",
                        url
                    )));
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
/// use secrecy::SecretString;
/// use std::collections::HashMap;
///
/// let mut credentials = HashMap::new();
/// credentials.insert("token".to_string(), SecretString::new("my-bearer-token".to_string()));
///
/// let auth = AuthConfig {
///     auth_type: "bearer_token".to_string(),
///     credentials,
/// };
/// ```
///
/// # Breaking Changes
///
/// As of version 0.4.0, `credentials` values are `SecretString` instead of `String`.
/// When creating `AuthConfig` programmatically, use `SecretString::new()`.
/// YAML and environment variable loading automatically converts strings to `SecretString`.
///
/// **Security Note**: Credentials are stored using `SecretString` to prevent exposure in logs,
/// error messages, or memory dumps. Credentials are zeroed in memory when dropped.
/// Credentials are NOT serialized for security reasons - they must be provided via environment
/// variables or programmatic configuration.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AuthConfig {
    /// Type of authentication (e.g., "api_key", "bearer_token", "basic")
    pub auth_type: String,

    /// Authentication parameters (e.g., token, key, username, password)
    /// Stored as SecretString to prevent exposure in logs, errors, or memory dumps.
    ///
    /// Required credentials by auth type:
    /// - `api_key`: requires `key` credential
    /// - `bearer_token`: requires `token` credential  
    /// - `basic`: requires `username` and `password` credentials
    ///
    /// **Security**: Credentials are NOT serialized (skipped during serialization) to prevent
    /// accidental exposure in configuration files or logs.
    #[serde(skip_serializing, deserialize_with = "deserialize_secret_credentials")]
    pub credentials: HashMap<String, SecretString>,
}

/// Custom deserializer for credentials that converts String to SecretString
fn deserialize_secret_credentials<'de, D>(
    deserializer: D,
) -> Result<HashMap<String, SecretString>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::Deserialize;
    let map: HashMap<String, String> = HashMap::deserialize(deserializer)?;
    Ok(map
        .into_iter()
        .map(|(k, v)| (k, SecretString::new(v)))
        .collect())
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
            "api_key" => {
                if !self.credentials.contains_key("key") {
                    return Err(OtlpConfigError::MissingRequiredField(
                        "key required for api_key authentication".to_string(),
                    ));
                }
            }
            "bearer_token" => {
                if !self.credentials.contains_key("token") {
                    return Err(OtlpConfigError::MissingRequiredField(
                        "token required for bearer_token authentication".to_string(),
                    ));
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

    /// Set maximum trace buffer size
    pub fn max_trace_buffer_size(mut self, size: usize) -> Self {
        self.config.max_trace_buffer_size = size;
        self
    }

    /// Set maximum metric buffer size
    pub fn max_metric_buffer_size(mut self, size: usize) -> Self {
        self.config.max_metric_buffer_size = size;
        self
    }

    /// Set metric temporality (Cumulative or Delta)
    ///
    /// Defaults to Cumulative if not specified (for backward compatibility).
    pub fn with_temporality(
        mut self,
        temporality: opentelemetry_sdk::metrics::Temporality,
    ) -> Self {
        self.config.metric_temporality = Some(temporality);
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

    /// Set dashboard configuration
    pub fn dashboard(mut self, dashboard: DashboardConfig) -> Self {
        self.config.dashboard = dashboard;
        self
    }

    /// Enable or disable dashboard
    pub fn dashboard_enabled(mut self, enabled: bool) -> Self {
        self.config.dashboard.enabled = enabled;
        self
    }

    /// Set dashboard port
    pub fn dashboard_port(mut self, port: u16) -> Self {
        self.config.dashboard.port = port;
        self
    }

    /// Set dashboard static directory
    pub fn dashboard_static_dir(mut self, dir: impl Into<PathBuf>) -> Self {
        self.config.dashboard.static_dir = dir.into();
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

fn default_max_trace_buffer_size() -> usize {
    10000
}

fn default_max_metric_buffer_size() -> usize {
    10000
}
