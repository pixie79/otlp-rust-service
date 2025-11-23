//! Unit test for ForwardingConfig validation

use otlp_arrow_library::config::{ForwardingConfig, ForwardingProtocol};
use otlp_arrow_library::error::OtlpConfigError;

#[test]
fn test_forwarding_config_disabled_no_validation() {
    let config = ForwardingConfig {
        enabled: false,
        endpoint_url: None,
        protocol: ForwardingProtocol::Protobuf,
        authentication: None,
    };
    
    assert!(config.validate().is_ok());
}

#[test]
fn test_forwarding_config_enabled_requires_endpoint() {
    let config = ForwardingConfig {
        enabled: true,
        endpoint_url: None,
        protocol: ForwardingProtocol::Protobuf,
        authentication: None,
    };
    
    let result = config.validate();
    assert!(result.is_err());
    if let Err(OtlpConfigError::MissingRequiredField(msg)) = result {
        assert!(msg.contains("endpoint_url"));
    } else {
        panic!("Expected MissingRequiredField error");
    }
}

#[test]
fn test_forwarding_config_enabled_empty_endpoint_fails() {
    let config = ForwardingConfig {
        enabled: true,
        endpoint_url: Some("".to_string()),
        protocol: ForwardingProtocol::Protobuf,
        authentication: None,
    };
    
    let result = config.validate();
    assert!(result.is_err());
    if let Err(OtlpConfigError::InvalidUrl(msg)) = result {
        assert!(msg.contains("empty"));
    } else {
        panic!("Expected InvalidUrl error");
    }
}

#[test]
fn test_forwarding_config_valid_http_endpoint() {
    let config = ForwardingConfig {
        enabled: true,
        endpoint_url: Some("http://localhost:4317".to_string()),
        protocol: ForwardingProtocol::Protobuf,
        authentication: None,
    };
    
    assert!(config.validate().is_ok());
}

#[test]
fn test_forwarding_config_valid_https_endpoint() {
    let config = ForwardingConfig {
        enabled: true,
        endpoint_url: Some("https://example.com:4317".to_string()),
        protocol: ForwardingProtocol::Protobuf,
        authentication: None,
    };
    
    assert!(config.validate().is_ok());
}

#[test]
fn test_forwarding_config_invalid_scheme_fails() {
    let config = ForwardingConfig {
        enabled: true,
        endpoint_url: Some("ftp://example.com".to_string()),
        protocol: ForwardingProtocol::Protobuf,
        authentication: None,
    };
    
    let result = config.validate();
    assert!(result.is_err());
    if let Err(OtlpConfigError::InvalidUrl(msg)) = result {
        assert!(msg.contains("http://") || msg.contains("https://"));
    } else {
        panic!("Expected InvalidUrl error");
    }
}

#[test]
fn test_forwarding_config_arrow_flight_protocol() {
    let config = ForwardingConfig {
        enabled: true,
        endpoint_url: Some("https://example.com:4318".to_string()),
        protocol: ForwardingProtocol::ArrowFlight,
        authentication: None,
    };
    
    assert!(config.validate().is_ok());
}

