//! PyO3 bindings for OTLP Arrow Library
//!
//! Provides Python bindings for the OtlpLibrary struct and related types.

#![allow(non_local_definitions)]

use crate::api::public::OtlpLibrary;
use crate::config::{Config, ConfigBuilder};
use crate::otlp::OtlpSpanExporter;
use opentelemetry::KeyValue;
use opentelemetry::trace::{
    SpanContext, SpanId, SpanKind, Status, TraceFlags, TraceId, TraceState,
};
use opentelemetry_sdk::trace::SpanData;
use pyo3::exceptions::PyRuntimeError;
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList};
use std::sync::Arc;
use std::time::SystemTime;
use tokio::runtime::Runtime;

/// Python wrapper for OtlpLibrary
#[pyclass]
pub struct PyOtlpLibrary {
    library: Arc<OtlpLibrary>,
    runtime: Arc<Runtime>,
}

#[pymethods]
impl PyOtlpLibrary {
    /// Create a new OTLP library instance
    ///
    /// Args:
    ///     output_dir: Optional output directory path (default: "./output_dir")
    ///     write_interval_secs: Optional write interval in seconds (default: 5)
    ///     trace_cleanup_interval_secs: Optional trace cleanup interval (default: 600)
    ///     metric_cleanup_interval_secs: Optional metric cleanup interval (default: 3600)
    ///     protobuf_enabled: Optional enable Protobuf protocol (default: true)
    ///     protobuf_port: Optional Protobuf port (default: 4317)
    ///     arrow_flight_enabled: Optional enable Arrow Flight protocol (default: true)
    ///     arrow_flight_port: Optional Arrow Flight port (default: 4318)
    #[new]
    #[pyo3(signature = (*, output_dir=None, write_interval_secs=None, trace_cleanup_interval_secs=None, metric_cleanup_interval_secs=None, protobuf_enabled=None, protobuf_port=None, arrow_flight_enabled=None, arrow_flight_port=None))]
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        output_dir: Option<&str>,
        write_interval_secs: Option<u64>,
        trace_cleanup_interval_secs: Option<u64>,
        metric_cleanup_interval_secs: Option<u64>,
        protobuf_enabled: Option<bool>,
        protobuf_port: Option<u16>,
        arrow_flight_enabled: Option<bool>,
        arrow_flight_port: Option<u16>,
    ) -> PyResult<Self> {
        let mut builder = ConfigBuilder::new();

        if let Some(dir) = output_dir {
            builder = builder.output_dir(dir);
        }
        if let Some(interval) = write_interval_secs {
            builder = builder.write_interval_secs(interval);
        }
        if let Some(interval) = trace_cleanup_interval_secs {
            builder = builder.trace_cleanup_interval_secs(interval);
        }
        if let Some(interval) = metric_cleanup_interval_secs {
            builder = builder.metric_cleanup_interval_secs(interval);
        }
        if let Some(enabled) = protobuf_enabled {
            builder = builder.protobuf_enabled(enabled);
        }
        if let Some(port) = protobuf_port {
            builder = builder.protobuf_port(port);
        }
        if let Some(enabled) = arrow_flight_enabled {
            builder = builder.arrow_flight_enabled(enabled);
        }
        if let Some(port) = arrow_flight_port {
            builder = builder.arrow_flight_port(port);
        }

        let config = builder
            .build()
            .map_err(|e| PyRuntimeError::new_err(format!("Configuration error: {}", e)))?;

        Self::new_with_config(config)
    }

    /// Export a single trace span from a Python dictionary
    ///
    /// Args:
    ///     span_dict: Dictionary with span data (trace_id, span_id, name, etc.)
    ///
    /// Example:
    ///     library.export_trace({
    ///         "trace_id": bytes([1, 2, ...]),  # 16 bytes
    ///         "span_id": bytes([1, 2, ...]),   # 8 bytes
    ///         "name": "my-span",
    ///         "kind": "server",  # or "client", "internal", "producer", "consumer"
    ///         "attributes": {"service.name": "my-service"}
    ///     })
    pub fn export_trace(&self, span_dict: &PyDict) -> PyResult<()> {
        let span = dict_to_span_data(span_dict)?;
        let library = self.library.clone();
        self.runtime
            .block_on(async move { library.export_trace(span).await })
            .map_err(|e| PyRuntimeError::new_err(format!("Export error: {}", e)))
    }

    /// Export multiple trace spans from a Python list of dictionaries
    ///
    /// Args:
    ///     spans: List of dictionaries, each containing span data
    pub fn export_traces(&self, spans: &PyList) -> PyResult<()> {
        let mut span_data_vec = Vec::new();
        for item in spans.iter() {
            let dict = item.downcast::<PyDict>()?;
            span_data_vec.push(dict_to_span_data(dict)?);
        }

        let library = self.library.clone();
        self.runtime
            .block_on(async move { library.export_traces(span_data_vec).await })
            .map_err(|e| PyRuntimeError::new_err(format!("Export error: {}", e)))
    }

    /// Export metrics from a Python dictionary
    ///
    /// Args:
    ///     metrics_dict: Dictionary with metrics data
    ///
    /// Note: Full metrics conversion is complex. This creates a minimal Protobuf request.
    /// For ResourceMetrics, use export_metrics_arrow instead.
    pub fn export_metrics(&self, _metrics_dict: &PyDict) -> PyResult<()> {
        // Create a minimal Protobuf request
        // Full implementation would parse the dict and create proper protobuf request
        use opentelemetry_proto::tonic::collector::metrics::v1::ExportMetricsServiceRequest;
        let request = ExportMetricsServiceRequest::default();
        let library = self.library.clone();
        self.runtime
            .block_on(async move { library.export_metrics(request).await })
            .map_err(|e| PyRuntimeError::new_err(format!("Export error: {}", e)))
    }

    /// Export metrics to Arrow format from a Python dictionary
    ///
    /// Args:
    ///     metrics_dict: Dictionary with metrics data
    ///
    /// Note: This method exports metrics via Protobuf format.
    /// Users should convert ResourceMetrics to Protobuf using opentelemetry-otlp exporter,
    /// then call export_metrics(protobuf). For now, this creates a minimal Protobuf request.
    pub fn export_metrics_arrow(&self, _metrics_dict: &PyDict) -> PyResult<()> {
        // Create a minimal Protobuf request
        // Full implementation would parse the dict and create proper protobuf request
        // Users should use opentelemetry-otlp exporter to convert ResourceMetrics to Protobuf,
        // then call export_metrics(protobuf) directly
        use opentelemetry_proto::tonic::collector::metrics::v1::ExportMetricsServiceRequest;
        let request = ExportMetricsServiceRequest::default();
        let library = self.library.clone();
        self.runtime
            .block_on(async move { library.export_metrics(request).await })
            .map_err(|e| PyRuntimeError::new_err(format!("Export error: {}", e)))
    }

    /// Force immediate flush of all buffered messages to disk
    pub fn flush(&self) -> PyResult<()> {
        let library = self.library.clone();
        self.runtime
            .block_on(async move { library.flush().await })
            .map_err(|e| PyRuntimeError::new_err(format!("Flush error: {}", e)))
    }

    /// Gracefully shut down the library, flushing all pending writes
    pub fn shutdown(&self) -> PyResult<()> {
        let library = self.library.clone();
        self.runtime
            .block_on(async move { library.shutdown().await })
            .map_err(|e| PyRuntimeError::new_err(format!("Shutdown error: {}", e)))
    }

    /// Create a SpanExporter implementation for use with OpenTelemetry SDK
    ///
    /// Returns:
    ///     PyOtlpSpanExporter: A span exporter that can be used with OpenTelemetry SDK
    ///
    /// Example:
    ///     ```python
    ///     library = PyOtlpLibrary(output_dir="/tmp/otlp")
    ///     span_exporter = library.span_exporter()
    ///     # Use span_exporter with OpenTelemetry SDK
    ///     ```
    pub fn span_exporter(&self) -> PyResult<PyOtlpSpanExporter> {
        let exporter = self.library.span_exporter();
        Ok(PyOtlpSpanExporter {
            exporter: Arc::new(exporter),
        })
    }
}

impl PyOtlpLibrary {
    /// Internal helper to create library with config
    fn new_with_config(config: Config) -> PyResult<Self> {
        // Create a Tokio runtime for async operations
        let runtime = Runtime::new()
            .map_err(|e| PyRuntimeError::new_err(format!("Failed to create runtime: {}", e)))?;

        // Create the library instance
        let library = runtime
            .block_on(async { OtlpLibrary::new(config).await })
            .map_err(|e| PyRuntimeError::new_err(format!("Failed to create library: {}", e)))?;

        Ok(Self {
            library: Arc::new(library),
            runtime: Arc::new(runtime),
        })
    }
}

/// Convert Python dictionary to SpanData
fn dict_to_span_data(dict: &PyDict) -> PyResult<SpanData> {
    // Extract trace_id (16 bytes)
    let trace_id_obj = dict
        .get_item("trace_id")?
        .ok_or_else(|| PyRuntimeError::new_err("Missing 'trace_id' in span dict"))?;
    let trace_id_bytes = trace_id_obj.downcast::<pyo3::types::PyBytes>()?.as_bytes();

    if trace_id_bytes.len() != 16 {
        return Err(PyRuntimeError::new_err("trace_id must be exactly 16 bytes"));
    }

    let trace_id = TraceId::from_bytes([
        trace_id_bytes[0],
        trace_id_bytes[1],
        trace_id_bytes[2],
        trace_id_bytes[3],
        trace_id_bytes[4],
        trace_id_bytes[5],
        trace_id_bytes[6],
        trace_id_bytes[7],
        trace_id_bytes[8],
        trace_id_bytes[9],
        trace_id_bytes[10],
        trace_id_bytes[11],
        trace_id_bytes[12],
        trace_id_bytes[13],
        trace_id_bytes[14],
        trace_id_bytes[15],
    ]);

    // Extract span_id (8 bytes)
    let span_id_obj = dict
        .get_item("span_id")?
        .ok_or_else(|| PyRuntimeError::new_err("Missing 'span_id' in span dict"))?;
    let span_id_bytes = span_id_obj.downcast::<pyo3::types::PyBytes>()?.as_bytes();

    if span_id_bytes.len() != 8 {
        return Err(PyRuntimeError::new_err("span_id must be exactly 8 bytes"));
    }

    let span_id = SpanId::from_bytes([
        span_id_bytes[0],
        span_id_bytes[1],
        span_id_bytes[2],
        span_id_bytes[3],
        span_id_bytes[4],
        span_id_bytes[5],
        span_id_bytes[6],
        span_id_bytes[7],
    ]);

    // Extract parent_span_id (optional, 8 bytes)
    let parent_span_id = dict
        .get_item("parent_span_id")
        .ok()
        .flatten()
        .and_then(|parent_bytes_obj| parent_bytes_obj.downcast::<pyo3::types::PyBytes>().ok())
        .map(|parent_bytes| {
            let bytes = parent_bytes.as_bytes();
            if bytes.len() == 8 {
                SpanId::from_bytes([
                    bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
                ])
            } else {
                SpanId::INVALID
            }
        })
        .unwrap_or(SpanId::INVALID);

    // Extract name
    let name = dict
        .get_item("name")?
        .ok_or_else(|| PyRuntimeError::new_err("Missing 'name' in span dict"))?
        .extract::<String>()?;

    // Extract kind (default to Internal)
    let span_kind = dict
        .get_item("kind")
        .ok()
        .flatten()
        .and_then(|k| k.extract::<String>().ok())
        .map(|k| match k.to_lowercase().as_str() {
            "server" => SpanKind::Server,
            "client" => SpanKind::Client,
            "producer" => SpanKind::Producer,
            "consumer" => SpanKind::Consumer,
            _ => SpanKind::Internal,
        })
        .unwrap_or(SpanKind::Internal);

    // Extract attributes (optional)
    let attributes: Vec<KeyValue> = dict
        .get_item("attributes")
        .ok()
        .flatten()
        .and_then(|attrs| attrs.downcast::<PyDict>().ok())
        .map(|attrs_dict| {
            attrs_dict
                .iter()
                .filter_map(|(key, value)| {
                    let key_str = key.extract::<String>().ok()?;
                    let value = match value.extract::<String>() {
                        Ok(s) => opentelemetry::Value::String(s.into()),
                        Err(_) => match value.extract::<i64>() {
                            Ok(i) => opentelemetry::Value::I64(i),
                            Err(_) => match value.extract::<f64>() {
                                Ok(f) => opentelemetry::Value::F64(f),
                                Err(_) => match value.extract::<bool>() {
                                    Ok(b) => opentelemetry::Value::Bool(b),
                                    Err(_) => {
                                        opentelemetry::Value::String(value.to_string().into())
                                    }
                                },
                            },
                        },
                    };
                    Some(KeyValue::new(key_str, value))
                })
                .collect()
        })
        .unwrap_or_default();

    // Extract start_time and end_time (optional, default to now)
    let start_time = SystemTime::now();
    let end_time = SystemTime::now();

    // Extract status (optional, default to Ok)
    let status = dict
        .get_item("status")
        .ok()
        .flatten()
        .and_then(|s| s.extract::<String>().ok())
        .map(|s| match s.to_lowercase().as_str() {
            "error" => Status::Error {
                description: dict
                    .get_item("status_message")
                    .ok()
                    .flatten()
                    .and_then(|m| m.extract::<String>().ok())
                    .unwrap_or_default()
                    .into(),
            },
            "unset" => Status::Unset,
            _ => Status::Ok,
        })
        .unwrap_or(Status::Ok);

    let span_context = SpanContext::new(
        trace_id,
        span_id,
        TraceFlags::default(),
        false,
        TraceState::default(),
    );

    let instrumentation_scope = opentelemetry::InstrumentationScope::builder("python").build();

    Ok(SpanData {
        span_context,
        parent_span_id,
        span_kind,
        name: std::borrow::Cow::Owned(name),
        start_time,
        end_time,
        attributes: attributes.into_iter().collect(),
        events: opentelemetry_sdk::trace::SpanEvents::default(),
        links: opentelemetry_sdk::trace::SpanLinks::default(),
        status,
        dropped_attributes_count: 0,
        parent_span_is_remote: false,
        instrumentation_scope,
    })
}

/// Python wrapper for OtlpSpanExporter
///
/// This wrapper exposes the OtlpSpanExporter to Python code.
/// The exporter field is kept for future use when Python OpenTelemetry SDK integration
/// is implemented (tracked in Issue #6).
#[pyclass]
pub struct PyOtlpSpanExporter {
    #[allow(dead_code)]
    exporter: Arc<OtlpSpanExporter>,
}

#[pymethods]
impl PyOtlpSpanExporter {
    /// Get a string representation of the exporter
    fn __repr__(&self) -> String {
        "PyOtlpSpanExporter".to_string()
    }
}

/// Python module definition
#[pymodule]
fn otlp_arrow_library(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<PyOtlpLibrary>()?;
    m.add_class::<PyOtlpSpanExporter>()?;
    m.add("__version__", env!("CARGO_PKG_VERSION"))?;
    Ok(())
}
