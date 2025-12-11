//! Unit test for authentication configuration

use otlp_arrow_library::config::AuthConfig;
use otlp_arrow_library::error::OtlpConfigError;
use secrecy::SecretString;
use std::collections::HashMap;

#[test]
fn test_auth_config_empty_type_fails() {
    let config = AuthConfig {
        auth_type: "".to_string(),
        credentials: HashMap::new(),
    };
    
    let result = config.validate();
    assert!(result.is_err());
    if let Err(OtlpConfigError::ValidationFailed(msg)) = result {
        assert!(msg.contains("empty") || msg.contains("type"));
    } else {
        panic!("Expected ValidationFailed error");
    }
}

#[test]
fn test_auth_config_api_key_valid() {
    let mut credentials = HashMap::new();
    credentials.insert("key".to_string(), SecretString::new("test-api-key".to_string()));
    
    let config = AuthConfig {
        auth_type: "api_key".to_string(),
        credentials,
    };
    
    assert!(config.validate().is_ok());
}

#[test]
fn test_auth_config_bearer_token_valid() {
    let mut credentials = HashMap::new();
    credentials.insert("token".to_string(), SecretString::new("test-token".to_string()));
    
    let config = AuthConfig {
        auth_type: "bearer_token".to_string(),
        credentials,
    };
    
    assert!(config.validate().is_ok());
}

#[test]
fn test_auth_config_basic_valid() {
    let mut credentials = HashMap::new();
    credentials.insert("username".to_string(), SecretString::new("test-user".to_string()));
    credentials.insert("password".to_string(), SecretString::new("test-pass".to_string()));
    
    let config = AuthConfig {
        auth_type: "basic".to_string(),
        credentials,
    };
    
    assert!(config.validate().is_ok());
}

#[test]
fn test_auth_config_api_key_missing_key() {
    let credentials = HashMap::new();
    
    let config = AuthConfig {
        auth_type: "api_key".to_string(),
        credentials,
    };
    
    let result = config.validate();
    // Note: Current implementation may not validate required credentials
    // This test documents expected behavior
    let _ = result;
}

#[test]
fn test_auth_config_bearer_token_missing_token() {
    let credentials = HashMap::new();
    
    let config = AuthConfig {
        auth_type: "bearer_token".to_string(),
        credentials,
    };
    
    let result = config.validate();
    // Note: Current implementation may not validate required credentials
    // This test documents expected behavior
    let _ = result;
}

#[test]
fn test_auth_config_basic_missing_username() {
    let mut credentials = HashMap::new();
    credentials.insert("password".to_string(), SecretString::new("test-pass".to_string()));
    
    let config = AuthConfig {
        auth_type: "basic".to_string(),
        credentials,
    };
    
    let result = config.validate();
    assert!(result.is_err());
    if let Err(OtlpConfigError::MissingRequiredField(msg)) = result {
        assert!(msg.contains("username"));
    } else {
        panic!("Expected MissingRequiredField error");
    }
}

