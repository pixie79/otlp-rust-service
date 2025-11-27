//! Unit tests for YAML configuration loading

use otlp_arrow_library::config::{Config, ConfigLoader};
use otlp_arrow_library::error::OtlpConfigError;
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

#[test]
fn test_load_valid_yaml_config() {
    let temp_dir = TempDir::new().unwrap();
    let config_file = temp_dir.path().join("config.yaml");
    
    let yaml_content = r#"
output_dir: /tmp/test_output
write_interval_secs: 10
trace_cleanup_interval_secs: 300
metric_cleanup_interval_secs: 1800
protocols:
  protobuf_enabled: true
  protobuf_port: 4317
  arrow_flight_enabled: true
  arrow_flight_port: 4318
"#;
    
    fs::write(&config_file, yaml_content).unwrap();
    
    let config = ConfigLoader::from_yaml(&config_file).unwrap();
    
    assert_eq!(config.output_dir, PathBuf::from("/tmp/test_output"));
    assert_eq!(config.write_interval_secs, 10);
    assert_eq!(config.trace_cleanup_interval_secs, 300);
    assert_eq!(config.metric_cleanup_interval_secs, 1800);
    assert!(config.protocols.protobuf_enabled);
    assert_eq!(config.protocols.protobuf_port, 4317);
    assert!(config.protocols.arrow_flight_enabled);
    assert_eq!(config.protocols.arrow_flight_port, 4318);
}

#[test]
fn test_load_yaml_with_defaults() {
    let temp_dir = TempDir::new().unwrap();
    let config_file = temp_dir.path().join("config.yaml");
    
    // Minimal YAML with only required fields
    let yaml_content = r#"
output_dir: /tmp/test_output
"#;
    
    fs::write(&config_file, yaml_content).unwrap();
    
    let config = ConfigLoader::from_yaml(&config_file).unwrap();
    
    // Should use defaults for unspecified fields
    assert_eq!(config.write_interval_secs, 5); // default
    assert_eq!(config.trace_cleanup_interval_secs, 600); // default
    assert_eq!(config.metric_cleanup_interval_secs, 3600); // default
    assert!(config.protocols.protobuf_enabled); // default
    assert_eq!(config.protocols.protobuf_port, 4317); // default
}

#[test]
fn test_load_yaml_with_invalid_syntax() {
    let temp_dir = TempDir::new().unwrap();
    let config_file = temp_dir.path().join("config.yaml");
    
    let invalid_yaml = r#"
output_dir: /tmp/test_output
write_interval_secs: [invalid
"#;
    
    fs::write(&config_file, invalid_yaml).unwrap();
    
    let result = ConfigLoader::from_yaml(&config_file);
    assert!(result.is_err());
    match result.unwrap_err() {
        OtlpConfigError::ValidationFailed(_) => {},
        _ => panic!("Expected ValidationFailed error"),
    }
}

#[test]
fn test_load_yaml_with_missing_file() {
    let result = ConfigLoader::from_yaml("/nonexistent/path/config.yaml");
    assert!(result.is_err());
    match result.unwrap_err() {
        OtlpConfigError::InvalidOutputDir(_) => {},
        _ => panic!("Expected InvalidOutputDir error for missing file"),
    }
}

#[test]
fn test_load_yaml_with_env_overrides() {
    let temp_dir = TempDir::new().unwrap();
    let config_file = temp_dir.path().join("config.yaml");
    
    let yaml_content = r#"
output_dir: /tmp/test_output
write_interval_secs: 10
"#;
    
    fs::write(&config_file, yaml_content).unwrap();
    
    // Set environment variable to override YAML
    unsafe {
        std::env::set_var("OTLP_WRITE_INTERVAL_SECS", "15");
    }
    
    let config = ConfigLoader::from_yaml(&config_file).unwrap();
    
    // Environment variable should override YAML
    assert_eq!(config.write_interval_secs, 15);
    
    // Clean up
    unsafe {
        std::env::remove_var("OTLP_WRITE_INTERVAL_SECS");
    }
}

#[test]
fn test_load_yaml_with_forwarding_config() {
    let temp_dir = TempDir::new().unwrap();
    let config_file = temp_dir.path().join("config.yaml");
    
    let yaml_content = r#"
output_dir: /tmp/test_output
forwarding:
  enabled: true
  endpoint_url: "https://example.com/otlp"
  protocol: protobuf
  authentication:
    auth_type: bearer_token
    credentials:
      token: "secret-token"
"#;
    
    fs::write(&config_file, yaml_content).unwrap();
    
    let config = ConfigLoader::from_yaml(&config_file).unwrap();
    
    assert!(config.forwarding.is_some());
    let forwarding = config.forwarding.as_ref().unwrap();
    assert!(forwarding.enabled);
    assert_eq!(forwarding.endpoint_url.as_ref().unwrap(), "https://example.com/otlp");
    assert!(forwarding.authentication.is_some());
}

