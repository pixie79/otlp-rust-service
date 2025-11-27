//! Extract metric data from SDK ResourceMetrics
//!
//! IMPORTANT: As an OpenTelemetry endpoint, we typically receive Protobuf directly via gRPC,
//! not ResourceMetrics. ResourceMetrics is an SDK-internal structure that only exists in the
//! client before export. When metrics come via gRPC, we receive ExportMetricsServiceRequest
//! (Protobuf) directly - no proxy needed!
//!
//! The proxy is ONLY needed when we receive ResourceMetrics directly (e.g., from OtlpMetricExporter
//! or export_metrics_arrow). In that case, we use opentelemetry-otlp's exporter to convert
//! ResourceMetrics to protobuf, then extract the data into our internal structure.

use crate::error::{OtlpError, OtlpExportError};
use crate::otlp::metrics_data::*;
use opentelemetry::KeyValue;
use opentelemetry_proto::tonic::collector::metrics::v1::ExportMetricsServiceRequest;
use opentelemetry_proto::tonic::common::v1::KeyValue as ProtoKeyValue;
use opentelemetry_proto::tonic::metrics::v1::{
    HistogramDataPoint as ProtoHistogramDataPoint, Metric, NumberDataPoint as ProtoNumberDataPoint,
};
use opentelemetry_sdk::metrics::data::ResourceMetrics;
use tracing::warn;

/// Extract metric data from SDK ResourceMetrics using opentelemetry-otlp conversion
///
/// This function uses opentelemetry-otlp's exporter to convert ResourceMetrics
/// to protobuf format, then extracts the data into our internal structure.
///
/// # Parameters
///
/// * `metrics` - The ResourceMetrics to extract
/// * `sdk_extraction_enabled` - Whether SDK extraction is enabled (from config)
///
/// # Limitations
///
/// Due to opentelemetry-sdk 0.31's private fields, this requires using
/// opentelemetry-otlp's exporter which may not expose conversion directly.
/// For best results, use the gRPC ingestion path which preserves protobuf format.
///
/// If `sdk_extraction_enabled` is false, returns a minimal structure.
pub async fn extract_metrics_from_resource_metrics(
    metrics: &ResourceMetrics,
    sdk_extraction_enabled: bool,
) -> Result<InternalResourceMetrics, OtlpError> {
    // If extraction is disabled, return minimal structure
    if !sdk_extraction_enabled {
        return Ok(InternalResourceMetrics {
            resource: InternalResource {
                attributes: vec![],
                dropped_attributes_count: 0,
            },
            scope_metrics: vec![],
            schema_url: String::new(),
        });
    }
    // Try to convert via opentelemetry-otlp exporter with byte capture
    match convert_via_otlp_exporter(metrics).await {
        Ok(protobuf) => extract_from_protobuf(&protobuf).map_err(|e| {
            OtlpError::Export(OtlpExportError::FormatConversionError(format!(
                "Failed to extract from protobuf: {}",
                e
            )))
        }),
        Err(e) => {
            warn!(
                error = %e,
                "Failed to extract metrics via opentelemetry-otlp. \
                 Consider using gRPC ingestion path which preserves protobuf format."
            );
            // Return minimal structure as fallback
            Ok(InternalResourceMetrics {
                resource: InternalResource {
                    attributes: vec![],
                    dropped_attributes_count: 0,
                },
                scope_metrics: vec![],
                schema_url: String::new(),
            })
        }
    }
}

/// Convert ResourceMetrics to protobuf using opentelemetry-otlp exporter
///
/// This creates a temporary gRPC server (reusing our server infrastructure)
/// that captures the protobuf request from opentelemetry-otlp's exporter.
async fn convert_via_otlp_exporter(
    metrics: &ResourceMetrics,
) -> Result<ExportMetricsServiceRequest, anyhow::Error> {
    use tokio::sync::oneshot;
    use tokio::time::{Duration, timeout};

    // Create a channel to capture the request
    let (tx, rx) = oneshot::channel();

    // Create temporary server using shared helper (reuses our server infrastructure)
    use crate::otlp::server::create_temporary_metrics_server;
    let (server_handle, addr_str) = create_temporary_metrics_server(tx)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to create temporary server: {}", e))?;

    // Give the server a moment to start
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Create an opentelemetry-otlp exporter pointing to our temporary server
    use opentelemetry_otlp::WithExportConfig;
    use opentelemetry_sdk::metrics::exporter::PushMetricExporter;

    // Create builder with explicit type to disambiguate
    type TonicMetricExporterBuilder =
        opentelemetry_otlp::MetricExporterBuilder<opentelemetry_otlp::TonicExporterBuilderSet>;

    let builder: TonicMetricExporterBuilder = opentelemetry_otlp::MetricExporterBuilder::default();
    let exporter = builder
        .with_endpoint(&addr_str)
        .build()
        .map_err(|e| anyhow::anyhow!("Failed to build metrics exporter: {}", e))?;

    // Export the metrics (this will send to our temporary server)
    exporter
        .export(metrics)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to export metrics: {}", e))?;

    // Wait for the captured request (with timeout)
    let captured_request = timeout(Duration::from_secs(5), rx)
        .await
        .map_err(|_| anyhow::anyhow!("Timeout waiting for captured request"))?
        .map_err(|_| anyhow::anyhow!("Failed to receive captured request"))?;

    // Shutdown the temporary server
    server_handle.abort();

    Ok(captured_request)
}

/// Extract internal structure from protobuf ExportMetricsServiceRequest
pub fn extract_from_protobuf(
    request: &ExportMetricsServiceRequest,
) -> Result<InternalResourceMetrics, anyhow::Error> {
    if request.resource_metrics.is_empty() {
        return Err(anyhow::anyhow!("Empty resource metrics"));
    }

    // Convert the first ResourceMetrics (handle multiple if needed)
    let proto_rm = &request.resource_metrics[0];

    // Extract resource
    let resource = if let Some(ref proto_resource) = proto_rm.resource {
        InternalResource {
            attributes: proto_resource
                .attributes
                .iter()
                .filter_map(|kv| proto_key_value_to_sdk(kv))
                .collect(),
            dropped_attributes_count: proto_resource.dropped_attributes_count,
        }
    } else {
        InternalResource {
            attributes: vec![],
            dropped_attributes_count: 0,
        }
    };

    // Extract scope metrics
    let scope_metrics = proto_rm
        .scope_metrics
        .iter()
        .map(|proto_sm| {
            let scope = if let Some(ref proto_scope) = proto_sm.scope {
                InternalInstrumentationScope {
                    name: proto_scope.name.clone(),
                    version: if proto_scope.version.is_empty() {
                        None
                    } else {
                        Some(proto_scope.version.clone())
                    },
                    attributes: proto_scope
                        .attributes
                        .iter()
                        .filter_map(|kv| proto_key_value_to_sdk(kv))
                        .collect(),
                    dropped_attributes_count: proto_scope.dropped_attributes_count,
                }
            } else {
                InternalInstrumentationScope {
                    name: "unknown".to_string(),
                    version: None,
                    attributes: vec![],
                    dropped_attributes_count: 0,
                }
            };

            let metrics = proto_sm
                .metrics
                .iter()
                .filter_map(|proto_metric| proto_metric_to_internal(proto_metric))
                .collect();

            InternalScopeMetrics {
                scope,
                metrics,
                schema_url: proto_sm.schema_url.clone(),
            }
        })
        .collect();

    Ok(InternalResourceMetrics {
        resource,
        scope_metrics,
        schema_url: proto_rm.schema_url.clone(),
    })
}

/// Convert protobuf KeyValue to SDK KeyValue
fn proto_key_value_to_sdk(kv: &ProtoKeyValue) -> Option<KeyValue> {
    let value = kv.value.as_ref()?.value.as_ref()?;
    let otel_value = match value {
        opentelemetry_proto::tonic::common::v1::any_value::Value::StringValue(s) => {
            opentelemetry::Value::String(s.clone().into())
        }
        opentelemetry_proto::tonic::common::v1::any_value::Value::IntValue(i) => {
            opentelemetry::Value::I64(*i)
        }
        opentelemetry_proto::tonic::common::v1::any_value::Value::DoubleValue(d) => {
            opentelemetry::Value::F64(*d)
        }
        opentelemetry_proto::tonic::common::v1::any_value::Value::BoolValue(b) => {
            opentelemetry::Value::Bool(*b)
        }
        opentelemetry_proto::tonic::common::v1::any_value::Value::ArrayValue(_) => {
            // Arrays not fully supported in SDK 0.31
            return None;
        }
        opentelemetry_proto::tonic::common::v1::any_value::Value::KvlistValue(_) => {
            // KeyValue lists not fully supported
            return None;
        }
        opentelemetry_proto::tonic::common::v1::any_value::Value::BytesValue(_) => {
            return None;
        }
    };
    Some(KeyValue::new(kv.key.clone(), otel_value))
}

/// Convert protobuf Metric to InternalMetric
fn proto_metric_to_internal(proto_metric: &Metric) -> Option<InternalMetric> {
    let data = match proto_metric.data.as_ref()? {
        opentelemetry_proto::tonic::metrics::v1::metric::Data::Gauge(gauge) => {
            let data_points = gauge
                .data_points
                .iter()
                .filter_map(|dp| proto_number_data_point_to_internal(dp))
                .collect();
            InternalMetricData::Gauge(InternalGauge { data_points })
        }
        opentelemetry_proto::tonic::metrics::v1::metric::Data::Sum(sum) => {
            let data_points = sum
                .data_points
                .iter()
                .filter_map(|dp| proto_number_data_point_to_internal(dp))
                .collect();
            InternalMetricData::Sum(InternalSum {
                data_points,
                aggregation_temporality: sum.aggregation_temporality,
                is_monotonic: sum.is_monotonic,
            })
        }
        opentelemetry_proto::tonic::metrics::v1::metric::Data::Histogram(hist) => {
            let data_points = hist
                .data_points
                .iter()
                .filter_map(|dp| proto_histogram_data_point_to_internal(dp))
                .collect();
            InternalMetricData::Histogram(InternalHistogram {
                data_points,
                aggregation_temporality: hist.aggregation_temporality,
            })
        }
        _ => return None, // Unsupported metric type
    };

    Some(InternalMetric {
        name: proto_metric.name.clone(),
        description: if proto_metric.description.is_empty() {
            None
        } else {
            Some(proto_metric.description.clone())
        },
        unit: if proto_metric.unit.is_empty() {
            None
        } else {
            Some(proto_metric.unit.clone())
        },
        data,
    })
}

/// Convert protobuf NumberDataPoint to InternalNumberDataPoint
fn proto_number_data_point_to_internal(
    dp: &ProtoNumberDataPoint,
) -> Option<InternalNumberDataPoint> {
    let value = match dp.value.as_ref()? {
        opentelemetry_proto::tonic::metrics::v1::number_data_point::Value::AsInt(i) => {
            InternalNumberValue::AsInt(*i)
        }
        opentelemetry_proto::tonic::metrics::v1::number_data_point::Value::AsDouble(d) => {
            InternalNumberValue::AsDouble(*d)
        }
    };

    Some(InternalNumberDataPoint {
        attributes: dp
            .attributes
            .iter()
            .filter_map(|kv| proto_key_value_to_sdk(kv))
            .collect(),
        start_time_unix_nano: if dp.start_time_unix_nano == 0 {
            None
        } else {
            Some(dp.start_time_unix_nano)
        },
        time_unix_nano: dp.time_unix_nano,
        value,
    })
}

/// Convert protobuf HistogramDataPoint to InternalHistogramDataPoint
fn proto_histogram_data_point_to_internal(
    dp: &ProtoHistogramDataPoint,
) -> Option<InternalHistogramDataPoint> {
    Some(InternalHistogramDataPoint {
        attributes: dp
            .attributes
            .iter()
            .filter_map(|kv| proto_key_value_to_sdk(kv))
            .collect(),
        start_time_unix_nano: if dp.start_time_unix_nano == 0 {
            None
        } else {
            Some(dp.start_time_unix_nano)
        },
        time_unix_nano: dp.time_unix_nano,
        count: dp.count,
        sum: dp.sum,
        bucket_counts: dp.bucket_counts.clone(),
        explicit_bounds: dp.explicit_bounds.clone(),
        min: dp.min,
        max: dp.max,
    })
}
