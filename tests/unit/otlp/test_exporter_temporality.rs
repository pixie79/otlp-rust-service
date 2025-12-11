//! Unit tests for exporter temporality configuration
//!
//! Tests that temporality can be configured for metric exporters and defaults to Cumulative.

use otlp_arrow_library::otlp::OtlpFileExporter;
use otlp_arrow_library::config::Config;
use opentelemetry_sdk::metrics::Temporality;

#[tokio::test]
async fn test_exporter_temporality_default() {
    // Test that temporality defaults to Cumulative
    let config = Config::default();
    let exporter = OtlpFileExporter::new(&config).unwrap();
    
    // Verify default temporality
    assert_eq!(exporter.temporality(), Temporality::Cumulative, "Default temporality should be Cumulative");
}

#[tokio::test]
async fn test_exporter_temporality_cumulative() {
    // Test setting temporality to Cumulative
    let mut config = Config::default();
    config.metric_temporality = Some(Temporality::Cumulative);
    let exporter = OtlpFileExporter::new(&config).unwrap();
    
    // Verify temporality is Cumulative
    assert_eq!(exporter.temporality(), Temporality::Cumulative, "Temporality should be Cumulative");
}

#[tokio::test]
async fn test_exporter_temporality_delta() {
    // Test setting temporality to Delta
    let mut config = Config::default();
    config.metric_temporality = Some(Temporality::Delta);
    let exporter = OtlpFileExporter::new(&config).unwrap();
    
    // Verify temporality is Delta
    assert_eq!(exporter.temporality(), Temporality::Delta, "Temporality should be Delta");
}
