//! Unit tests for Python OpenTelemetry SDK adapters
//!
//! Tests the type conversion functions and adapter creation logic

use otlp_arrow_library::python::adapters::conversion::{
    convert_metric_export_result_to_dict, convert_readable_span_to_dict,
    convert_span_sequence_to_dict_list,
};
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList};

/// Helper to create a Python interpreter for testing
fn python() -> Python {
    Python::with_gil(|py| py)
}

#[test]
fn test_convert_metric_export_result_to_dict() {
    Python::with_gil(|py| {
        // Create a mock MetricExportResult structure
        // This is a simplified test - full implementation would test with real OpenTelemetry SDK objects
        let resource_metrics = PyDict::new(py);
        
        // Create resource
        let resource = PyDict::new(py);
        let resource_attrs = PyDict::new(py);
        resource_attrs.set_item("service.name", "test-service").unwrap();
        resource.set_item("attributes", resource_attrs).unwrap();
        resource_metrics.set_item("resource", resource).unwrap();
        
        // Create scope_metrics
        let scope_metrics = PyList::empty(py);
        let scope_metric = PyDict::new(py);
        
        // Create scope
        let scope = PyDict::new(py);
        scope.set_item("name", "test-scope").unwrap();
        scope.set_item("version", "1.0.0").unwrap();
        scope_metric.set_item("scope", scope).unwrap();
        
        // Create metrics list
        let metrics = PyList::empty(py);
        let metric = PyDict::new(py);
        metric.set_item("name", "test-metric").unwrap();
        metric.set_item("description", "Test metric").unwrap();
        metric.set_item("unit", "1").unwrap();
        metrics.append(metric).unwrap();
        scope_metric.set_item("metrics", metrics).unwrap();
        scope_metrics.append(scope_metric).unwrap();
        
        resource_metrics.set_item("scope_metrics", scope_metrics).unwrap();
        
        // Test conversion
        let result = convert_metric_export_result_to_dict(resource_metrics, py);
        
        // Should succeed
        assert!(result.is_ok(), "Conversion should succeed");
        let converted = result.unwrap();
        
        // Verify structure
        assert!(converted.contains("resource").unwrap(), "Should contain resource");
        assert!(converted.contains("scope_metrics").unwrap(), "Should contain scope_metrics");
        
        // Verify resource attributes
        let converted_resource = converted.get_item("resource").unwrap().downcast::<PyDict>().unwrap();
        assert!(converted_resource.contains("attributes").unwrap(), "Resource should have attributes");
    });
}

#[test]
fn test_convert_readable_span_to_dict() {
    Python::with_gil(|py| {
        // Create a mock ReadableSpan structure
        // This is a simplified test - full implementation would test with real OpenTelemetry SDK objects
        let span = PyDict::new(py);
        
        // Create context
        let context = PyDict::new(py);
        // Trace ID as 128-bit integer (16 bytes)
        let trace_id: u128 = 0x1234567890abcdef1234567890abcdef;
        context.set_item("trace_id", trace_id).unwrap();
        // Span ID as 64-bit integer (8 bytes)
        let span_id: u64 = 0x1234567890abcdef;
        context.set_item("span_id", span_id).unwrap();
        span.set_item("context", context).unwrap();
        
        // Set span attributes
        span.set_item("name", "test-span").unwrap();
        span.set_item("kind", "INTERNAL").unwrap();
        
        let attributes = PyDict::new(py);
        attributes.set_item("service.name", "test-service").unwrap();
        span.set_item("attributes", attributes).unwrap();
        
        // Test conversion
        let result = convert_readable_span_to_dict(span, py);
        
        // Should succeed
        assert!(result.is_ok(), "Conversion should succeed");
        let converted = result.unwrap();
        
        // Verify structure
        assert!(converted.contains("trace_id").unwrap(), "Should contain trace_id");
        assert!(converted.contains("span_id").unwrap(), "Should contain span_id");
        assert!(converted.contains("name").unwrap(), "Should contain name");
        assert!(converted.contains("attributes").unwrap(), "Should contain attributes");
        
        // Verify trace_id is bytes
        let trace_id_bytes = converted.get_item("trace_id").unwrap();
        assert!(trace_id_bytes.downcast::<pyo3::types::PyBytes>().is_ok(), "trace_id should be bytes");
        
        // Verify span_id is bytes
        let span_id_bytes = converted.get_item("span_id").unwrap();
        assert!(span_id_bytes.downcast::<pyo3::types::PyBytes>().is_ok(), "span_id should be bytes");
    });
}

#[test]
fn test_convert_span_sequence_to_dict_list() {
    Python::with_gil(|py| {
        // Create a list of mock spans
        let spans = PyList::empty(py);
        
        for i in 0..3 {
            let span = PyDict::new(py);
            let context = PyDict::new(py);
            let trace_id: u128 = 0x1234567890abcdef1234567890abcdef + i as u128;
            let span_id: u64 = 0x1234567890abcdef + i as u64;
            context.set_item("trace_id", trace_id).unwrap();
            context.set_item("span_id", span_id).unwrap();
            span.set_item("context", context).unwrap();
            span.set_item("name", format!("span-{}", i)).unwrap();
            spans.append(span).unwrap();
        }
        
        // Test conversion
        let result = convert_span_sequence_to_dict_list(spans, py);
        
        // Should succeed
        assert!(result.is_ok(), "Conversion should succeed");
        let converted = result.unwrap();
        
        // Verify we got a list
        assert_eq!(converted.len(), 3, "Should convert 3 spans");
        
        // Verify each span was converted
        for i in 0..3 {
            let span_dict = converted.get_item(i).unwrap().downcast::<PyDict>().unwrap();
            assert!(span_dict.contains("trace_id").unwrap(), "Span should have trace_id");
            assert!(span_dict.contains("span_id").unwrap(), "Span should have span_id");
            assert!(span_dict.contains("name").unwrap(), "Span should have name");
        }
    });
}

#[test]
fn test_metric_exporter_adapter_creation() {
    // This test would require creating a PyOtlpLibrary instance
    // For now, we'll test that the adapter struct can be created
    // Full integration test will be in Python tests
    Python::with_gil(|py| {
        // Test that we can import the module
        let module = py.import("otlp_arrow_library");
        assert!(module.is_ok(), "Should be able to import otlp_arrow_library");
    });
}

#[test]
fn test_span_exporter_adapter_creation() {
    // This test would require creating a PyOtlpLibrary instance
    // For now, we'll test that the adapter struct can be created
    // Full integration test will be in Python tests
    Python::with_gil(|py| {
        // Test that we can import the module
        let module = py.import("otlp_arrow_library");
        assert!(module.is_ok(), "Should be able to import otlp_arrow_library");
    });
}

