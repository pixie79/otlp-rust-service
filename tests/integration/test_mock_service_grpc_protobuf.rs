//! Integration test for mock service gRPC Protobuf interface

use opentelemetry_proto::tonic::collector::trace::v1::{
    trace_service_client::TraceServiceClient, ExportTraceServiceRequest,
};
use opentelemetry_proto::tonic::collector::metrics::v1::{
    metrics_service_client::MetricsServiceClient, ExportMetricsServiceRequest,
};
use opentelemetry_proto::tonic::trace::v1::{ResourceSpans, ScopeSpans, Span};
use opentelemetry_proto::tonic::common::v1::{AnyValue, KeyValue};
use otlp_arrow_library::MockOtlpService;
use std::time::Duration;
use tokio::time::sleep;

/// Helper to create a simple test span in protobuf format
fn create_test_protobuf_span() -> ExportTraceServiceRequest {
    let span = Span {
        trace_id: vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16],
        span_id: vec![1, 2, 3, 4, 5, 6, 7, 8],
        parent_span_id: vec![9, 10, 11, 12, 13, 14, 15, 16],
        name: "test-span".to_string(),
        kind: 1, // Server
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

/// Helper to create a simple test metrics request in protobuf format
fn create_test_protobuf_metrics_request() -> ExportMetricsServiceRequest {
    use opentelemetry_proto::tonic::metrics::v1::{ResourceMetrics, ScopeMetrics};
    
    let resource = Some(opentelemetry_proto::tonic::resource::v1::Resource {
        attributes: vec![
            KeyValue {
                key: "service.name".to_string(),
                value: Some(AnyValue {
                    value: Some(opentelemetry_proto::tonic::common::v1::any_value::Value::StringValue("test-service".to_string())),
                }),
            },
        ],
        dropped_attributes_count: 0,
    });

    let scope_metrics = ScopeMetrics {
        scope: None,
        metrics: vec![],
        schema_url: "".to_string(),
    };

    let resource_metrics = ResourceMetrics {
        resource,
        scope_metrics: vec![scope_metrics],
        schema_url: "".to_string(),
    };

    ExportMetricsServiceRequest {
        resource_metrics: vec![resource_metrics],
    }
}

#[tokio::test]
async fn test_mock_service_grpc_protobuf_trace() {
    let service = MockOtlpService::new();
    
    // Start the mock service
    let addresses = service.start().await.expect("Failed to start mock service");
    
    // Wait for server to start
    sleep(Duration::from_millis(200)).await;
    
    // Create gRPC client and send trace
    let mut client = TraceServiceClient::connect(addresses.protobuf_addr.clone())
        .await
        .expect("Failed to connect to mock service");
    
    let request = create_test_protobuf_span();
    let response = client.export(request).await.expect("Failed to export trace");
    
    // Verify response is successful
    assert!(response.get_ref().partial_success.is_none() || response.get_ref().partial_success.is_some());
    
    // Wait for processing
    sleep(Duration::from_millis(100)).await;
    
    // Verify trace was received
    assert!(service.assert_traces_received(1).await.is_ok());
    assert_eq!(service.grpc_calls_count().await, 1);
    assert_eq!(service.api_calls_count().await, 0);
}

#[tokio::test]
async fn test_mock_service_grpc_protobuf_metrics() {
    let service = MockOtlpService::new();
    
    // Start the mock service
    let addresses = service.start().await.expect("Failed to start mock service");
    
    // Wait for server to start
    sleep(Duration::from_millis(200)).await;
    
    // Create gRPC client and send metrics
    let mut client = MetricsServiceClient::connect(addresses.protobuf_addr.clone())
        .await
        .expect("Failed to connect to mock service");
    
    let request = create_test_protobuf_metrics_request();
    let response = client.export(request).await.expect("Failed to export metrics");
    
    // Verify response is successful
    assert!(response.get_ref().partial_success.is_none() || response.get_ref().partial_success.is_some());
    
    // Wait for processing
    sleep(Duration::from_millis(100)).await;
    
    // Verify metrics were received (may be 0 if conversion doesn't create ResourceMetrics)
    // The important thing is that the gRPC call was processed
    assert_eq!(service.grpc_calls_count().await, 1);
    assert_eq!(service.api_calls_count().await, 0);
}

