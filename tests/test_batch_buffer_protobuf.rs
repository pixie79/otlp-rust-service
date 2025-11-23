//! Unit test for protobuf storage in batch buffer

use opentelemetry_proto::tonic::collector::metrics::v1::ExportMetricsServiceRequest;
use otlp_arrow_library::otlp::BatchBuffer;

/// Helper to create a test protobuf metrics request
fn create_test_protobuf_metrics_request() -> ExportMetricsServiceRequest {
    // Create a minimal protobuf request with resource attributes
    use opentelemetry_proto::tonic::collector::metrics::v1::ExportMetricsServiceRequest;
    use opentelemetry_proto::tonic::common::v1::{AnyValue, KeyValue};
    use opentelemetry_proto::tonic::metrics::v1::ResourceMetrics;

    let resource = Some(opentelemetry_proto::tonic::resource::v1::Resource {
        attributes: vec![KeyValue {
            key: "service.name".to_string(),
            value: Some(AnyValue {
                value: Some(
                    opentelemetry_proto::tonic::common::v1::any_value::Value::StringValue(
                        "test-service".to_string(),
                    ),
                ),
            }),
        }],
        dropped_attributes_count: 0,
        entity_refs: vec![],
    });

    let resource_metrics = ResourceMetrics {
        resource,
        scope_metrics: vec![],
        schema_url: "".to_string(),
    };

    ExportMetricsServiceRequest {
        resource_metrics: vec![resource_metrics],
    }
}

#[tokio::test]
async fn test_batch_buffer_add_metrics_protobuf() {
    let buffer = BatchBuffer::new(5);

    let metrics_request = create_test_protobuf_metrics_request();

    // Add metrics in protobuf format
    let result = buffer.add_metrics_protobuf(metrics_request).await;
    assert!(result.is_ok(), "Adding protobuf metrics should succeed");

    // Verify count
    let count = buffer.metric_count().await;
    assert_eq!(count, 1, "Buffer should contain 1 metric");
}

#[tokio::test]
async fn test_batch_buffer_take_metrics_protobuf() {
    let buffer = BatchBuffer::new(5);

    let metrics_request = create_test_protobuf_metrics_request();
    buffer
        .add_metrics_protobuf(metrics_request.clone())
        .await
        .unwrap();

    // Take metrics (should return protobuf format)
    let taken = buffer.take_metrics().await;
    assert_eq!(taken.len(), 1, "Should take 1 metric");

    // Verify the protobuf structure is preserved
    let taken_request = &taken[0];
    assert_eq!(
        taken_request.resource_metrics.len(),
        1,
        "Protobuf request should contain resource metrics"
    );

    // Verify buffer is empty
    let count = buffer.metric_count().await;
    assert_eq!(count, 0, "Buffer should be empty after take");
}

#[tokio::test]
async fn test_batch_buffer_multiple_metrics_protobuf() {
    let buffer = BatchBuffer::new(5);

    // Add multiple metrics
    for i in 0..5 {
        let mut request = create_test_protobuf_metrics_request();
        // Modify resource to make each unique
        if let Some(ref mut rm) = request.resource_metrics.first_mut() {
            if let Some(ref mut resource) = rm.resource {
                resource.attributes[0].key = format!("service.name.{}", i);
            }
        }
        buffer.add_metrics_protobuf(request).await.unwrap();
    }

    // Verify count
    let count = buffer.metric_count().await;
    assert_eq!(count, 5, "Buffer should contain 5 metrics");

    // Take all metrics
    let taken = buffer.take_metrics().await;
    assert_eq!(taken.len(), 5, "Should take all 5 metrics");

    // Verify buffer is empty
    let count = buffer.metric_count().await;
    assert_eq!(count, 0, "Buffer should be empty after take");
}

#[tokio::test]
async fn test_batch_buffer_protobuf_clone_support() {
    let buffer = BatchBuffer::new(5);

    let metrics_request = create_test_protobuf_metrics_request();

    // Clone the request (this is the key benefit of using protobuf)
    let cloned_request = metrics_request.clone();
    assert_eq!(
        metrics_request.resource_metrics.len(),
        cloned_request.resource_metrics.len(),
        "Cloned request should have same structure"
    );

    // Add both original and cloned
    buffer.add_metrics_protobuf(metrics_request).await.unwrap();
    buffer.add_metrics_protobuf(cloned_request).await.unwrap();

    // Verify both were added
    let count = buffer.metric_count().await;
    assert_eq!(
        count, 2,
        "Buffer should contain 2 metrics (original + cloned)"
    );
}
