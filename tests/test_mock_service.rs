//! Unit tests for MockOtlpService creation and state management

use otlp_arrow_library::MockOtlpService;
use opentelemetry_sdk::trace::SpanData;
use opentelemetry_sdk::metrics::data::ResourceMetrics;
use opentelemetry::trace::{SpanContext, SpanId, SpanKind, TraceId, TraceFlags, TraceState};

#[tokio::test]
async fn test_mock_service_creation() {
    let service = MockOtlpService::new();
    
    // Verify initial state
    assert_eq!(service.grpc_calls_count().await, 0);
    assert_eq!(service.api_calls_count().await, 0);
    assert!(service.assert_traces_received(0).await.is_ok());
    assert!(service.assert_metrics_received(0).await.is_ok());
}

#[tokio::test]
async fn test_mock_service_default() {
    let service = MockOtlpService::default();
    
    // Verify default creates a valid service
    assert_eq!(service.grpc_calls_count().await, 0);
    assert_eq!(service.api_calls_count().await, 0);
}

#[tokio::test]
async fn test_mock_service_receive_trace() {
    let service = MockOtlpService::new();
    
    // Create a test span
    let span = create_test_span();
    
    // Receive trace via public API
    service.receive_trace(span).await;
    
    // Verify state
    assert_eq!(service.api_calls_count().await, 1);
    assert_eq!(service.grpc_calls_count().await, 0);
    assert!(service.assert_traces_received(1).await.is_ok());
}

#[tokio::test]
async fn test_mock_service_receive_metric() {
    let service = MockOtlpService::new();
    
    // Create a test metric
    let metric = ResourceMetrics::default();
    
    // Receive metric via public API
    service.receive_metric(metric).await;
    
    // Verify state
    assert_eq!(service.api_calls_count().await, 1);
    assert_eq!(service.grpc_calls_count().await, 0);
    assert!(service.assert_metrics_received(1).await.is_ok());
}

#[tokio::test]
async fn test_mock_service_reset() {
    let service = MockOtlpService::new();
    
    // Add some data
    service.receive_trace(create_test_span()).await;
    service.receive_metric(ResourceMetrics::default()).await;
    
    // Verify data exists
    assert_eq!(service.api_calls_count().await, 2);
    assert!(service.assert_traces_received(1).await.is_ok());
    assert!(service.assert_metrics_received(1).await.is_ok());
    
    // Reset
    service.reset().await;
    
    // Verify state is cleared
    assert_eq!(service.api_calls_count().await, 0);
    assert_eq!(service.grpc_calls_count().await, 0);
    assert!(service.assert_traces_received(0).await.is_ok());
    assert!(service.assert_metrics_received(0).await.is_ok());
}

#[tokio::test]
async fn test_mock_service_assert_traces_failure() {
    let service = MockOtlpService::new();
    
    // Add one trace
    service.receive_trace(create_test_span()).await;
    
    // Assert wrong count should fail
    let result = service.assert_traces_received(2).await;
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Expected 2 traces"));
}

#[tokio::test]
async fn test_mock_service_assert_metrics_failure() {
    let service = MockOtlpService::new();
    
    // Add one metric
    service.receive_metric(ResourceMetrics::default()).await;
    
    // Assert wrong count should fail
    let result = service.assert_metrics_received(2).await;
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Expected 2 metrics"));
}

/// Helper to create a test span
fn create_test_span() -> SpanData {
    let trace_id = TraceId::from_bytes([1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16]);
    let span_id = SpanId::from_bytes([1, 2, 3, 4, 5, 6, 7, 8]);
    let span_context = SpanContext::new(trace_id, span_id, TraceFlags::default(), false, TraceState::default());
    
    let instrumentation_scope = opentelemetry::InstrumentationScope::builder("test")
        .build();
    
    SpanData {
        span_context,
        parent_span_id: SpanId::INVALID,
        span_kind: SpanKind::Internal,
        name: std::borrow::Cow::Borrowed("test-span"),
        start_time: std::time::SystemTime::now(),
        end_time: std::time::SystemTime::now(),
        attributes: std::iter::empty().collect(),
        events: opentelemetry_sdk::trace::SpanEvents::default(),
        links: opentelemetry_sdk::trace::SpanLinks::default(),
        status: opentelemetry::trace::Status::Ok,
        dropped_attributes_count: 0,
        parent_span_is_remote: false,
        instrumentation_scope,
    }
}

