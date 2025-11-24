use otlp_arrow_library::config::{DashboardConfig, OtlpConfigError};
use std::path::PathBuf;

#[test]
fn test_dashboard_config_defaults() {
    let config = DashboardConfig::default();

    assert!(!config.enabled);
    assert_eq!(config.port, 8080);
    assert_eq!(config.static_dir, PathBuf::from("./dashboard/dist"));
}

#[test]
fn test_dashboard_config_validation_disabled() {
    let config = DashboardConfig {
        enabled: false,
        port: 0, // Invalid port, but validation should pass when disabled
        static_dir: PathBuf::from("/nonexistent"),
    };

    // Validation should pass when disabled
    assert!(config.validate().is_ok());
}

#[test]
fn test_dashboard_config_validation_invalid_port() {
    let config = DashboardConfig {
        enabled: true,
        port: 0, // Invalid port
        static_dir: PathBuf::from("./dashboard/dist"),
    };

    let result = config.validate();
    assert!(result.is_err());
    if let Err(OtlpConfigError::ValidationFailed(msg)) = result {
        assert!(msg.contains("port must be between 1 and 65535"));
    } else {
        panic!("Expected ValidationFailed error");
    }
}

#[test]
fn test_dashboard_config_validation_port_too_large() {
    let config = DashboardConfig {
        enabled: true,
        port: 65536, // Invalid port (too large)
        static_dir: PathBuf::from("./dashboard/dist"),
    };

    let result = config.validate();
    assert!(result.is_err());
}

#[test]
fn test_dashboard_config_validation_port_conflict_protobuf() {
    let config = DashboardConfig {
        enabled: true,
        port: 4317, // Conflicts with Protobuf port
        static_dir: PathBuf::from("./dashboard/dist"),
    };

    let result = config.validate();
    assert!(result.is_err());
    if let Err(OtlpConfigError::ValidationFailed(msg)) = result {
        assert!(msg.contains("conflicts with gRPC port"));
    } else {
        panic!("Expected ValidationFailed error");
    }
}

#[test]
fn test_dashboard_config_validation_port_conflict_arrow_flight() {
    let config = DashboardConfig {
        enabled: true,
        port: 4318, // Conflicts with Arrow Flight port
        static_dir: PathBuf::from("./dashboard/dist"),
    };

    let result = config.validate();
    assert!(result.is_err());
    if let Err(OtlpConfigError::ValidationFailed(msg)) = result {
        assert!(msg.contains("conflicts with gRPC port"));
    } else {
        panic!("Expected ValidationFailed error");
    }
}

#[test]
fn test_dashboard_config_validation_missing_directory() {
    let config = DashboardConfig {
        enabled: true,
        port: 8080,
        static_dir: PathBuf::from("/nonexistent/directory/that/does/not/exist"),
    };

    let result = config.validate();
    assert!(result.is_err());
    if let Err(OtlpConfigError::InvalidOutputDir(msg)) = result {
        assert!(msg.contains("does not exist"));
    } else {
        panic!("Expected InvalidOutputDir error");
    }
}

#[test]
fn test_dashboard_config_validation_valid() {
    // Create a temporary directory for testing
    let temp_dir = tempfile::tempdir().unwrap();
    let config = DashboardConfig {
        enabled: true,
        port: 8080,
        static_dir: temp_dir.path().to_path_buf(),
    };

    assert!(config.validate().is_ok());
}

