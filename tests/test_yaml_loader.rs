//! Unit tests for YAML configuration loading

use otlp_arrow_library::config::ConfigLoader;
use otlp_arrow_library::error::OtlpConfigError;
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;
use tempfile::TempDir;

// Mutex to serialize environment variable access across parallel tests
// Environment variables are process-wide, so parallel tests can interfere with each other
static ENV_MUTEX: Mutex<()> = Mutex::new(());

/// Helper function to clear all OTLP-related environment variables
fn clear_otlp_env_vars() {
    std::env::remove_var("OTLP_OUTPUT_DIR");
    std::env::remove_var("OTLP_WRITE_INTERVAL_SECS");
    std::env::remove_var("OTLP_TRACE_CLEANUP_INTERVAL_SECS");
    std::env::remove_var("OTLP_METRIC_CLEANUP_INTERVAL_SECS");
    std::env::remove_var("OTLP_PROTOBUF_ENABLED");
    std::env::remove_var("OTLP_PROTOBUF_PORT");
    std::env::remove_var("OTLP_ARROW_FLIGHT_ENABLED");
    std::env::remove_var("OTLP_ARROW_FLIGHT_PORT");
    std::env::remove_var("OTLP_FORWARDING_ENABLED");
    std::env::remove_var("OTLP_FORWARDING_ENDPOINT_URL");
    std::env::remove_var("OTLP_FORWARDING_PROTOCOL");
}

#[test]
fn test_load_valid_yaml_config() {
    let _guard = ENV_MUTEX.lock().unwrap();
    clear_otlp_env_vars();
    
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
    let _guard = ENV_MUTEX.lock().unwrap();
    clear_otlp_env_vars();
    
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
    let _guard = ENV_MUTEX.lock().unwrap();
    clear_otlp_env_vars();
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
    let _guard = ENV_MUTEX.lock().unwrap();
    clear_otlp_env_vars();
    let result = ConfigLoader::from_yaml("/nonexistent/path/config.yaml");
    assert!(result.is_err());
    match result.unwrap_err() {
        OtlpConfigError::InvalidOutputDir(_) => {},
        _ => panic!("Expected InvalidOutputDir error for missing file"),
    }
}

#[test]
fn test_load_yaml_with_env_overrides() {
    let _guard = ENV_MUTEX.lock().unwrap();
    clear_otlp_env_vars();
    let temp_dir = TempDir::new().unwrap();
    let config_file = temp_dir.path().join("config.yaml");
    
    let yaml_content = r#"
output_dir: /tmp/test_output
write_interval_secs: 10
"#;
    
    fs::write(&config_file, yaml_content).unwrap();
    
    // Set environment variable to override YAML
    std::env::set_var("OTLP_WRITE_INTERVAL_SECS", "15");
    
    let config = ConfigLoader::from_yaml(&config_file).unwrap();
    
    // Environment variable should override YAML
    assert_eq!(config.write_interval_secs, 15);
    
    // Clean up
    std::env::remove_var("OTLP_WRITE_INTERVAL_SECS");
}

#[test]
fn test_load_yaml_with_forwarding_config() {
    let _guard = ENV_MUTEX.lock().unwrap();
    clear_otlp_env_vars();
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

