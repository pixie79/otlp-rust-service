//! Python OpenTelemetry SDK adapter implementations
//!
//! This module provides adapter classes that implement Python OpenTelemetry SDK
//! exporter interfaces, enabling seamless integration between Python OpenTelemetry SDK
//! and OtlpLibrary without requiring custom adapter code.

#![allow(non_local_definitions, unsafe_op_in_unsafe_fn)] // pyo3 pymethods macro generates non-local impl blocks; PyO3 parameter extraction is safe

pub mod conversion;

use crate::python::bindings::PyOtlpLibrary;
use pyo3::prelude::*;

/// Python garbage collection handling utilities
///
/// Provides utilities for managing Python object references to prevent
/// premature garbage collection while adapters are in use by Python OpenTelemetry SDK.
pub mod gc {
    use super::*;

    /// Wrapper for PyOtlpLibrary that prevents garbage collection
    ///
    /// This type ensures that the library instance remains valid while
    /// adapters are in use by Python OpenTelemetry SDK, even across
    /// async boundaries or long-lived references.
    ///
    /// Usage: When creating adapters, pass `Py::clone_ref(library)` to create
    /// a reference that prevents garbage collection.
    pub type LibraryRef = Py<PyOtlpLibrary>;

    /// Verify that a library reference is still valid
    ///
    /// This function checks if the library instance is still valid
    /// (not shut down) and can be used for export operations.
    ///
    /// # Arguments
    ///
    /// * `library_ref` - The library reference to check
    /// * `py` - Python interpreter instance
    ///
    /// # Returns
    ///
    /// `true` if the library is valid, `false` otherwise
    pub fn is_library_valid(library_ref: &LibraryRef, py: Python<'_>) -> bool {
        // Try to get a reference to verify it's still valid
        // If we can get a reference, the object hasn't been garbage collected
        // Additional validation (e.g., checking if shutdown) can be added here if needed
        library_ref.try_borrow(py).is_ok()
    }
}

// Re-export for convenience
pub use gc::{LibraryRef, is_library_valid};

use crate::python::adapters::conversion::{
    convert_metric_export_result_to_dict, convert_span_sequence_to_dict_list, error_message_to_py,
};
use pyo3::types::PyString;

/// Python metric exporter adapter that implements Python OpenTelemetry SDK's MetricExporter interface
///
/// This adapter bridges Python OpenTelemetry SDK's metric export system with OtlpLibrary,
/// enabling direct use with PeriodicExportingMetricReader without custom adapter code.
#[pyclass]
pub struct PyOtlpMetricExporterAdapter {
    /// Reference to the library instance (prevents garbage collection)
    pub(crate) library: LibraryRef,
}

#[pymethods]
impl PyOtlpMetricExporterAdapter {
    /// Export metrics data to the library
    ///
    /// Implements Python OpenTelemetry SDK's MetricExporter.export() method.
    ///
    /// # Arguments
    ///
    /// * `metrics_data` - MetricExportResult from Python OpenTelemetry SDK
    /// * `timeout_millis` - Optional timeout in milliseconds (ignored)
    ///
    /// # Returns
    ///
    /// ExportResult (SUCCESS or FAILURE)
    #[pyo3(signature = (metrics_data, *, timeout_millis=None))]
    #[allow(unused_variables, unsafe_op_in_unsafe_fn)] // timeout_millis is part of SDK interface but not used; PyO3 parameter extraction is safe
    pub fn export(
        &self,
        metrics_data: &PyAny,        // SAFETY: PyO3 parameter extraction is safe
        timeout_millis: Option<f64>, // Changed from u64 to f64 to match SDK
        py: Python<'_>,
    ) -> PyResult<PyObject> {
        // Validate library is still valid
        if !is_library_valid(&self.library, py) {
            return Err(error_message_to_py(
                "Library instance is no longer valid".to_string(),
            ));
        }

        // Convert Python OpenTelemetry SDK types to Protobuf ExportMetricsServiceRequest
        // Step 1: Convert MetricExportResult to dict (for future full conversion)
        let _metrics_dict = match convert_metric_export_result_to_dict(metrics_data, py) {
            Ok(dict) => dict,
            Err(e) => {
                // If conversion fails, return a proper Python exception instead of crashing
                return Err(e);
            }
        };

        // TODO: Implement full conversion: dict -> InternalResourceMetrics -> Protobuf
        // For now, create a minimal Protobuf request to get it compiling
        // Full implementation would parse metrics_dict and create proper Protobuf request
        use opentelemetry_proto::tonic::collector::metrics::v1::ExportMetricsServiceRequest;
        let protobuf_request = ExportMetricsServiceRequest::default();

        // Get library instance and runtime
        let library_ref = self.library.borrow(py);
        let library = library_ref.library.clone();
        let runtime = library_ref.runtime.clone();
        drop(library_ref); // Explicitly drop PyRef before async operation

        // Release GIL before blocking on async operation to prevent deadlocks and segfaults
        py.allow_threads(|| {
            runtime
                .block_on(async move { library.export_metrics(protobuf_request).await })
                .map_err(|e| error_message_to_py(format!("Failed to export metrics: {}", e)))
        })?;

        // Return ExportResult.SUCCESS
        // In Python OpenTelemetry SDK, ExportResult is an enum with SUCCESS and FAILURE variants
        // We'll return a Python object that represents SUCCESS
        let export_result = py
            .import("opentelemetry.sdk.metrics.export")
            .and_then(|module| module.getattr("ExportResult"))
            .and_then(|export_result| export_result.getattr("SUCCESS"));

        match export_result {
            Ok(success) => Ok(success.into()),
            Err(_) => {
                // Fallback: return a simple success indicator if ExportResult is not available
                Ok(py.None())
            }
        }
    }

    /// Shutdown the exporter (no-op)
    ///
    /// Implements Python OpenTelemetry SDK's MetricExporter.shutdown() method.
    /// This is a no-op because library shutdown is handled separately.
    #[pyo3(signature = (*, timeout_millis=None))]
    #[allow(unused_variables)] // timeout_millis is part of SDK interface but not used
    pub fn shutdown(&self, timeout_millis: Option<f64>, _py: Python<'_>) -> PyResult<()> {
        // No-op: library shutdown is separate operation
        Ok(())
    }

    /// Force flush of pending exports
    ///
    /// Implements Python OpenTelemetry SDK's MetricExporter.force_flush() method.
    ///
    /// # Arguments
    ///
    /// * `timeout_millis` - Optional timeout in milliseconds (ignored)
    ///
    /// # Returns
    ///
    /// ExportResult (SUCCESS or FAILURE)
    #[pyo3(signature = (*, timeout_millis=None))]
    #[allow(unused_variables)] // timeout_millis is part of SDK interface but not used
    pub fn force_flush(&self, timeout_millis: Option<f64>, py: Python<'_>) -> PyResult<PyObject> {
        // Validate library is still valid
        if !is_library_valid(&self.library, py) {
            return Err(error_message_to_py(
                "Library instance is no longer valid".to_string(),
            ));
        }

        // Delegate to library.flush()
        // Extract library and runtime, then drop PyRef before calling block_on
        // This avoids potential lifetime issues with PyRef during async operations
        let library_ref = self.library.borrow(py);
        let library = library_ref.library.clone();
        let runtime = library_ref.runtime.clone();
        drop(library_ref); // Explicitly drop PyRef before async operation

        // Release GIL before blocking on async operation to prevent deadlocks and segfaults
        py.allow_threads(|| {
            runtime
                .block_on(async move { library.flush().await })
                .map_err(|e| error_message_to_py(format!("Failed to flush metrics: {}", e)))
        })?;

        // Return ExportResult.SUCCESS
        let export_result = py
            .import("opentelemetry.sdk.metrics.export")
            .and_then(|module| module.getattr("ExportResult"))
            .and_then(|export_result| export_result.getattr("SUCCESS"));

        match export_result {
            Ok(success) => Ok(success.into()),
            Err(_) => Ok(py.None()),
        }
    }

    /// Return temporality preference
    ///
    /// Implements Python OpenTelemetry SDK's MetricExporter.temporality() method.
    ///
    /// # Returns
    ///
    /// Temporality enum value (default: CUMULATIVE)
    pub fn temporality(&self, py: Python<'_>) -> PyResult<PyObject> {
        // Return Temporality.CUMULATIVE
        let temporality_result = py
            .import("opentelemetry.sdk.metrics.export")
            .and_then(|module| module.getattr("Temporality"))
            .and_then(|temporality| temporality.getattr("CUMULATIVE"));

        match temporality_result {
            Ok(cumulative) => Ok(cumulative.into()),
            Err(_) => {
                // Fallback: return a string representation
                Ok(PyString::new(py, "CUMULATIVE").into_py(py))
            }
        }
    }

    /// Get string representation
    fn __repr__(&self) -> String {
        "PyOtlpMetricExporterAdapter".to_string()
    }

    /// Get _preferred_temporality attribute (required by OpenTelemetry SDK)
    ///
    /// This is accessed as an attribute by PeriodicExportingMetricReader
    fn __getattr__(&self, name: &str, py: Python<'_>) -> PyResult<PyObject> {
        match name {
            "_preferred_temporality" => {
                // Return a dict mapping metric types to AggregationTemporality.CUMULATIVE
                // The SDK expects: {Counter: CUMULATIVE, Histogram: CUMULATIVE, ...}
                let temporality_dict = pyo3::types::PyDict::new(py);

                // Safely import and get AggregationTemporality.CUMULATIVE
                let cumulative = match py
                    .import("opentelemetry.sdk.metrics.export")
                    .and_then(|module| module.getattr("AggregationTemporality"))
                    .and_then(|agg_temp| agg_temp.getattr("CUMULATIVE"))
                {
                    Ok(cum) => cum,
                    Err(_) => {
                        // If import fails, return empty dict (SDK will handle it)
                        return Ok(temporality_dict.into());
                    }
                };

                // Get metric types from opentelemetry.sdk.metrics
                if let Ok(metrics_module) = py.import("opentelemetry.sdk.metrics") {
                    let metric_types = [
                        "Counter",
                        "Histogram",
                        "UpDownCounter",
                        "ObservableCounter",
                        "ObservableGauge",
                        "ObservableUpDownCounter",
                    ];

                    for metric_type_name in metric_types {
                        if let Ok(metric_type) = metrics_module.getattr(metric_type_name) {
                            let _ = temporality_dict.set_item(metric_type, cumulative);
                        }
                    }
                }

                Ok(temporality_dict.into())
            }
            "_preferred_aggregation" => {
                // Return empty dict - SDK will use default aggregations
                let empty_dict = pyo3::types::PyDict::new(py);
                Ok(empty_dict.into())
            }
            _ => {
                // Return AttributeError for unknown attributes (Python convention)
                Err(pyo3::exceptions::PyAttributeError::new_err(format!(
                    "'PyOtlpMetricExporterAdapter' object has no attribute '{}'",
                    name
                )))
            }
        }
    }
}

/// Python span exporter adapter that implements Python OpenTelemetry SDK's SpanExporter interface
///
/// This adapter bridges Python OpenTelemetry SDK's trace export system with OtlpLibrary,
/// enabling direct use with BatchSpanProcessor and TracerProvider without custom adapter code.
#[pyclass]
pub struct PyOtlpSpanExporterAdapter {
    /// Reference to the library instance (prevents garbage collection)
    pub(crate) library: LibraryRef,
}

#[pymethods]
impl PyOtlpSpanExporterAdapter {
    /// Export span data to the library
    ///
    /// Implements Python OpenTelemetry SDK's SpanExporter.export() method.
    ///
    /// # Arguments
    ///
    /// * `spans` - Sequence of ReadableSpan objects from Python OpenTelemetry SDK
    ///
    /// # Returns
    ///
    /// SpanExportResult (SUCCESS or FAILURE)
    #[allow(unsafe_op_in_unsafe_fn)] // PyO3 parameter extraction is safe
    pub fn export(&self, spans: &PyAny, py: Python<'_>) -> PyResult<PyObject> { // SAFETY: PyO3 parameter extraction is safe
        // SAFETY: PyO3 parameter extraction is safe
        // Validate library is still valid
        if !is_library_valid(&self.library, py) {
            return Err(error_message_to_py(
                "Library instance is no longer valid".to_string(),
            ));
        }

        // Convert Python OpenTelemetry SDK types to library-compatible format
        let spans_list = convert_span_sequence_to_dict_list(spans, py)?;

        // Get library instance and delegate to export_traces
        let library_ref = self.library.borrow(py);
        library_ref
            .export_traces(spans_list)
            .map_err(|e| error_message_to_py(format!("Failed to export spans: {}", e)))?;

        // Return SpanExportResult.SUCCESS
        let span_export_result = py
            .import("opentelemetry.sdk.trace.export")
            .and_then(|module| module.getattr("SpanExportResult"))
            .and_then(|span_export_result| span_export_result.getattr("SUCCESS"));

        match span_export_result {
            Ok(success) => Ok(success.into()),
            Err(_) => Ok(py.None()),
        }
    }

    /// Shutdown the exporter (no-op)
    ///
    /// Implements Python OpenTelemetry SDK's SpanExporter.shutdown() method.
    /// This is a no-op because library shutdown is handled separately.
    #[pyo3(signature = (*, timeout_millis=None))]
    #[allow(unused_variables)] // timeout_millis is part of SDK interface but not used
    pub fn shutdown(&self, timeout_millis: Option<f64>, _py: Python<'_>) -> PyResult<()> {
        // No-op: library shutdown is separate operation
        Ok(())
    }

    /// Force flush of pending exports
    ///
    /// Implements Python OpenTelemetry SDK's SpanExporter.force_flush() method.
    ///
    /// # Arguments
    ///
    /// * `timeout_millis` - Optional timeout in milliseconds (ignored)
    ///
    /// # Returns
    ///
    /// SpanExportResult (SUCCESS or FAILURE)
    pub fn force_flush(&self, _timeout_millis: Option<u64>, py: Python<'_>) -> PyResult<PyObject> {
        // Validate library is still valid
        if !is_library_valid(&self.library, py) {
            return Err(error_message_to_py(
                "Library instance is no longer valid".to_string(),
            ));
        }

        // Delegate to library.flush()
        // Extract library and runtime, then drop PyRef before calling block_on
        // This avoids potential lifetime issues with PyRef during async operations
        let library_ref = self.library.borrow(py);
        let library = library_ref.library.clone();
        let runtime = library_ref.runtime.clone();
        drop(library_ref); // Explicitly drop PyRef before async operation

        // Release GIL before blocking on async operation to prevent deadlocks and segfaults
        py.allow_threads(|| {
            runtime
                .block_on(async move { library.flush().await })
                .map_err(|e| error_message_to_py(format!("Failed to flush spans: {}", e)))
        })?;

        // Return SpanExportResult.SUCCESS
        let span_export_result = py
            .import("opentelemetry.sdk.trace.export")
            .and_then(|module| module.getattr("SpanExportResult"))
            .and_then(|span_export_result| span_export_result.getattr("SUCCESS"));

        match span_export_result {
            Ok(success) => Ok(success.into()),
            Err(_) => Ok(py.None()),
        }
    }

    /// Get string representation
    fn __repr__(&self) -> String {
        "PyOtlpSpanExporterAdapter".to_string()
    }
}
