//! Unit tests for environment variable configuration loading

use otlp_arrow_library::config::ConfigLoader;
use std::path::PathBuf;
use std::sync::Mutex;

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
fn test_load_from_env_with_all_vars() {
    let _guard = ENV_MUTEX.lock().unwrap();
    clear_otlp_env_vars();

    std::env::set_var("OTLP_OUTPUT_DIR", "/tmp/env_test");
    std::env::set_var("OTLP_WRITE_INTERVAL_SECS", "20");
    std::env::set_var("OTLP_TRACE_CLEANUP_INTERVAL_SECS", "400");
    std::env::set_var("OTLP_METRIC_CLEANUP_INTERVAL_SECS", "2000");
    std::env::set_var("OTLP_PROTOBUF_ENABLED", "true");
    std::env::set_var("OTLP_PROTOBUF_PORT", "4319");
    std::env::set_var("OTLP_ARROW_FLIGHT_ENABLED", "false");
    std::env::set_var("OTLP_ARROW_FLIGHT_PORT", "4320");

    let config = ConfigLoader::from_env().unwrap();

    assert_eq!(config.output_dir, PathBuf::from("/tmp/env_test"));
    assert_eq!(config.write_interval_secs, 20);
    assert_eq!(config.trace_cleanup_interval_secs, 400);
    assert_eq!(config.metric_cleanup_interval_secs, 2000);
    assert!(config.protocols.protobuf_enabled);
    assert_eq!(config.protocols.protobuf_port, 4319);
    assert!(!config.protocols.arrow_flight_enabled);
    assert_eq!(config.protocols.arrow_flight_port, 4320);

    clear_otlp_env_vars();
}

#[test]
fn test_load_from_env_with_defaults() {
    let _guard = ENV_MUTEX.lock().unwrap();
    clear_otlp_env_vars();

    // Only set output_dir, others should use defaults
    std::env::set_var("OTLP_OUTPUT_DIR", "/tmp/env_defaults");

    let config = ConfigLoader::from_env().unwrap();

    assert_eq!(config.output_dir, PathBuf::from("/tmp/env_defaults"));
    assert_eq!(config.write_interval_secs, 5); // default
    assert_eq!(config.trace_cleanup_interval_secs, 600); // default
    assert_eq!(config.metric_cleanup_interval_secs, 3600); // default
    assert!(config.protocols.protobuf_enabled); // default
    assert_eq!(config.protocols.protobuf_port, 4317); // default

    clear_otlp_env_vars();
}

#[test]
fn test_load_from_env_with_invalid_values() {
    let _guard = ENV_MUTEX.lock().unwrap();
    clear_otlp_env_vars();

    // Set invalid values that should be ignored
    std::env::set_var("OTLP_WRITE_INTERVAL_SECS", "not_a_number");
    std::env::set_var("OTLP_PROTOBUF_PORT", "99999"); // Invalid port

    let config = ConfigLoader::from_env();

    // Should still succeed but with defaults for invalid values
    // (The validation will catch port issues, but invalid parse just uses default)
    assert!(config.is_ok());

    clear_otlp_env_vars();
}

#[test]
fn test_load_from_env_with_forwarding() {
    let _guard = ENV_MUTEX.lock().unwrap();
    clear_otlp_env_vars();

    std::env::set_var("OTLP_OUTPUT_DIR", "/tmp/env_forwarding");
    std::env::set_var("OTLP_FORWARDING_ENABLED", "true");
    std::env::set_var("OTLP_FORWARDING_ENDPOINT_URL", "https://example.com/otlp");
    std::env::set_var("OTLP_FORWARDING_PROTOCOL", "arrow_flight");

    let config = ConfigLoader::from_env().unwrap();

    assert!(config.forwarding.is_some());
    let forwarding = config.forwarding.as_ref().unwrap();
    assert!(forwarding.enabled);
    assert_eq!(
        forwarding.endpoint_url.as_ref().unwrap(),
        "https://example.com/otlp"
    );
    use otlp_arrow_library::config::ForwardingProtocol;
    assert!(matches!(
        forwarding.protocol,
        ForwardingProtocol::ArrowFlight
    ));

    clear_otlp_env_vars();
}

#[test]
fn test_env_var_priority_over_provided_config() {
    let _guard = ENV_MUTEX.lock().unwrap();
    clear_otlp_env_vars();

    // Create a config with specific values
    let provided_config = otlp_arrow_library::config::ConfigBuilder::new()
        .output_dir("/tmp/provided")
        .write_interval_secs(10)
        .build()
        .unwrap();

    // Set environment variable
    std::env::set_var("OTLP_WRITE_INTERVAL_SECS", "25");

    // Load with provided config - env should override
    let config = ConfigLoader::load(Some(provided_config)).unwrap();

    // Environment variable should override provided config
    assert_eq!(config.write_interval_secs, 25);
    // But output_dir from provided config should be used (env not set)
    assert_eq!(config.output_dir, PathBuf::from("/tmp/provided"));

    clear_otlp_env_vars();
}
