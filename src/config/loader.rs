//! Configuration loader
//!
//! Loads configuration from YAML files, environment variables, or programmatic API.
//! Priority: provided config > environment variables > defaults

use std::env;
use std::path::PathBuf;

use crate::config::types::Config;
use crate::error::OtlpConfigError;
use tracing::{debug, info, warn};

/// Configuration loader
pub struct ConfigLoader;

impl ConfigLoader {
    /// Load configuration from YAML file
    pub fn from_yaml(path: impl AsRef<std::path::Path>) -> Result<Config, OtlpConfigError> {
        let path = path.as_ref();
        info!(
            config_path = %path.display(),
            "Loading configuration from YAML file"
        );

        let content = std::fs::read_to_string(path)
            .map_err(|e| {
                warn!(
                    config_path = %path.display(),
                    error = %e,
                    "Failed to read configuration file"
                );
                OtlpConfigError::InvalidOutputDir(format!("Failed to read config file: {}", e))
            })?;

        debug!(
            config_path = %path.display(),
            file_size_bytes = content.len(),
            "Read configuration file"
        );

        let mut config: Config = serde_yaml::from_str(&content)
            .map_err(|e| {
                warn!(
                    config_path = %path.display(),
                    error = %e,
                    "Failed to parse YAML configuration"
                );
                OtlpConfigError::ValidationFailed(format!("Failed to parse YAML: {}", e))
            })?;

        debug!(
            config_path = %path.display(),
            "Parsed YAML configuration successfully"
        );

        // Apply environment variable overrides
        Self::apply_env_overrides(&mut config);

        debug!(
            config_path = %path.display(),
            "Applied environment variable overrides"
        );

        // Validate configuration
        config.validate().map_err(|e| {
            warn!(
                config_path = %path.display(),
                error = %e,
                "Configuration validation failed"
            );
            e
        })?;

        info!(
            config_path = %path.display(),
            output_dir = %config.output_dir.display(),
            write_interval_secs = config.write_interval_secs,
            protobuf_enabled = config.protocols.protobuf_enabled,
            arrow_flight_enabled = config.protocols.arrow_flight_enabled,
            "Configuration loaded and validated successfully"
        );

        Ok(config)
    }

    /// Load configuration from environment variables
    pub fn from_env() -> Result<Config, OtlpConfigError> {
        info!("Loading configuration from environment variables");

        let mut config = Config::default();

        debug!(
            output_dir = %config.output_dir.display(),
            write_interval_secs = config.write_interval_secs,
            "Starting with default configuration"
        );

        // Apply environment variable overrides
        Self::apply_env_overrides(&mut config);

        debug!("Applied environment variable overrides");

        // Validate configuration
        config.validate().map_err(|e| {
            warn!(
                error = %e,
                "Configuration validation failed"
            );
            e
        })?;

        info!(
            output_dir = %config.output_dir.display(),
            write_interval_secs = config.write_interval_secs,
            protobuf_enabled = config.protocols.protobuf_enabled,
            arrow_flight_enabled = config.protocols.arrow_flight_enabled,
            "Configuration loaded from environment variables and validated successfully"
        );

        Ok(config)
    }

    /// Load configuration with priority: provided config > environment variables > defaults
    pub fn load(provided: Option<Config>) -> Result<Config, OtlpConfigError> {
        if provided.is_some() {
            info!("Loading configuration with provided config and environment variable overrides");
        } else {
            info!("Loading configuration with defaults and environment variable overrides");
        }

        let mut config = provided.unwrap_or_else(Config::default);

        debug!(
            output_dir = %config.output_dir.display(),
            write_interval_secs = config.write_interval_secs,
            "Starting configuration"
        );

        // Apply environment variable overrides (they override provided config)
        Self::apply_env_overrides(&mut config);

        debug!("Applied environment variable overrides");

        // Validate configuration
        config.validate().map_err(|e| {
            warn!(
                error = %e,
                "Configuration validation failed"
            );
            e
        })?;

        info!(
            output_dir = %config.output_dir.display(),
            write_interval_secs = config.write_interval_secs,
            protobuf_enabled = config.protocols.protobuf_enabled,
            arrow_flight_enabled = config.protocols.arrow_flight_enabled,
            "Configuration loaded and validated successfully"
        );

        Ok(config)
    }

    /// Apply environment variable overrides to configuration
    fn apply_env_overrides(config: &mut Config) {
        // OTLP_OUTPUT_DIR
        if let Ok(dir) = env::var("OTLP_OUTPUT_DIR") {
            debug!(
                env_var = "OTLP_OUTPUT_DIR",
                value = %dir,
                "Applying environment variable override"
            );
            config.output_dir = PathBuf::from(dir);
        }

        // OTLP_WRITE_INTERVAL_SECS
        if let Ok(interval) = env::var("OTLP_WRITE_INTERVAL_SECS") {
            match interval.parse::<u64>() {
                Ok(secs) => {
                    debug!(
                        env_var = "OTLP_WRITE_INTERVAL_SECS",
                        value = secs,
                        "Applying environment variable override"
                    );
                    config.write_interval_secs = secs;
                }
                Err(e) => {
                    warn!(
                        env_var = "OTLP_WRITE_INTERVAL_SECS",
                        value = %interval,
                        error = %e,
                        "Failed to parse environment variable, using default"
                    );
                }
            }
        }

        // OTLP_TRACE_CLEANUP_INTERVAL_SECS
        if let Ok(interval) = env::var("OTLP_TRACE_CLEANUP_INTERVAL_SECS") {
            if let Ok(secs) = interval.parse::<u64>() {
                config.trace_cleanup_interval_secs = secs;
            }
        }

        // OTLP_METRIC_CLEANUP_INTERVAL_SECS
        if let Ok(interval) = env::var("OTLP_METRIC_CLEANUP_INTERVAL_SECS") {
            if let Ok(secs) = interval.parse::<u64>() {
                config.metric_cleanup_interval_secs = secs;
            }
        }

        // OTLP_PROTOBUF_ENABLED
        if let Ok(enabled) = env::var("OTLP_PROTOBUF_ENABLED") {
            match enabled.parse::<bool>() {
                Ok(val) => {
                    debug!(
                        env_var = "OTLP_PROTOBUF_ENABLED",
                        value = val,
                        "Applying environment variable override"
                    );
                    config.protocols.protobuf_enabled = val;
                }
                Err(e) => {
                    warn!(
                        env_var = "OTLP_PROTOBUF_ENABLED",
                        value = %enabled,
                        error = %e,
                        "Failed to parse environment variable, using default"
                    );
                }
            }
        }

        // OTLP_PROTOBUF_PORT
        if let Ok(port) = env::var("OTLP_PROTOBUF_PORT") {
            match port.parse::<u16>() {
                Ok(p) => {
                    debug!(
                        env_var = "OTLP_PROTOBUF_PORT",
                        value = p,
                        "Applying environment variable override"
                    );
                    config.protocols.protobuf_port = p;
                }
                Err(e) => {
                    warn!(
                        env_var = "OTLP_PROTOBUF_PORT",
                        value = %port,
                        error = %e,
                        "Failed to parse environment variable, using default"
                    );
                }
            }
        }

        // OTLP_ARROW_FLIGHT_ENABLED
        if let Ok(enabled) = env::var("OTLP_ARROW_FLIGHT_ENABLED") {
            match enabled.parse::<bool>() {
                Ok(val) => {
                    debug!(
                        env_var = "OTLP_ARROW_FLIGHT_ENABLED",
                        value = val,
                        "Applying environment variable override"
                    );
                    config.protocols.arrow_flight_enabled = val;
                }
                Err(e) => {
                    warn!(
                        env_var = "OTLP_ARROW_FLIGHT_ENABLED",
                        value = %enabled,
                        error = %e,
                        "Failed to parse environment variable, using default"
                    );
                }
            }
        }

        // OTLP_ARROW_FLIGHT_PORT
        if let Ok(port) = env::var("OTLP_ARROW_FLIGHT_PORT") {
            match port.parse::<u16>() {
                Ok(p) => {
                    debug!(
                        env_var = "OTLP_ARROW_FLIGHT_PORT",
                        value = p,
                        "Applying environment variable override"
                    );
                    config.protocols.arrow_flight_port = p;
                }
                Err(e) => {
                    warn!(
                        env_var = "OTLP_ARROW_FLIGHT_PORT",
                        value = %port,
                        error = %e,
                        "Failed to parse environment variable, using default"
                    );
                }
            }
        }

        // OTLP_FORWARDING_ENABLED
        if let Ok(enabled) = env::var("OTLP_FORWARDING_ENABLED") {
            if enabled.parse::<bool>().unwrap_or(false) {
                use crate::config::types::ForwardingConfig;
                let mut forwarding = config.forwarding.take().unwrap_or_else(ForwardingConfig::default);
                forwarding.enabled = true;

                // OTLP_FORWARDING_ENDPOINT_URL
                if let Ok(url) = env::var("OTLP_FORWARDING_ENDPOINT_URL") {
                    forwarding.endpoint_url = Some(url);
                }

                // OTLP_FORWARDING_PROTOCOL
                if let Ok(protocol) = env::var("OTLP_FORWARDING_PROTOCOL") {
                    use crate::config::types::ForwardingProtocol;
                    forwarding.protocol = match protocol.to_lowercase().as_str() {
                        "protobuf" => ForwardingProtocol::Protobuf,
                        "arrow_flight" | "arrowflight" => ForwardingProtocol::ArrowFlight,
                        _ => ForwardingProtocol::default(),
                    };
                }

                config.forwarding = Some(forwarding);
            }
        }
    }
}

