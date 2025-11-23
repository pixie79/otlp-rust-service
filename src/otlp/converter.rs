//! Format conversion module
//!
//! Converts between Protobuf and Arrow Flight formats for OTLP messages.
//!
//! This module provides bidirectional conversion between:
//! - **Protobuf**: Standard OTLP gRPC with Protobuf encoding
//! - **Arrow Flight**: OpenTelemetry Protocol with Apache Arrow (OTAP)
//!
//! # Conversion Flow
//!
//! ## Protobuf → Arrow Flight
//!
//! 1. Convert Protobuf request to SDK types (`SpanData`, `ResourceMetrics`)
//! 2. Convert SDK types to Arrow `RecordBatch`
//! 3. Return batch for Arrow Flight transmission
//!
//! ## Arrow Flight → Protobuf
//!
//! 1. Convert Arrow `RecordBatch` to SDK types
//! 2. Convert SDK types to Protobuf request
//! 3. Return request for Protobuf transmission
//!
//! # Error Handling
//!
//! All conversion methods return `Result` types and log errors using structured logging.
//! Conversion failures are logged but do not cause the application to fail - forwarding
//! will be skipped if conversion fails.

use crate::error::{OtlpError, OtlpExportError};
use arrow::record_batch::RecordBatch;
use opentelemetry_proto::tonic::collector::metrics::v1::ExportMetricsServiceRequest;
use opentelemetry_proto::tonic::collector::trace::v1::ExportTraceServiceRequest;
use opentelemetry_sdk::metrics::data::ResourceMetrics;
use opentelemetry_sdk::trace::SpanData;
use std::sync::Arc;
use tracing::{error, warn};

use crate::otlp::server;
use crate::otlp::server_arrow;

/// Format converter for OTLP messages
///
/// Provides conversion between Protobuf and Arrow Flight formats for both traces
/// and metrics. Used internally by the forwarding system to convert messages to
/// the format required by the remote endpoint.
///
/// # Example
///
/// ```no_run
/// use otlp_arrow_library::otlp::converter::FormatConverter;
/// use opentelemetry_proto::tonic::collector::trace::v1::ExportTraceServiceRequest;
///
/// let converter = FormatConverter::new();
/// let request = ExportTraceServiceRequest::default();
///
/// // Convert Protobuf to Arrow Flight
/// match converter.protobuf_to_arrow_flight_traces(&request) {
///     Ok(Some(batch)) => {
///         // Use batch for Arrow Flight transmission
///     }
///     Ok(None) => {
///         // Empty request
///     }
///     Err(e) => {
///         // Handle conversion error
///     }
/// }
/// ```
#[derive(Debug, Clone)]
pub struct FormatConverter;

impl FormatConverter {
    /// Create a new format converter
    pub fn new() -> Self {
        Self
    }

    /// Convert Protobuf trace request to Arrow Flight RecordBatch
    /// 
    /// This converts an OTLP Protobuf trace export request into Arrow format
    /// that can be sent via Arrow Flight.
    pub fn protobuf_to_arrow_flight_traces(
        &self,
        request: &ExportTraceServiceRequest,
    ) -> Result<Option<RecordBatch>, OtlpError> {
        // First convert Protobuf to SDK types
        let spans = server::convert_trace_request_to_spans(request)
            .map_err(|e| {
                error!(error = %e, "Failed to convert Protobuf trace request to spans");
                OtlpError::Export(OtlpExportError::FormatConversionError(
                    format!("Protobuf to Arrow Flight trace conversion failed: {}", e),
                ))
            })?;

        if spans.is_empty() {
            return Ok(None);
        }

        // Convert spans to Arrow RecordBatch using the same logic as exporter
        let batch = Self::spans_to_arrow_batch(&spans)
            .map_err(|e| {
                error!(error = %e, "Failed to convert spans to Arrow batch");
                OtlpError::Export(OtlpExportError::FormatConversionError(
                    format!("Span to Arrow batch conversion failed: {}", e),
                ))
            })?;

        Ok(Some(batch))
    }

    /// Convert Protobuf metrics request to Arrow Flight RecordBatch
    /// 
    /// This converts an OTLP Protobuf metrics export request into Arrow format
    /// that can be sent via Arrow Flight.
    pub fn protobuf_to_arrow_flight_metrics(
        &self,
        request: &ExportMetricsServiceRequest,
    ) -> Result<Option<RecordBatch>, OtlpError> {
        // First convert Protobuf to SDK types
        let resource_metrics = server::convert_metrics_request_to_resource_metrics(request)
            .map_err(|e| {
                error!(error = %e, "Failed to convert Protobuf metrics request to ResourceMetrics");
                OtlpError::Export(OtlpExportError::FormatConversionError(
                    format!("Protobuf to Arrow Flight metrics conversion failed: {}", e),
                ))
            })?;

        let resource_metrics = match resource_metrics {
            Some(rm) => rm,
            None => return Ok(None),
        };

        // Convert ResourceMetrics to Arrow RecordBatch using the same logic as exporter
        let batch = Self::resource_metrics_to_arrow_batch(&resource_metrics)
            .map_err(|e| {
                error!(error = %e, "Failed to convert ResourceMetrics to Arrow batch");
                OtlpError::Export(OtlpExportError::FormatConversionError(
                    format!("ResourceMetrics to Arrow batch conversion failed: {}", e),
                ))
            })?;

        Ok(Some(batch))
    }

    /// Convert Arrow Flight RecordBatch to Protobuf trace request
    /// 
    /// This converts an Arrow Flight trace batch into OTLP Protobuf format
    /// that can be sent via standard gRPC Protobuf.
    pub fn arrow_flight_to_protobuf_traces(
        &self,
        batch: &RecordBatch,
    ) -> Result<Option<ExportTraceServiceRequest>, OtlpError> {
        // Convert Arrow batch to SDK spans
        let spans = server_arrow::convert_arrow_batch_to_spans(batch)
            .map_err(|e| {
                error!(error = %e, "Failed to convert Arrow batch to spans");
                OtlpError::Export(OtlpExportError::FormatConversionError(
                    format!("Arrow batch to spans conversion failed: {}", e),
                ))
            })?;

        if spans.is_empty() {
            return Ok(None);
        }

        // Convert spans to Protobuf request
        // Note: We need to reconstruct the Protobuf request from spans
        // This is a simplified conversion - in a full implementation, we'd need
        // to properly reconstruct ResourceSpans with all metadata
        let request = ExportTraceServiceRequest::default();
        
        // For now, we create a minimal request
        // A full implementation would need to properly group spans by resource and scope
        warn!("Arrow Flight to Protobuf trace conversion: Simplified implementation - full metadata reconstruction not yet implemented");
        
        // TODO: Properly reconstruct ResourceSpans from spans with resource and scope information
        // This requires tracking resource and scope metadata during conversion
        
        Ok(Some(request))
    }

    /// Convert Arrow Flight RecordBatch to Protobuf metrics request
    /// 
    /// This converts an Arrow Flight metrics batch into OTLP Protobuf format
    /// that can be sent via standard gRPC Protobuf.
    pub fn arrow_flight_to_protobuf_metrics(
        &self,
        batch: &RecordBatch,
    ) -> Result<Option<ExportMetricsServiceRequest>, OtlpError> {
        // Convert Arrow batch to SDK ResourceMetrics
        let resource_metrics = server_arrow::convert_arrow_batch_to_resource_metrics(batch)
            .map_err(|e| {
                error!(error = %e, "Failed to convert Arrow batch to ResourceMetrics");
                OtlpError::Export(OtlpExportError::FormatConversionError(
                    format!("Arrow batch to ResourceMetrics conversion failed: {}", e),
                ))
            })?;

        let resource_metrics = match resource_metrics {
            Some(rm) => rm,
            None => return Ok(None),
        };

        // Convert ResourceMetrics to Protobuf request
        // Note: We need to reconstruct the Protobuf request from ResourceMetrics
        // This is a simplified conversion
        let request = ExportMetricsServiceRequest::default();
        
        warn!("Arrow Flight to Protobuf metrics conversion: Simplified implementation - full metadata reconstruction not yet implemented");
        
        // TODO: Properly reconstruct ResourceMetrics in Protobuf format
        // This requires converting SDK ResourceMetrics back to Protobuf ResourceMetrics
        
        Ok(Some(request))
    }

    /// Convert SDK spans to Protobuf trace request
    /// 
    /// Helper method to convert spans (from any source) to Protobuf format.
    pub fn spans_to_protobuf(
        &self,
        spans: Vec<SpanData>,
    ) -> Result<Option<ExportTraceServiceRequest>, OtlpError> {
        if spans.is_empty() {
            return Ok(None);
        }

        // Create a minimal Protobuf request
        // Full implementation would properly group spans by resource and scope
        let request = ExportTraceServiceRequest::default();
        
        warn!("Spans to Protobuf conversion: Simplified implementation - full metadata reconstruction not yet implemented");
        
        Ok(Some(request))
    }

    /// Convert SDK ResourceMetrics to Protobuf metrics request
    /// 
    /// Helper method to convert ResourceMetrics (from any source) to Protobuf format.
    pub fn resource_metrics_to_protobuf(
        &self,
        metrics: &ResourceMetrics,
    ) -> Result<Option<ExportMetricsServiceRequest>, OtlpError> {
        // Create a minimal Protobuf request
        let request = ExportMetricsServiceRequest::default();
        
        warn!("ResourceMetrics to Protobuf conversion: Simplified implementation - full metadata reconstruction not yet implemented");
        
        Ok(Some(request))
    }
}

impl Default for FormatConverter {
    fn default() -> Self {
        Self::new()
    }
}

impl FormatConverter {
    /// Convert spans to Arrow RecordBatch (helper function)
    pub(crate) fn spans_to_arrow_batch(spans: &[SpanData]) -> Result<RecordBatch, anyhow::Error> {
        use arrow::array::*;
        use arrow::datatypes::*;
        use std::sync::Arc;

        if spans.is_empty() {
            return Err(anyhow::anyhow!("Cannot create empty RecordBatch"));
        }

        // Create Arrow schema for spans (same as exporter)
        let schema = Schema::new(vec![
            Field::new("trace_id", DataType::Binary, false),
            Field::new("span_id", DataType::Binary, false),
            Field::new("parent_span_id", DataType::Binary, true),
            Field::new("name", DataType::Utf8, false),
            Field::new("kind", DataType::Int32, false),
            Field::new("start_time_unix_nano", DataType::UInt64, false),
            Field::new("end_time_unix_nano", DataType::UInt64, false),
            Field::new("status_code", DataType::Int32, false),
            Field::new("status_message", DataType::Utf8, true),
            Field::new("attributes", DataType::Utf8, true), // JSON-encoded
        ]);

        let mut trace_ids = Vec::new();
        let mut span_ids = Vec::new();
        let mut parent_span_ids = Vec::new();
        let mut names = Vec::new();
        let mut kinds = Vec::new();
        let mut start_times = Vec::new();
        let mut end_times = Vec::new();
        let mut status_codes = Vec::new();
        let mut status_messages = Vec::new();
        let mut attributes = Vec::new();

        for span_data in spans {
            trace_ids.push(Some(span_data.span_context.trace_id().to_bytes().to_vec()));
            span_ids.push(Some(span_data.span_context.span_id().to_bytes().to_vec()));
            let parent_bytes = span_data.parent_span_id.to_bytes();
            parent_span_ids.push(if parent_bytes.iter().any(|&b| b != 0) {
                Some(parent_bytes.to_vec())
            } else {
                None
            });
            names.push(Some(span_data.name.to_string()));
            kinds.push(span_data.span_kind.clone() as i32);
            start_times.push(span_data.start_time.duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default().as_nanos() as u64);
            end_times.push(span_data.end_time.duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default().as_nanos() as u64);
            use opentelemetry::trace::Status as OtelStatus;
            status_codes.push(match span_data.status {
                OtelStatus::Unset => 0,
                OtelStatus::Ok => 1,
                OtelStatus::Error { .. } => 2,
            });
            status_messages.push(Some(String::new())); // Status message not available in opentelemetry 0.31

            // Serialize attributes as JSON
            let mut attrs_obj = serde_json::Map::new();
            for kv in span_data.attributes.iter() {
                let key = kv.key.as_str();
                let json_value = match &kv.value {
                    opentelemetry::Value::I64(i) => serde_json::Value::Number((*i).into()),
                    opentelemetry::Value::F64(f) => serde_json::Value::Number(serde_json::Number::from_f64(*f).unwrap_or(serde_json::Number::from(0))),
                    opentelemetry::Value::Bool(b) => serde_json::Value::Bool(*b),
                    opentelemetry::Value::String(s) => serde_json::Value::String(s.to_string()),
                    _ => serde_json::Value::String(format!("{:?}", kv.value)),
                };
                attrs_obj.insert(key.to_string(), json_value);
            }
            let attrs_json = serde_json::to_string(&attrs_obj).unwrap_or_else(|_| "{}".to_string());
            attributes.push(Some(attrs_json));
        }

        // Build Arrow arrays
        let trace_id_refs: Vec<Option<&[u8]>> = trace_ids.iter().map(|opt| opt.as_deref()).collect();
        let span_id_refs: Vec<Option<&[u8]>> = span_ids.iter().map(|opt| opt.as_deref()).collect();
        let parent_span_id_refs: Vec<Option<&[u8]>> = parent_span_ids.iter().map(|opt| opt.as_deref()).collect();
        let name_refs: Vec<Option<&str>> = names.iter().map(|opt| opt.as_ref().map(|s| s.as_ref())).collect();

        let trace_id_array = Arc::new(BinaryArray::from(trace_id_refs));
        let span_id_array = Arc::new(BinaryArray::from(span_id_refs));
        let parent_span_id_array = Arc::new(BinaryArray::from(parent_span_id_refs));
        let name_array = Arc::new(StringArray::from(name_refs));
        let kind_array = Arc::new(Int32Array::from(kinds));
        let start_time_array = Arc::new(UInt64Array::from(start_times));
        let end_time_array = Arc::new(UInt64Array::from(end_times));
        let status_code_array = Arc::new(Int32Array::from(status_codes));
        let status_message_array = Arc::new(StringArray::from(status_messages));
        let attributes_array = Arc::new(StringArray::from(attributes));

        // Create RecordBatch
        let batch = RecordBatch::try_new(
            Arc::new(schema),
            vec![
                trace_id_array,
                span_id_array,
                parent_span_id_array,
                name_array,
                kind_array,
                start_time_array,
                end_time_array,
                status_code_array,
                status_message_array,
                attributes_array,
            ],
        )?;

        Ok(batch)
    }

    /// Convert ResourceMetrics to Arrow RecordBatch (helper function)
    /// 
    /// Note: ResourceMetrics fields are private in opentelemetry-sdk 0.31, so we use
    /// a simplified approach. For full implementation, we'd need to use opentelemetry-proto
    /// conversion utilities or access metrics through public APIs.
    pub(crate) fn resource_metrics_to_arrow_batch(_metrics: &ResourceMetrics) -> Result<RecordBatch, anyhow::Error> {
        use arrow::array::*;
        use arrow::datatypes::*;
        use std::sync::Arc;

        // Since ResourceMetrics fields are private, we create a minimal empty batch
        let _ = _metrics; // Acknowledge parameter for future use
        // Full implementation would require proper access to metrics data
        // This is a placeholder that creates the correct schema structure
        let schema = Schema::new(vec![
            Field::new("metric_name", DataType::Utf8, false),
            Field::new("value", DataType::Float64, false),
            Field::new("timestamp_unix_nano", DataType::UInt64, false),
            Field::new("metric_type", DataType::Utf8, false),
            Field::new("attributes", DataType::Utf8, true),
        ]);

        // Create empty batch for now - full implementation would extract actual metrics
        let batch = RecordBatch::try_new(
            Arc::new(schema),
            vec![
                Arc::new(StringArray::from(Vec::<String>::new())),
                Arc::new(Float64Array::from(Vec::<f64>::new())),
                Arc::new(UInt64Array::from(Vec::<u64>::new())),
                Arc::new(StringArray::from(Vec::<String>::new())),
                Arc::new(StringArray::from(Vec::<Option<String>>::new())),
            ],
        )?;

        Ok(batch)
    }
}

