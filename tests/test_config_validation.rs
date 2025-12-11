//! Unit tests for configuration validation

use otlp_arrow_library::config::ConfigBuilder;
use otlp_arrow_library::error::OtlpConfigError;

#[test]
fn test_valid_config_passes_validation() {
    let config = ConfigBuilder::new()
        .output_dir("/tmp/valid")
        .write_interval_secs(10)
        .trace_cleanup_interval_secs(300)
        .metric_cleanup_interval_secs(1800)
        .build()
        .unwrap();

    // Should not panic and return Ok
    assert!(config.validate().is_ok());
}

#[test]
fn test_empty_output_dir_fails_validation() {
    let config = ConfigBuilder::new().output_dir("").build();

    assert!(config.is_err());
    match config.unwrap_err() {
        OtlpConfigError::InvalidOutputDir(_) => {}
        _ => panic!("Expected InvalidOutputDir error"),
    }
}

#[test]
fn test_zero_write_interval_fails_validation() {
    let config = ConfigBuilder::new()
        .output_dir("/tmp/test")
        .write_interval_secs(0)
        .build();

    assert!(config.is_err());
    match config.unwrap_err() {
        OtlpConfigError::InvalidInterval(_) => {}
        _ => panic!("Expected InvalidInterval error"),
    }
}

#[test]
fn test_zero_trace_cleanup_interval_fails_validation() {
    let config = ConfigBuilder::new()
        .output_dir("/tmp/test")
        .trace_cleanup_interval_secs(0)
        .build();

    assert!(config.is_err());
    match config.unwrap_err() {
        OtlpConfigError::InvalidInterval(_) => {}
        _ => panic!("Expected InvalidInterval error"),
    }
}

#[test]
fn test_zero_metric_cleanup_interval_fails_validation() {
    let config = ConfigBuilder::new()
        .output_dir("/tmp/test")
        .metric_cleanup_interval_secs(0)
        .build();

    assert!(config.is_err());
    match config.unwrap_err() {
        OtlpConfigError::InvalidInterval(_) => {}
        _ => panic!("Expected InvalidInterval error"),
    }
}

#[test]
fn test_trace_cleanup_interval_too_large_fails_validation() {
    let config = ConfigBuilder::new()
        .output_dir("/tmp/test")
        .trace_cleanup_interval_secs(86401) // > 1 day
        .build();

    assert!(config.is_err());
    match config.unwrap_err() {
        OtlpConfigError::InvalidInterval(_) => {}
        _ => panic!("Expected InvalidInterval error"),
    }
}

#[test]
fn test_metric_cleanup_interval_too_large_fails_validation() {
    let config = ConfigBuilder::new()
        .output_dir("/tmp/test")
        .metric_cleanup_interval_secs(86401) // > 1 day
        .build();

    assert!(config.is_err());
    match config.unwrap_err() {
        OtlpConfigError::InvalidInterval(_) => {}
        _ => panic!("Expected InvalidInterval error"),
    }
}

#[test]
fn test_valid_cleanup_intervals_pass_validation() {
    // Test maximum allowed value (86400 seconds = 1 day)
    let config = ConfigBuilder::new()
        .output_dir("/tmp/test")
        .trace_cleanup_interval_secs(86400)
        .metric_cleanup_interval_secs(86400)
        .build()
        .unwrap();

    assert!(config.validate().is_ok());
}

#[test]
fn test_forwarding_config_validation() {
    use otlp_arrow_library::config::{ForwardingConfig, ForwardingProtocol};

    // Valid forwarding config
    let forwarding = ForwardingConfig {
        enabled: true,
        endpoint_url: Some("https://example.com/otlp".to_string()),
        protocol: ForwardingProtocol::Protobuf,
        authentication: None,
    };

    assert!(forwarding.validate().is_ok());

    // Invalid: enabled but no endpoint
    let forwarding_invalid = ForwardingConfig {
        enabled: true,
        endpoint_url: None,
        protocol: ForwardingProtocol::Protobuf,
        authentication: None,
    };

    assert!(forwarding_invalid.validate().is_err());

    // Invalid: enabled but empty endpoint
    let forwarding_empty = ForwardingConfig {
        enabled: true,
        endpoint_url: Some("".to_string()),
        protocol: ForwardingProtocol::Protobuf,
        authentication: None,
    };

    assert!(forwarding_empty.validate().is_err());

    // Invalid: endpoint without http/https
    let forwarding_bad_url = ForwardingConfig {
        enabled: true,
        endpoint_url: Some("ftp://example.com/otlp".to_string()),
        protocol: ForwardingProtocol::Protobuf,
        authentication: None,
    };

    assert!(forwarding_bad_url.validate().is_err());
}

#[test]
fn test_output_dir_path_too_long_fails_validation() {
    // Create a path that's too long (over 4096 characters)
    let long_path = "/".to_string() + &"a".repeat(4100);
    let config = ConfigBuilder::new().output_dir(&long_path).build();

    assert!(config.is_err());
    match config.unwrap_err() {
        OtlpConfigError::InvalidOutputDir(msg) => {
            assert!(msg.contains("too long"));
        }
        _ => panic!("Expected InvalidOutputDir error for path too long"),
    }
}

#[test]
fn test_write_interval_too_large_fails_validation() {
    let config = ConfigBuilder::new()
        .output_dir("/tmp/test")
        .write_interval_secs(3601) // > 1 hour
        .build();

    assert!(config.is_err());
    match config.unwrap_err() {
        OtlpConfigError::InvalidInterval(msg) => {
            assert!(msg.contains("Write interval"));
            assert!(msg.contains("3600"));
        }
        _ => panic!("Expected InvalidInterval error"),
    }
}

#[test]
fn test_trace_cleanup_interval_too_small_fails_validation() {
    let config = ConfigBuilder::new()
        .output_dir("/tmp/test")
        .trace_cleanup_interval_secs(59) // < 60 seconds
        .build();

    assert!(config.is_err());
    match config.unwrap_err() {
        OtlpConfigError::InvalidInterval(msg) => {
            assert!(msg.contains("Trace cleanup interval"));
            assert!(msg.contains("60"));
        }
        _ => panic!("Expected InvalidInterval error"),
    }
}

#[test]
fn test_metric_cleanup_interval_too_small_fails_validation() {
    let config = ConfigBuilder::new()
        .output_dir("/tmp/test")
        .metric_cleanup_interval_secs(59) // < 60 seconds
        .build();

    assert!(config.is_err());
    match config.unwrap_err() {
        OtlpConfigError::InvalidInterval(msg) => {
            assert!(msg.contains("Metric cleanup interval"));
            assert!(msg.contains("60"));
        }
        _ => panic!("Expected InvalidInterval error"),
    }
}

#[test]
fn test_valid_write_interval_max() {
    // Test maximum allowed value (3600 seconds = 1 hour)
    let config = ConfigBuilder::new()
        .output_dir("/tmp/test")
        .write_interval_secs(3600)
        .build()
        .unwrap();

    assert!(config.validate().is_ok());
}

#[test]
fn test_valid_cleanup_interval_min() {
    // Test minimum allowed value (60 seconds)
    let config = ConfigBuilder::new()
        .output_dir("/tmp/test")
        .trace_cleanup_interval_secs(60)
        .metric_cleanup_interval_secs(60)
        .build()
        .unwrap();

    assert!(config.validate().is_ok());
}

#[test]
fn test_auth_config_validation() {
    use otlp_arrow_library::config::AuthConfig;
    use secrecy::SecretString;
    use std::collections::HashMap;

    // Valid bearer token config
    let mut credentials = HashMap::new();
    credentials.insert(
        "token".to_string(),
        SecretString::new("secret-token".to_string()),
    );
    let auth = AuthConfig {
        auth_type: "bearer_token".to_string(),
        credentials,
    };
    assert!(auth.validate().is_ok());

    // Valid API key config (uses "key" not "api_key")
    let mut credentials = HashMap::new();
    credentials.insert(
        "key".to_string(),
        SecretString::new("secret-key".to_string()),
    );
    let auth = AuthConfig {
        auth_type: "api_key".to_string(),
        credentials,
    };
    assert!(auth.validate().is_ok());

    // Valid basic auth config
    let mut credentials = HashMap::new();
    credentials.insert(
        "username".to_string(),
        SecretString::new("user".to_string()),
    );
    credentials.insert(
        "password".to_string(),
        SecretString::new("pass".to_string()),
    );
    let auth = AuthConfig {
        auth_type: "basic".to_string(),
        credentials,
    };
    assert!(auth.validate().is_ok());

    // Invalid: empty auth type
    let auth = AuthConfig {
        auth_type: "".to_string(),
        credentials: HashMap::new(),
    };
    assert!(auth.validate().is_err());

    // Invalid: bearer_token without token
    let auth = AuthConfig {
        auth_type: "bearer_token".to_string(),
        credentials: HashMap::new(),
    };
    assert!(auth.validate().is_err());

    // Invalid: basic auth without username
    let mut credentials = HashMap::new();
    credentials.insert(
        "password".to_string(),
        SecretString::new("pass".to_string()),
    );
    let auth = AuthConfig {
        auth_type: "basic".to_string(),
        credentials,
    };
    assert!(auth.validate().is_err());

    // Invalid: unsupported auth type
    let auth = AuthConfig {
        auth_type: "unsupported".to_string(),
        credentials: HashMap::new(),
    };
    assert!(auth.validate().is_err());
}
