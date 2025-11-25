//! Integration tests for OtlpMetricExporter and OtlpSpanExporter with OpenTelemetry SDK

use otlp_arrow_library::{Config, OtlpLibrary};
use opentelemetry::KeyValue;
use opentelemetry_sdk::metrics::{MeterProvider, PeriodicReader};
use opentelemetry_sdk::metrics::data::ResourceMetrics;
use opentelemetry_sdk::Resource;
use opentelemetry_sdk::trace::TracerProvider;
use std::path::PathBuf;
use std::time::Duration;
use tempfile::TempDir;
use tokio::time::sleep;

#[tokio::test]
async fn test_otlp_metric_exporter_with_periodic_reader() {
    let temp_dir = TempDir::new().unwrap();
    
    let config = Config {
        output_dir: PathBuf::from(temp_dir.path()),
        write_interval_secs: 1,
        trace_cleanup_interval_secs: 600,
        metric_cleanup_interval_secs: 3600,
        protocols: Default::default(),
        forwarding: None,
        dashboard: Default::default(),
    };

    let library = OtlpLibrary::new(config).await.unwrap();
    let metric_exporter = library.metric_exporter();
    
    // Create PeriodicReader with the exporter
    let reader = PeriodicReader::builder(metric_exporter)
        .with_interval(Duration::from_secs(1))
        .build();
    
    // Create MeterProvider with the reader
    let provider = MeterProvider::builder()
        .with_reader(reader)
        .build();
    
    // Create a meter and record a metric
    let meter = provider.meter("test-instrumentation");
    let counter = meter.u64_counter("test_counter").init();
    counter.add(1, &[]);
    
    // Wait for periodic export
    sleep(Duration::from_secs(2)).await;
    
    // Flush to ensure all writes complete
    library.flush().await.expect("Failed to flush");
    
    // Verify metrics were written
    let metrics_dir = temp_dir.path().join("otlp/metrics");
    assert!(metrics_dir.exists(), "Metrics directory should exist");
    
    // Shutdown provider
    provider.shutdown().await;
    library.shutdown().await.expect("Failed to shutdown");
}

#[tokio::test]
async fn test_otlp_span_exporter_with_tracer_provider() {
    let temp_dir = TempDir::new().unwrap();
    
    let config = Config {
        output_dir: PathBuf::from(temp_dir.path()),
        write_interval_secs: 1,
        trace_cleanup_interval_secs: 600,
        metric_cleanup_interval_secs: 3600,
        protocols: Default::default(),
        forwarding: None,
        dashboard: Default::default(),
    };

    let library = OtlpLibrary::new(config).await.unwrap();
    let span_exporter = library.span_exporter();
    
    // Create TracerProvider with the exporter
    let provider = TracerProvider::builder()
        .with_batch_exporter(span_exporter, opentelemetry_sdk::runtime::Tokio)
        .build();
    
    // Create a tracer and record a span
    let tracer = provider.tracer("test-instrumentation");
    let mut span = tracer.start("test-span");
    span.set_attribute(KeyValue::new("test.attribute", "test-value"));
    span.end();
    
    // Wait for batch export
    sleep(Duration::from_secs(2)).await;
    
    // Flush to ensure all writes complete
    library.flush().await.expect("Failed to flush");
    
    // Verify traces were written
    let traces_dir = temp_dir.path().join("otlp/traces");
    assert!(traces_dir.exists(), "Traces directory should exist");
    
    // Shutdown provider
    provider.shutdown();
    library.shutdown().await.expect("Failed to shutdown");
}

#[tokio::test]
async fn test_concurrent_exporter_usage() {
    let temp_dir = TempDir::new().unwrap();
    
    let config = Config {
        output_dir: PathBuf::from(temp_dir.path()),
        write_interval_secs: 1,
        trace_cleanup_interval_secs: 600,
        metric_cleanup_interval_secs: 3600,
        protocols: Default::default(),
        forwarding: None,
        dashboard: Default::default(),
    };

    let library = OtlpLibrary::new(config).await.unwrap();
    let metric_exporter1 = library.metric_exporter();
    let metric_exporter2 = library.metric_exporter();
    let span_exporter1 = library.span_exporter();
    let span_exporter2 = library.span_exporter();
    
    // Create multiple providers with different exporters from the same library
    let reader1 = PeriodicReader::builder(metric_exporter1)
        .with_interval(Duration::from_secs(1))
        .build();
    let reader2 = PeriodicReader::builder(metric_exporter2)
        .with_interval(Duration::from_secs(1))
        .build();
    
    let provider1 = MeterProvider::builder()
        .with_reader(reader1)
        .build();
    let provider2 = MeterProvider::builder()
        .with_reader(reader2)
        .build();
    
    let provider3 = TracerProvider::builder()
        .with_batch_exporter(span_exporter1, opentelemetry_sdk::runtime::Tokio)
        .build();
    let provider4 = TracerProvider::builder()
        .with_batch_exporter(span_exporter2, opentelemetry_sdk::runtime::Tokio)
        .build();
    
    // Use all providers concurrently
    let meter1 = provider1.meter("test1");
    let meter2 = provider2.meter("test2");
    let tracer3 = provider3.tracer("test3");
    let tracer4 = provider4.tracer("test4");
    
    let counter1 = meter1.u64_counter("counter1").init();
    let counter2 = meter2.u64_counter("counter2").init();
    
    counter1.add(1, &[]);
    counter2.add(2, &[]);
    
    let mut span3 = tracer3.start("span3");
    span3.end();
    let mut span4 = tracer4.start("span4");
    span4.end();
    
    // Wait for exports
    sleep(Duration::from_secs(2)).await;
    
    // Flush to ensure all writes complete
    library.flush().await.expect("Failed to flush");
    
    // Verify data was written
    let metrics_dir = temp_dir.path().join("otlp/metrics");
    let traces_dir = temp_dir.path().join("otlp/traces");
    assert!(metrics_dir.exists(), "Metrics directory should exist");
    assert!(traces_dir.exists(), "Traces directory should exist");
    
    // Shutdown all providers
    provider1.shutdown().await;
    provider2.shutdown().await;
    provider3.shutdown();
    provider4.shutdown();
    library.shutdown().await.expect("Failed to shutdown");
}

