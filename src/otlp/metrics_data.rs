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

/// Internal representation of metric data (gauge, sum, or histogram)
#[derive(Debug, Clone)]
pub enum InternalMetricData {
    /// Gauge metric data
    Gauge(InternalGauge),
    /// Sum metric data
    Sum(InternalSum),
    /// Histogram metric data
    Histogram(InternalHistogram),
}

/// Internal representation of a gauge metric with public fields
#[derive(Debug, Clone)]
pub struct InternalGauge {
    /// Data points for the gauge metric
    pub data_points: Vec<InternalNumberDataPoint>,
}

/// Internal representation of a sum metric with public fields
#[derive(Debug, Clone)]
pub struct InternalSum {
    /// Data points for the sum metric
    pub data_points: Vec<InternalNumberDataPoint>,
    /// Aggregation temporality (cumulative or delta)
    pub aggregation_temporality: i32,
    /// Whether the sum is monotonic
    pub is_monotonic: bool,
}

/// Internal representation of a histogram metric with public fields
#[derive(Debug, Clone)]
pub struct InternalHistogram {
    /// Data points for the histogram metric
    pub data_points: Vec<InternalHistogramDataPoint>,
    /// Aggregation temporality (cumulative or delta)
    pub aggregation_temporality: i32,
}

/// Internal representation of a number data point with public fields
#[derive(Debug, Clone)]
pub struct InternalNumberDataPoint {
    /// Attributes associated with this data point
    pub attributes: Vec<KeyValue>,
    /// Start time of the data point in nanoseconds since Unix epoch
    pub start_time_unix_nano: Option<u64>,
    /// Time of the data point in nanoseconds since Unix epoch
    pub time_unix_nano: u64,
    /// Value of the data point (integer or double)
    pub value: InternalNumberValue,
}

/// Internal representation of a number value (integer or double)
#[derive(Debug, Clone)]
pub enum InternalNumberValue {
    /// Integer value
    AsInt(i64),
    /// Double (floating-point) value
    AsDouble(f64),
}

/// Internal representation of a histogram data point with public fields
#[derive(Debug, Clone)]
pub struct InternalHistogramDataPoint {
    /// Attributes associated with this data point
    pub attributes: Vec<KeyValue>,
    /// Start time of the data point in nanoseconds since Unix epoch
    pub start_time_unix_nano: Option<u64>,
    /// Time of the data point in nanoseconds since Unix epoch
    pub time_unix_nano: u64,
    /// Count of values in the histogram
    pub count: u64,
    /// Sum of all values in the histogram
    pub sum: Option<f64>,
    /// Count of values in each bucket
    pub bucket_counts: Vec<u64>,
    /// Explicit bounds for histogram buckets
    pub explicit_bounds: Vec<f64>,
    /// Minimum value in the histogram
    pub min: Option<f64>,
    /// Maximum value in the histogram
    pub max: Option<f64>,
}
