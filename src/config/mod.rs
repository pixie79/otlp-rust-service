//! Configuration module
//!
//! Provides configuration management for the OTLP Arrow Library including
//! loading from YAML files, environment variables, and programmatic API.

pub mod loader;
pub mod types;

pub use loader::ConfigLoader;
pub use types::{
    AuthConfig, Config, ConfigBuilder, ForwardingConfig, ForwardingProtocol, ProtocolConfig,
};
