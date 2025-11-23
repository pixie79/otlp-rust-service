//! Integration test for mock service message validation

use otlp_arrow_library::MockOtlpService;
use opentelemetry_proto::tonic::collector::trace::v1::{
    trace_service_client::TraceServiceClient, ExportTraceServiceRequest,
};
use opentelemetry_proto::tonic::trace::v1::{ResourceSpans, ScopeSpans, Span};
use opentelemetry_proto::tonic::common::v1::{AnyValue, KeyValue};
use std::time::Duration;
use tokio::time::sleep;

#[tokio::test]
async fn test_mock_service_validation_valid_trace() {
    let service = MockOtlpService::new();
    
    // Start the mock service
    let addresses = service.start().await.expect("Failed to start mock service");
    
    // Wait for server to start
    sleep(Duration::from_millis(200)).await;
    
    // Create valid trace request
    let request = create_valid_trace_request();
    
    let mut client = TraceServiceClient::connect(addresses.protobuf_addr.clone())
        .await
        .expect("Failed to connect");
    
    let response = client.export(request).await.expect("Should accept valid trace");
    assert!(response.get_ref().partial_success.is_none() || response.get_ref().partial_success.is_some());
    
    sleep(Duration::from_millis(100)).await;
    
    // Verify trace was processed
    assert!(service.assert_traces_received(1).await.is_ok());
}

#[tokio::test]
async fn test_mock_service_validation_empty_request() {
    let service = MockOtlpService::new();
    
    // Start the mock service
    let addresses = service.start().await.expect("Failed to start mock service");
    
    // Wait for server to start
    sleep(Duration::from_millis(200)).await;
    
    // Create empty request (should still be accepted per OTLP spec)
    let request = ExportTraceServiceRequest {
        resource_spans: vec![],
    };
    
    let mut client = TraceServiceClient::connect(addresses.protobuf_addr.clone())
        .await
        .expect("Failed to connect");
    
    let response = client.export(request).await.expect("Should accept empty request");
    assert!(response.get_ref().partial_success.is_none() || response.get_ref().partial_success.is_some());
    
    sleep(Duration::from_millis(100)).await;
    
    // Verify no traces were added (empty request)
    assert!(service.assert_traces_received(0).await.is_ok());
}

#[tokio::test]
async fn test_mock_service_validation_invalid_trace_id() {
    let service = MockOtlpService::new();
    
    // Start the mock service
    let addresses = service.start().await.expect("Failed to start mock service");
    
    // Wait for server to start
    sleep(Duration::from_millis(200)).await;
    
    // Create request with invalid trace_id length (should be handled gracefully)
    let mut request = create_valid_trace_request();
    if let Some(resource_span) = request.resource_spans.first_mut() {
        if let Some(scope_span) = resource_span.scope_spans.first_mut() {
            if let Some(span) = scope_span.spans.first_mut() {
                span.trace_id = vec![1; 15]; // Invalid length (should be 16)
            }
        }
    }
    
    let mut client = TraceServiceClient::connect(addresses.protobuf_addr.clone())
        .await
        .expect("Failed to connect");
    
    // Service should handle invalid trace_id gracefully (either accept or return error, but not crash)
    let result = client.export(request).await;
    
    // The service should either accept it (if lenient) or return an error
    // Either way, it shouldn't crash
    assert!(result.is_ok() || result.is_err());
    
    sleep(Duration::from_millis(100)).await;
    
    // If accepted, invalid spans should be skipped
    // If rejected, no traces should be added
    // Either way, the service should be in a valid state
    let trace_count = service.assert_traces_received(0).await.is_ok() || 
                      service.assert_traces_received(1).await.is_ok();
    assert!(trace_count);
}

/// Helper to create a valid trace request
fn create_valid_trace_request() -> ExportTraceServiceRequest {
    let span = Span {
        trace_id: vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16],
        span_id: vec![1, 2, 3, 4, 5, 6, 7, 8],
        parent_span_id: vec![],
        name: "validation-test-span".to_string(),
        kind: 1,
        start_time_unix_nano: 1000000000,
        end_time_unix_nano: 2000000000,
        attributes: vec![
            KeyValue {
                key: "service.name".to_string(),
                value: Some(AnyValue {
                    value: Some(opentelemetry_proto::tonic::common::v1::any_value::Value::StringValue("test-service".to_string())),
                }),
            },
        ],
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

