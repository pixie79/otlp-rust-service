//! Convert internal metric data structures to Protobuf and Arrow formats
//!
//! Provides conversion from InternalResourceMetrics to both Protobuf
//! (using opentelemetry-proto structures) and Arrow RecordBatch.

use crate::error::{OtlpError, OtlpExportError};
use crate::otlp::metrics_data::*;
use arrow::record_batch::RecordBatch;
use opentelemetry_proto::tonic::collector::metrics::v1::ExportMetricsServiceRequest;
use opentelemetry_proto::tonic::common::v1::{AnyValue, KeyValue as ProtoKeyValue};
use opentelemetry_proto::tonic::metrics::v1::{
    Gauge, Histogram, HistogramDataPoint as ProtoHistogramDataPoint, Metric,
    NumberDataPoint as ProtoNumberDataPoint, Sum,
};
use opentelemetry_proto::tonic::resource::v1::Resource as ProtoResource;
use std::sync::Arc;

impl InternalResourceMetrics {
    /// Convert to Protobuf ExportMetricsServiceRequest
    ///
    /// Uses opentelemetry-proto structures directly to build the protobuf request.
    pub fn to_protobuf(&self) -> Result<ExportMetricsServiceRequest, OtlpError> {
        use opentelemetry_proto::tonic::common::v1::InstrumentationScope as ProtoInstrumentationScope;
        use opentelemetry_proto::tonic::metrics::v1::ResourceMetrics as ProtoResourceMetrics;
        use opentelemetry_proto::tonic::metrics::v1::ScopeMetrics as ProtoScopeMetrics;

        let proto_resource = Some(ProtoResource {
            attributes: self
                .resource
                .attributes
                .iter()
                .map(sdk_key_value_to_proto)
                .collect(),
            dropped_attributes_count: self.resource.dropped_attributes_count,
            entity_refs: vec![], // Entity refs not used in SDK 0.31
        });

        let scope_metrics = self
            .scope_metrics
            .iter()
            .map(|sm| {
                let scope = Some(ProtoInstrumentationScope {
                    name: sm.scope.name.clone(),
                    version: sm.scope.version.clone().unwrap_or_default(),
                    attributes: sm
                        .scope
                        .attributes
                        .iter()
                        .map(sdk_key_value_to_proto)
                        .collect(),
                    dropped_attributes_count: sm.scope.dropped_attributes_count,
                });

                let metrics = sm
                    .metrics
                    .iter()
                    .filter_map(internal_metric_to_proto)
                    .collect();

                ProtoScopeMetrics {
                    scope,
                    metrics,
                    schema_url: sm.schema_url.clone(),
                }
            })
            .collect();

        let resource_metrics = ProtoResourceMetrics {
            resource: proto_resource,
            scope_metrics,
            schema_url: self.schema_url.clone(),
        };

        Ok(ExportMetricsServiceRequest {
            resource_metrics: vec![resource_metrics],
        })
    }

    /// Convert to Arrow RecordBatch
    ///
    /// Creates an Arrow RecordBatch with metric data in a columnar format.
    pub fn to_arrow_batch(&self) -> Result<RecordBatch, OtlpError> {
        use arrow::array::*;
        use arrow::datatypes::*;

        if self.scope_metrics.is_empty() {
            // Return empty batch with correct schema
            let schema = Schema::new(vec![
                Field::new("metric_name", DataType::Utf8, false),
                Field::new("value", DataType::Float64, false),
                Field::new("timestamp_unix_nano", DataType::UInt64, false),
                Field::new("metric_type", DataType::Utf8, false),
                Field::new("attributes", DataType::Utf8, true),
            ]);

            return RecordBatch::try_new(
                Arc::new(schema),
                vec![
                    Arc::new(StringArray::from(Vec::<String>::new())),
                    Arc::new(Float64Array::from(Vec::<f64>::new())),
                    Arc::new(UInt64Array::from(Vec::<u64>::new())),
                    Arc::new(StringArray::from(Vec::<String>::new())),
                    Arc::new(StringArray::from(Vec::<Option<String>>::new())),
                ],
            )
            .map_err(|e| OtlpError::Export(OtlpExportError::ArrowConversionError(e.to_string())));
        }

        let mut metric_names = Vec::new();
        let mut values = Vec::new();
        let mut timestamps = Vec::new();
        let mut metric_types = Vec::new();
        let mut attributes = Vec::new();

        // Flatten all metrics from all scopes into rows
        for scope_metric in &self.scope_metrics {
            for metric in &scope_metric.metrics {
                let metric_type_str = match &metric.data {
                    InternalMetricData::Gauge(_) => "gauge",
                    InternalMetricData::Sum(_) => "sum",
                    InternalMetricData::Histogram(_) => "histogram",
                };

                match &metric.data {
                    InternalMetricData::Gauge(gauge) => {
                        for dp in &gauge.data_points {
                            metric_names.push(Some(metric.name.clone()));
                            values.push(match &dp.value {
                                InternalNumberValue::AsInt(i) => *i as f64,
                                InternalNumberValue::AsDouble(d) => *d,
                            });
                            timestamps.push(dp.time_unix_nano);
                            metric_types.push(Some(metric_type_str.to_string()));

                            // Serialize attributes as JSON (convert opentelemetry::Value manually)
                            let mut attrs_map = serde_json::Map::new();
                            for kv in &dp.attributes {
                                let json_value = match &kv.value {
                                    opentelemetry::Value::String(s) => {
                                        serde_json::Value::String(s.to_string())
                                    }
                                    opentelemetry::Value::I64(i) => {
                                        serde_json::Value::Number((*i).into())
                                    }
                                    opentelemetry::Value::F64(f) => serde_json::Value::Number(
                                        serde_json::Number::from_f64(*f)
                                            .unwrap_or(serde_json::Number::from(0)),
                                    ),
                                    opentelemetry::Value::Bool(b) => serde_json::Value::Bool(*b),
                                    _ => serde_json::Value::String(format!("{:?}", kv.value)),
                                };
                                attrs_map.insert(kv.key.as_str().to_string(), json_value);
                            }
                            let attrs_json = serde_json::to_string(&attrs_map)
                                .unwrap_or_else(|_| "{}".to_string());
                            attributes.push(Some(attrs_json));
                        }
                    }
                    InternalMetricData::Sum(sum) => {
                        for dp in &sum.data_points {
                            metric_names.push(Some(metric.name.clone()));
                            values.push(match &dp.value {
                                InternalNumberValue::AsInt(i) => *i as f64,
                                InternalNumberValue::AsDouble(d) => *d,
                            });
                            timestamps.push(dp.time_unix_nano);
                            metric_types.push(Some(metric_type_str.to_string()));

                            // Serialize attributes as JSON (convert opentelemetry::Value manually)
                            let mut attrs_map = serde_json::Map::new();
                            for kv in &dp.attributes {
                                let json_value = match &kv.value {
                                    opentelemetry::Value::String(s) => {
                                        serde_json::Value::String(s.to_string())
                                    }
                                    opentelemetry::Value::I64(i) => {
                                        serde_json::Value::Number((*i).into())
                                    }
                                    opentelemetry::Value::F64(f) => serde_json::Value::Number(
                                        serde_json::Number::from_f64(*f)
                                            .unwrap_or(serde_json::Number::from(0)),
                                    ),
                                    opentelemetry::Value::Bool(b) => serde_json::Value::Bool(*b),
                                    _ => serde_json::Value::String(format!("{:?}", kv.value)),
                                };
                                attrs_map.insert(kv.key.as_str().to_string(), json_value);
                            }
                            let attrs_json = serde_json::to_string(&attrs_map)
                                .unwrap_or_else(|_| "{}".to_string());
                            attributes.push(Some(attrs_json));
                        }
                    }
                    InternalMetricData::Histogram(hist) => {
                        for dp in &hist.data_points {
                            // For histograms, we'll use the sum if available, or count
                            let value = dp.sum.unwrap_or(dp.count as f64);

                            metric_names.push(Some(metric.name.clone()));
                            values.push(value);
                            timestamps.push(dp.time_unix_nano);
                            metric_types.push(Some(metric_type_str.to_string()));

                            // Serialize attributes as JSON (convert opentelemetry::Value manually)
                            let mut attrs_map = serde_json::Map::new();
                            for kv in &dp.attributes {
                                let json_value = match &kv.value {
                                    opentelemetry::Value::String(s) => {
                                        serde_json::Value::String(s.to_string())
                                    }
                                    opentelemetry::Value::I64(i) => {
                                        serde_json::Value::Number((*i).into())
                                    }
                                    opentelemetry::Value::F64(f) => serde_json::Value::Number(
                                        serde_json::Number::from_f64(*f)
                                            .unwrap_or(serde_json::Number::from(0)),
                                    ),
                                    opentelemetry::Value::Bool(b) => serde_json::Value::Bool(*b),
                                    _ => serde_json::Value::String(format!("{:?}", kv.value)),
                                };
                                attrs_map.insert(kv.key.as_str().to_string(), json_value);
                            }
                            let attrs_json = serde_json::to_string(&attrs_map)
                                .unwrap_or_else(|_| "{}".to_string());
                            attributes.push(Some(attrs_json));
                        }
                    }
                }
            }
        }

        if metric_names.is_empty() {
            return Err(OtlpError::Export(OtlpExportError::ArrowConversionError(
                "No metric data points to convert".to_string(),
            )));
        }

        let schema = Schema::new(vec![
            Field::new("metric_name", DataType::Utf8, false),
            Field::new("value", DataType::Float64, false),
            Field::new("timestamp_unix_nano", DataType::UInt64, false),
            Field::new("metric_type", DataType::Utf8, false),
            Field::new("attributes", DataType::Utf8, true),
        ]);

        let name_refs: Vec<Option<&str>> = metric_names
            .iter()
            .map(|opt| opt.as_ref().map(|s| s.as_ref()))
            .collect();
        let type_refs: Vec<Option<&str>> = metric_types
            .iter()
            .map(|opt| opt.as_ref().map(|s| s.as_ref()))
            .collect();
        let attr_refs: Vec<Option<&str>> = attributes
            .iter()
            .map(|opt| opt.as_ref().map(|s| s.as_ref()))
            .collect();

        let batch = RecordBatch::try_new(
            Arc::new(schema),
            vec![
                Arc::new(StringArray::from(name_refs)),
                Arc::new(Float64Array::from(values)),
                Arc::new(UInt64Array::from(timestamps)),
                Arc::new(StringArray::from(type_refs)),
                Arc::new(StringArray::from(attr_refs)),
            ],
        )
        .map_err(|e| OtlpError::Export(OtlpExportError::ArrowConversionError(e.to_string())))?;

        Ok(batch)
    }
}

/// Convert SDK KeyValue to protobuf KeyValue
fn sdk_key_value_to_proto(kv: &opentelemetry::KeyValue) -> ProtoKeyValue {
    use opentelemetry_proto::tonic::common::v1::any_value::Value;

    let value = match &kv.value {
        opentelemetry::Value::String(s) => Some(AnyValue {
            value: Some(Value::StringValue(s.to_string())),
        }),
        opentelemetry::Value::I64(i) => Some(AnyValue {
            value: Some(Value::IntValue(*i)),
        }),
        opentelemetry::Value::F64(f) => Some(AnyValue {
            value: Some(Value::DoubleValue(*f)),
        }),
        opentelemetry::Value::Bool(b) => Some(AnyValue {
            value: Some(Value::BoolValue(*b)),
        }),
        _ => None, // Unsupported value types
    };

    ProtoKeyValue {
        key: kv.key.as_str().to_string(),
        value,
    }
}

/// Convert InternalMetric to protobuf Metric
fn internal_metric_to_proto(metric: &InternalMetric) -> Option<Metric> {
    let data = match &metric.data {
        InternalMetricData::Gauge(gauge) => {
            let data_points = gauge
                .data_points
                .iter()
                .filter_map(internal_number_data_point_to_proto)
                .collect();
            Some(
                opentelemetry_proto::tonic::metrics::v1::metric::Data::Gauge(Gauge { data_points }),
            )
        }
        InternalMetricData::Sum(sum) => {
            let data_points = sum
                .data_points
                .iter()
                .filter_map(internal_number_data_point_to_proto)
                .collect();
            Some(opentelemetry_proto::tonic::metrics::v1::metric::Data::Sum(
                Sum {
                    data_points,
                    aggregation_temporality: sum.aggregation_temporality,
                    is_monotonic: sum.is_monotonic,
                },
            ))
        }
        InternalMetricData::Histogram(hist) => {
            let data_points = hist
                .data_points
                .iter()
                .filter_map(internal_histogram_data_point_to_proto)
                .collect();
            Some(
                opentelemetry_proto::tonic::metrics::v1::metric::Data::Histogram(Histogram {
                    data_points,
                    aggregation_temporality: hist.aggregation_temporality,
                }),
            )
        }
    }?;

    Some(Metric {
        name: metric.name.clone(),
        description: metric.description.clone().unwrap_or_default(),
        unit: metric.unit.clone().unwrap_or_default(),
        data: Some(data),
        metadata: vec![], // Metadata not used in SDK 0.31
    })
}

/// Convert InternalNumberDataPoint to protobuf NumberDataPoint
fn internal_number_data_point_to_proto(
    dp: &InternalNumberDataPoint,
) -> Option<ProtoNumberDataPoint> {
    use opentelemetry_proto::tonic::metrics::v1::number_data_point::Value as NumberValue;

    let value = match &dp.value {
        InternalNumberValue::AsInt(i) => Some(NumberValue::AsInt(*i)),
        InternalNumberValue::AsDouble(d) => Some(NumberValue::AsDouble(*d)),
    }?;

    Some(ProtoNumberDataPoint {
        attributes: dp.attributes.iter().map(sdk_key_value_to_proto).collect(),
        start_time_unix_nano: dp.start_time_unix_nano.unwrap_or(0),
        time_unix_nano: dp.time_unix_nano,
        value: Some(value),
        exemplars: vec![], // Exemplars not used in SDK 0.31
        flags: 0,
    })
}

/// Convert InternalHistogramDataPoint to protobuf HistogramDataPoint
fn internal_histogram_data_point_to_proto(
    dp: &InternalHistogramDataPoint,
) -> Option<ProtoHistogramDataPoint> {
    Some(ProtoHistogramDataPoint {
        attributes: dp.attributes.iter().map(sdk_key_value_to_proto).collect(),
        start_time_unix_nano: dp.start_time_unix_nano.unwrap_or(0),
        time_unix_nano: dp.time_unix_nano,
        count: dp.count,
        sum: dp.sum, // Protobuf expects Option<f64>
        bucket_counts: dp.bucket_counts.clone(),
        explicit_bounds: dp.explicit_bounds.clone(),
        exemplars: vec![], // Exemplars not used in SDK 0.31
        flags: 0,
        min: dp.min, // Protobuf expects Option<f64>
        max: dp.max, // Protobuf expects Option<f64>
    })
}
