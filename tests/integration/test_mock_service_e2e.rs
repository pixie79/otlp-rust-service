//! Integration test for end-to-end testing with mock service

use opentelemetry_proto::tonic::collector::trace::v1::{
    trace_service_client::TraceServiceClient, ExportTraceServiceRequest,
};
use opentelemetry_proto::tonic::trace::v1::{ResourceSpans, ScopeSpans, Span};
use opentelemetry_proto::tonic::common::v1::{AnyValue, KeyValue};
use otlp_arrow_library::MockOtlpService;
use opentelemetry_sdk::trace::SpanData;
use opentelemetry::trace::{SpanContext, SpanId, SpanKind, TraceId, TraceFlags, TraceState};
use std::time::Duration;
use tokio::time::sleep;

#[tokio::test]
async fn test_mock_service_e2e_all_interfaces() {
    let service = MockOtlpService::new();
    
    // Start the mock service
    let addresses = service.start().await.expect("Failed to start mock service");
    
    // Wait for server to start
    sleep(Duration::from_millis(200)).await;
    
    // Test 1: Send trace via gRPC Protobuf
    let mut protobuf_client = TraceServiceClient::connect(addresses.protobuf_addr.clone())
        .await
        .expect("Failed to connect to mock service");
    
    let protobuf_request = create_protobuf_span();
    protobuf_client.export(protobuf_request).await.expect("Failed to export trace");
    
    sleep(Duration::from_millis(100)).await;
    
    // Test 2: Send trace via public API
    let api_span = create_test_span();
    service.receive_trace(api_span).await;
    
    // Wait for all processing
    sleep(Duration::from_millis(100)).await;
    
    // Verify both traces were received
    assert!(service.assert_traces_received(2).await.is_ok());
    assert_eq!(service.grpc_calls_count().await, 1);
    assert_eq!(service.api_calls_count().await, 1);
}

#[tokio::test]
async fn test_mock_service_e2e_reset_and_reuse() {
    let service = MockOtlpService::new();
    
    // Start the mock service
    let _addresses = service.start().await.expect("Failed to start mock service");
    
    // Wait for server to start
    sleep(Duration::from_millis(200)).await;
    
    // Send some data
    service.receive_trace(create_test_span()).await;
    service.receive_trace(create_test_span()).await;
    
    // Verify data
    assert!(service.assert_traces_received(2).await.is_ok());
    
    // Reset
    service.reset().await;
    
    // Verify cleared
    assert!(service.assert_traces_received(0).await.is_ok());
    
    // Send new data
    service.receive_trace(create_test_span()).await;
    
    // Verify new data
    assert!(service.assert_traces_received(1).await.is_ok());
    assert_eq!(service.api_calls_count().await, 1);
}

/// Helper to create a protobuf span request
fn create_protobuf_span() -> ExportTraceServiceRequest {
    let span = Span {
        trace_id: vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16],
        span_id: vec![1, 2, 3, 4, 5, 6, 7, 8],
        parent_span_id: vec![],
        name: "e2e-test-span".to_string(),
        kind: 1,
        start_time_unix_nano: 1000000000,
        end_time_unix_nano: 2000000000,
        attributes: vec![],
        dropped_attributes_count: 0,
        events: vec![],
        dropped_events_count: 0,
        links: vec![],
        dropped_links_count: 0,
        status: None,
    };

    let scope_spans = ScopeSpans {
        scope: None,
        spans: vec![span],
        schema_url: "".to_string(),
    };

    let resource_spans = ResourceSpans {
        resource: None,
        scope_spans: vec![scope_spans],
        schema_url: "".to_string(),
    };

    ExportTraceServiceRequest {
        resource_spans: vec![resource_spans],
    }
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

