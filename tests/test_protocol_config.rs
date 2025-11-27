//! Unit tests for protocol configuration validation

use otlp_arrow_library::config::{ConfigBuilder, ProtocolConfig};
use otlp_arrow_library::error::OtlpConfigError;

#[test]
fn test_valid_protocol_config_both_enabled() {
    let protocols = ProtocolConfig {
        protobuf_enabled: true,
        protobuf_port: 4317,
        arrow_flight_enabled: true,
        arrow_flight_port: 4318,
        sdk_extraction_enabled: true,
    };

    assert!(protocols.validate().is_ok());
}

#[test]
fn test_valid_protocol_config_only_protobuf() {
    let protocols = ProtocolConfig {
        protobuf_enabled: true,
        protobuf_port: 4317,
        arrow_flight_enabled: false,
        arrow_flight_port: 4318,
        sdk_extraction_enabled: true,
    };

    assert!(protocols.validate().is_ok());
}

#[test]
fn test_valid_protocol_config_only_arrow_flight() {
    let protocols = ProtocolConfig {
        protobuf_enabled: false,
        protobuf_port: 4317,
        arrow_flight_enabled: true,
        arrow_flight_port: 4318,
        sdk_extraction_enabled: true,
    };

    assert!(protocols.validate().is_ok());
}

#[test]
fn test_both_protocols_disabled_fails_validation() {
    let protocols = ProtocolConfig {
        protobuf_enabled: false,
        protobuf_port: 4317,
        arrow_flight_enabled: false,
        arrow_flight_port: 4318,
        sdk_extraction_enabled: true,
    };

    assert!(protocols.validate().is_err());
    match protocols.validate().unwrap_err() {
        OtlpConfigError::ValidationFailed(msg) => {
            assert!(msg.contains("At least one protocol must be enabled"));
        }
        _ => panic!("Expected ValidationFailed error"),
    }
}

#[test]
fn test_same_port_when_both_enabled_fails_validation() {
    let protocols = ProtocolConfig {
        protobuf_enabled: true,
        protobuf_port: 4317,
        arrow_flight_enabled: true,
        arrow_flight_port: 4317, // Same port
        sdk_extraction_enabled: true,
    };

    assert!(protocols.validate().is_err());
    match protocols.validate().unwrap_err() {
        OtlpConfigError::ValidationFailed(msg) => {
            assert!(msg.contains("ports must be different"));
        }
        _ => panic!("Expected ValidationFailed error"),
    }
}

#[test]
fn test_zero_protobuf_port_fails_validation() {
    let protocols = ProtocolConfig {
        protobuf_enabled: true,
        protobuf_port: 0, // Invalid port
        arrow_flight_enabled: false,
        arrow_flight_port: 4318,
        sdk_extraction_enabled: true,
    };

    assert!(protocols.validate().is_err());
    match protocols.validate().unwrap_err() {
        OtlpConfigError::ValidationFailed(msg) => {
            assert!(msg.contains("Protobuf port"));
        }
        _ => panic!("Expected ValidationFailed error"),
    }
}

#[test]
fn test_zero_arrow_flight_port_fails_validation() {
    let protocols = ProtocolConfig {
        protobuf_enabled: false,
        protobuf_port: 4317,
        arrow_flight_enabled: true,
        arrow_flight_port: 0, // Invalid port
        sdk_extraction_enabled: true,
    };

    assert!(protocols.validate().is_err());
    match protocols.validate().unwrap_err() {
        OtlpConfigError::ValidationFailed(msg) => {
            assert!(msg.contains("Arrow Flight port"));
        }
        _ => panic!("Expected ValidationFailed error"),
    }
}

#[test]
fn test_valid_port_range() {
    let protocols = ProtocolConfig {
        protobuf_enabled: true,
        protobuf_port: 1, // Minimum valid port
        arrow_flight_enabled: true,
        arrow_flight_port: 65535, // Maximum valid port
        sdk_extraction_enabled: true,
    };

    assert!(protocols.validate().is_ok());
}

#[test]
fn test_protocol_config_in_full_config() {
    // Test that protocol validation is called when validating full config
    let protocols = ProtocolConfig {
        protobuf_enabled: false,
        protobuf_port: 4317,
        arrow_flight_enabled: false,
        arrow_flight_port: 4318,
        sdk_extraction_enabled: true,
    };

    let config = ConfigBuilder::new()
        .output_dir("/tmp/test")
        .protocols(protocols)
        .build();

    assert!(config.is_err());
    // The error should come from protocol validation
    match config.unwrap_err() {
        OtlpConfigError::ValidationFailed(msg) => {
            assert!(msg.contains("At least one protocol must be enabled"));
        }
        _ => panic!("Expected ValidationFailed error from protocol validation"),
    }
}

#[test]
fn test_different_ports_when_both_enabled_passes() {
    let protocols = ProtocolConfig {
        protobuf_enabled: true,
        protobuf_port: 4317,
        arrow_flight_enabled: true,
        arrow_flight_port: 4318,
        sdk_extraction_enabled: true,
    };

    assert!(protocols.validate().is_ok());

    // Test with different port combinations
    let protocols2 = ProtocolConfig {
        protobuf_enabled: true,
        protobuf_port: 5000,
        arrow_flight_enabled: true,
        arrow_flight_port: 5001,
        sdk_extraction_enabled: true,
    };
    assert!(protocols2.validate().is_ok());
}
