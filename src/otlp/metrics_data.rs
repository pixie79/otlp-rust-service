//! Internal metric data structures with public fields
//!
//! These structures mirror ResourceMetrics but with public fields,
//! allowing us to extract and convert metrics without relying on
//! opentelemetry-sdk's private fields.

use opentelemetry::KeyValue;

/// Internal representation of resource metrics with public fields
///
/// This structure mirrors `opentelemetry_sdk::metrics::data::ResourceMetrics`
/// but with public fields, allowing direct access and conversion without
/// relying on the SDK's private field accessors.
#[derive(Debug, Clone)]
pub struct InternalResourceMetrics {
    /// Resource attributes and metadata
    pub resource: InternalResource,
    /// Scope-specific metrics collections
    pub scope_metrics: Vec<InternalScopeMetrics>,
    /// Schema URL for the resource metrics
    pub schema_url: String,
}

/// Internal representation of a resource with public fields
#[derive(Debug, Clone)]
pub struct InternalResource {
    /// Resource attributes as key-value pairs
    pub attributes: Vec<KeyValue>,
    /// Number of attributes that were dropped due to limits
    pub dropped_attributes_count: u32,
}

/// Internal representation of scope metrics with public fields
#[derive(Debug, Clone)]
pub struct InternalScopeMetrics {
    /// Instrumentation scope information
    pub scope: InternalInstrumentationScope,
    /// Metrics collected in this scope
    pub metrics: Vec<InternalMetric>,
    /// Schema URL for the scope metrics
    pub schema_url: String,
}

/// Internal representation of an instrumentation scope with public fields
#[derive(Debug, Clone)]
pub struct InternalInstrumentationScope {
    /// Name of the instrumentation scope
    pub name: String,
    /// Version of the instrumentation scope
    pub version: Option<String>,
    /// Scope attributes as key-value pairs
    pub attributes: Vec<KeyValue>,
    /// Number of attributes that were dropped due to limits
    pub dropped_attributes_count: u32,
}

/// Internal representation of a metric with public fields
#[derive(Debug, Clone)]
pub struct InternalMetric {
    /// Metric name
    pub name: String,
    /// Metric description
    pub description: Option<String>,
    /// Metric unit
    pub unit: Option<String>,
    /// Metric data (gauge, sum, or histogram)
    pub data: InternalMetricData,
}

#[derive(Debug, Clone)]
pub enum InternalMetricData {
    Gauge(InternalGauge),
    Sum(InternalSum),
    Histogram(InternalHistogram),
}

#[derive(Debug, Clone)]
pub struct InternalGauge {
    pub data_points: Vec<InternalNumberDataPoint>,
}

#[derive(Debug, Clone)]
pub struct InternalSum {
    pub data_points: Vec<InternalNumberDataPoint>,
    pub aggregation_temporality: i32,
    pub is_monotonic: bool,
}

#[derive(Debug, Clone)]
pub struct InternalHistogram {
    pub data_points: Vec<InternalHistogramDataPoint>,
    pub aggregation_temporality: i32,
}

#[derive(Debug, Clone)]
pub struct InternalNumberDataPoint {
    pub attributes: Vec<KeyValue>,
    pub start_time_unix_nano: Option<u64>,
    pub time_unix_nano: u64,
    pub value: InternalNumberValue,
}

#[derive(Debug, Clone)]
pub enum InternalNumberValue {
    AsInt(i64),
    AsDouble(f64),
}

#[derive(Debug, Clone)]
pub struct InternalHistogramDataPoint {
    pub attributes: Vec<KeyValue>,
    pub start_time_unix_nano: Option<u64>,
    pub time_unix_nano: u64,
    pub count: u64,
    pub sum: Option<f64>,
    pub bucket_counts: Vec<u64>,
    pub explicit_bounds: Vec<f64>,
    pub min: Option<f64>,
    pub max: Option<f64>,
}
