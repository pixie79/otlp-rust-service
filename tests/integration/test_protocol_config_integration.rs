//! Integration test for protocol enable/disable configuration

use otlp_arrow_library::config::{ConfigBuilder, ProtocolConfig};
use tempfile::TempDir;

#[tokio::test]
async fn test_protocol_config_both_enabled() {
    let temp_dir = TempDir::new().unwrap();
    
    // Create config with both protocols enabled
    let mut protocols = ProtocolConfig::default();
    protocols.protobuf_enabled = true;
    protocols.protobuf_port = 4317;
    protocols.arrow_flight_enabled = true;
    protocols.arrow_flight_port = 4318;
    
    let config = ConfigBuilder::new()
        .output_dir(temp_dir.path())
        .protocols(protocols)
        .build()
        .unwrap();
    
    // Verify both protocols are enabled
    assert!(config.protocols.protobuf_enabled);
    assert!(config.protocols.arrow_flight_enabled);
    assert_eq!(config.protocols.protobuf_port, 4317);
    assert_eq!(config.protocols.arrow_flight_port, 4318);
}

#[tokio::test]
async fn test_protocol_config_only_protobuf() {
    let temp_dir = TempDir::new().unwrap();
    
    // Create config with only Protobuf enabled
    let mut protocols = ProtocolConfig::default();
    protocols.protobuf_enabled = true;
    protocols.protobuf_port = 4317;
    protocols.arrow_flight_enabled = false;
    
    let config = ConfigBuilder::new()
        .output_dir(temp_dir.path())
        .protocols(protocols)
        .build()
        .unwrap();
    
    // Verify only Protobuf is enabled
    assert!(config.protocols.protobuf_enabled);
    assert!(!config.protocols.arrow_flight_enabled);
}

#[tokio::test]
async fn test_protocol_config_only_arrow_flight() {
    let temp_dir = TempDir::new().unwrap();
    
    // Create config with only Arrow Flight enabled
    let mut protocols = ProtocolConfig::default();
    protocols.protobuf_enabled = false;
    protocols.arrow_flight_enabled = true;
    protocols.arrow_flight_port = 4318;
    
    let config = ConfigBuilder::new()
        .output_dir(temp_dir.path())
        .protocols(protocols)
        .build()
        .unwrap();
    
    // Verify only Arrow Flight is enabled
    assert!(!config.protocols.protobuf_enabled);
    assert!(config.protocols.arrow_flight_enabled);
}

#[tokio::test]
async fn test_protocol_config_custom_ports() {
    let temp_dir = TempDir::new().unwrap();
    
    // Create config with custom ports
    let mut protocols = ProtocolConfig::default();
    protocols.protobuf_enabled = true;
    protocols.protobuf_port = 5000;
    protocols.arrow_flight_enabled = true;
    protocols.arrow_flight_port = 5001;
    
    let config = ConfigBuilder::new()
        .output_dir(temp_dir.path())
        .protocols(protocols)
        .build()
        .unwrap();
    
    // Verify custom ports are set
    assert_eq!(config.protocols.protobuf_port, 5000);
    assert_eq!(config.protocols.arrow_flight_port, 5001);
}

#[tokio::test]
async fn test_protocol_config_defaults() {
    let temp_dir = TempDir::new().unwrap();
    
    // Create config with default protocol settings
    let config = ConfigBuilder::new()
        .output_dir(temp_dir.path())
        // Don't set protocols - should use defaults
        .build()
        .unwrap();
    
    // Verify default protocol settings
    assert!(config.protocols.protobuf_enabled, "Protobuf should be enabled by default");
    assert!(config.protocols.arrow_flight_enabled, "Arrow Flight should be enabled by default");
    assert_eq!(config.protocols.protobuf_port, 4317, "Should use default Protobuf port");
    assert_eq!(config.protocols.arrow_flight_port, 4318, "Should use default Arrow Flight port");
}

