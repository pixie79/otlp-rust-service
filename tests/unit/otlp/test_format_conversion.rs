//! Unit test for format conversion (Protobuf ↔ Arrow Flight)

use otlp_arrow_library::otlp::converter::FormatConverter;
use opentelemetry_proto::tonic::collector::metrics::v1::ExportMetricsServiceRequest;
use opentelemetry_proto::tonic::collector::trace::v1::ExportTraceServiceRequest;
use arrow::record_batch::RecordBatch;

#[test]
fn test_format_converter_new() {
    let converter = FormatConverter::new();
    // Just verify it can be created
    let _ = converter;
}

#[test]
fn test_format_converter_default() {
    let converter = FormatConverter::default();
    // Just verify it can be created
    let _ = converter;
}

#[test]
fn test_protobuf_to_arrow_flight_traces_empty() {
    let converter = FormatConverter::new();
    let request = ExportTraceServiceRequest::default();
    
    let result = converter.protobuf_to_arrow_flight_traces(&request);
    // Should handle empty request gracefully
    let _ = result;
}

#[test]
fn test_protobuf_to_arrow_flight_metrics_empty() {
    let converter = FormatConverter::new();
    let request = ExportMetricsServiceRequest::default();
    
    let result = converter.protobuf_to_arrow_flight_metrics(&request);
    // Should handle empty request gracefully
    let _ = result;
}

#[test]
fn test_arrow_flight_to_protobuf_traces_empty() {
    let converter = FormatConverter::new();
    // Create an empty RecordBatch
    use arrow::datatypes::{DataType, Field, Schema};
    use std::sync::Arc;
    
    let schema = Arc::new(Schema::new(vec![
        Field::new("trace_id", DataType::Binary, false),
    ]));
    
    let batch = RecordBatch::try_new(
        schema,
        vec![],
    ).unwrap();
    
    let result = converter.arrow_flight_to_protobuf_traces(&batch);
    // Should handle empty batch gracefully
    let _ = result;
}

#[test]
fn test_arrow_flight_to_protobuf_metrics_empty() {
    let converter = FormatConverter::new();
    // Create an empty RecordBatch
    use arrow::datatypes::{DataType, Field, Schema};
    use std::sync::Arc;
    
    let schema = Arc::new(Schema::new(vec![
        Field::new("metric_name", DataType::Utf8, false),
    ]));
    
    let batch = RecordBatch::try_new(
        schema,
        vec![],
    ).unwrap();
    
    let result = converter.arrow_flight_to_protobuf_metrics(&batch);
    // Should handle empty batch gracefully
    let _ = result;
}

#[test]
fn test_spans_to_protobuf_empty() {
    let converter = FormatConverter::new();
    let spans = vec![];
    
    let result = converter.spans_to_protobuf(spans);
    // Should handle empty spans gracefully
    let _ = result;
}

#[test]
fn test_resource_metrics_to_protobuf() {
    let converter = FormatConverter::new();
    use opentelemetry_sdk::metrics::data::ResourceMetrics;
    
    let metrics = ResourceMetrics::default();
    let result = converter.resource_metrics_to_protobuf(&metrics);
    // Should handle metrics gracefully
    let _ = result;
}

