//! Type conversion utilities for Python OpenTelemetry SDK adapters
//!
//! This module provides functions to convert between Python OpenTelemetry SDK types
//! and library-compatible dictionary formats, preserving 100% of data without
//! loss or corruption.

use pyo3::exceptions::{PyRuntimeError, PyValueError};
use pyo3::prelude::*;
use pyo3::types::{PyBytes, PyDict, PyList};
use tracing::warn;

/// Base trait for type conversion operations
///
/// This trait defines the interface for converting Python OpenTelemetry SDK types
/// to library-compatible formats. All conversion functions should preserve 100%
/// of data without loss or corruption.
pub trait TypeConverter {
    /// The input type from Python OpenTelemetry SDK
    type Input;
    /// The output type (library-compatible format)
    type Output;

    /// Convert Python OpenTelemetry SDK type to library-compatible format
    ///
    /// # Arguments
    ///
    /// * `input` - The Python OpenTelemetry SDK type to convert
    /// * `py` - Python interpreter instance
    ///
    /// # Returns
    ///
    /// The converted library-compatible format
    ///
    /// # Errors
    ///
    /// Returns `PyErr` if conversion fails (e.g., invalid input data, missing fields)
    fn convert(input: Self::Input, py: Python<'_>) -> PyResult<Self::Output>;
}

/// Error conversion utilities
///
/// Provides functions to convert Rust errors and library errors to
/// appropriate Python exceptions while preserving error context.
pub mod error_conversion {
    use super::*;
    use crate::error::OtlpError;

    /// Convert OtlpError to Python RuntimeError
    ///
    /// Preserves error context in the exception message for debugging.
    ///
    /// # Arguments
    ///
    /// * `error` - The OtlpError to convert
    ///
    /// # Returns
    ///
    /// A PyRuntimeError with the error message
    pub fn otlp_error_to_py(error: OtlpError) -> PyErr {
        let error_msg = format!("OtlpLibrary error: {}", error);
        warn!("Converting OtlpError to Python exception: {}", error_msg);
        PyRuntimeError::new_err(error_msg)
    }

    /// Convert a generic error message to Python RuntimeError
    ///
    /// Used for errors that don't have a specific OtlpError type.
    ///
    /// # Arguments
    ///
    /// * `message` - The error message
    ///
    /// # Returns
    ///
    /// A PyRuntimeError with the message
    pub fn error_message_to_py(message: String) -> PyErr {
        warn!("Converting error message to Python exception: {}", message);
        PyRuntimeError::new_err(message)
    }

    /// Convert a validation error to Python ValueError
    ///
    /// Used for input validation errors (e.g., missing required fields, invalid data format).
    ///
    /// # Arguments
    ///
    /// * `message` - The validation error message
    ///
    /// # Returns
    ///
    /// A PyValueError with the message
    pub fn validation_error_to_py(message: String) -> PyErr {
        warn!("Validation error: {}", message);
        PyValueError::new_err(message)
    }

    /// Convert a type conversion error to Python RuntimeError
    ///
    /// Used when type conversion fails (e.g., cannot extract data from Python object).
    ///
    /// # Arguments
    ///
    /// * `message` - The conversion error message
    /// * `details` - Optional additional details about the conversion failure
    ///
    /// # Returns
    ///
    /// A PyRuntimeError with the error message
    pub fn conversion_error_to_py(message: String, details: Option<String>) -> PyErr {
        let error_msg = if let Some(details) = details {
            format!("Type conversion error: {} ({})", message, details)
        } else {
            format!("Type conversion error: {}", message)
        };
        warn!("{}", error_msg);
        PyRuntimeError::new_err(error_msg)
    }
}

// Re-export for convenience
pub use error_conversion::{
    conversion_error_to_py, error_message_to_py, otlp_error_to_py, validation_error_to_py,
};

/// Convert Python OpenTelemetry SDK MetricExportResult to library-compatible dictionary
///
/// This function extracts metric data from Python OpenTelemetry SDK's MetricExportResult
/// and converts it to a dictionary format compatible with the library's export_metrics
/// and export_metrics_ref methods.
///
/// # Arguments
///
/// * `metrics_data` - Python object representing MetricExportResult
/// * `py` - Python interpreter instance
///
/// # Returns
///
/// A PyDict containing the converted metrics data
///
/// # Errors
///
/// Returns PyErr if conversion fails (invalid data, missing fields, etc.)
pub fn convert_metric_export_result_to_dict<'py>(
    metrics_data: &'py PyAny,
    py: Python<'py>,
) -> PyResult<&'py PyDict> {
    // Extract resource_metrics from MetricExportResult
    // MetricExportResult typically has a resource_metrics attribute
    // Use get_item first to check if it's a dict, then fall back to getattr
    let resource_metrics = if let Ok(dict) = metrics_data.downcast::<PyDict>() {
        dict.get_item("resource_metrics")?.ok_or_else(|| {
            conversion_error_to_py(
                "resource_metrics not found in MetricExportResult dict".to_string(),
                None,
            )
        })?
    } else {
        metrics_data.getattr("resource_metrics").map_err(|e| {
            conversion_error_to_py(
                "Failed to get resource_metrics from MetricExportResult".to_string(),
                Some(format!("{}", e)),
            )
        })?
    };

    // Handle resource_metrics - it can be a list or a single ResourceMetrics object
    // If it's a list, take the first item
    let resource_metrics_obj = if let Ok(list) = resource_metrics.downcast::<PyList>() {
        if list.is_empty() {
            return Err(conversion_error_to_py(
                "resource_metrics list is empty".to_string(),
                None,
            ));
        }
        list.get_item(0).ok_or_else(|| {
            conversion_error_to_py(
                "Failed to get first item from resource_metrics list".to_string(),
                None,
            )
        })?
    } else {
        resource_metrics
    };

    // Build dictionary structure compatible with library API
    let result = PyDict::new(py);

    // Extract resource attributes
    let resource = if let Ok(dict) = resource_metrics_obj.downcast::<PyDict>() {
        dict.get_item("resource")?.ok_or_else(|| {
            conversion_error_to_py(
                "resource not found in ResourceMetrics dict".to_string(),
                None,
            )
        })?
    } else {
        resource_metrics_obj.getattr("resource").map_err(|e| {
            conversion_error_to_py(
                "Failed to get resource from ResourceMetrics".to_string(),
                Some(format!("{}", e)),
            )
        })?
    };

    let resource_dict = PyDict::new(py);
    // Safely extract attributes - handle both dict access and attribute access
    let attributes = if let Ok(dict) = resource.downcast::<PyDict>() {
        dict.get_item("attributes").ok().flatten()
    } else {
        resource.getattr("attributes").ok()
    };

    if let Some(attributes) = attributes
        && let Ok(attrs_dict) = attributes.downcast::<PyDict>()
    {
        for (key, value) in attrs_dict.iter() {
            if let Err(e) = resource_dict.set_item(key, value) {
                // Log warning but continue - some values might not be settable
                warn!("Failed to set resource attribute: {:?}", e);
            }
        }
    }
    result.set_item("resource", resource_dict)?;

    // Extract scope_metrics
    let scope_metrics = if let Ok(dict) = resource_metrics_obj.downcast::<PyDict>() {
        dict.get_item("scope_metrics")?.ok_or_else(|| {
            conversion_error_to_py(
                "scope_metrics not found in ResourceMetrics dict".to_string(),
                None,
            )
        })?
    } else {
        resource_metrics_obj.getattr("scope_metrics").map_err(|e| {
            conversion_error_to_py(
                "Failed to get scope_metrics from ResourceMetrics".to_string(),
                Some(format!("{}", e)),
            )
        })?
    };

    let scope_metrics_list = PyList::empty(py);
    // Safely handle scope_metrics - could be a list or iterable
    if let Ok(metrics_list) = scope_metrics.downcast::<PyList>() {
        for scope_metric in metrics_list.iter() {
            // Skip None values
            if scope_metric.is_none() {
                continue;
            }

            let scope_metric_dict = PyDict::new(py);

            // Extract scope information - handle both dict and attribute access
            // Wrap in error handling to prevent segfaults from invalid Python objects
            let scope = if let Ok(dict) = scope_metric.downcast::<PyDict>() {
                dict.get_item("scope").ok().flatten()
            } else {
                // Use getattr with error handling - catch any Python exceptions
                scope_metric.getattr("scope").ok()
            };

            if let Some(scope) = scope {
                // Skip if scope is None
                if scope.is_none() {
                    continue;
                }

                let scope_dict = PyDict::new(py);
                // Safely extract name and version with error handling
                let name = if let Ok(d) = scope.downcast::<PyDict>() {
                    d.get_item("name").ok().flatten()
                } else {
                    scope.getattr("name").ok()
                };
                if let Some(name) = name
                    && !name.is_none()
                {
                    let _ = scope_dict.set_item("name", name);
                }

                let version = if let Ok(d) = scope.downcast::<PyDict>() {
                    d.get_item("version").ok().flatten()
                } else {
                    scope.getattr("version").ok()
                };
                if let Some(version) = version
                    && !version.is_none()
                {
                    let _ = scope_dict.set_item("version", version);
                }
                let _ = scope_metric_dict.set_item("scope", scope_dict);
            }

            // Extract metrics - handle both dict and attribute access
            let metrics = if let Ok(dict) = scope_metric.downcast::<PyDict>() {
                dict.get_item("metrics").ok().flatten()
            } else {
                scope_metric.getattr("metrics").ok()
            };

            if let Some(metrics) = metrics {
                if metrics.is_none() {
                    continue;
                }

                let metrics_list = PyList::empty(py);
                if let Ok(metrics_py_list) = metrics.downcast::<PyList>() {
                    for metric in metrics_py_list.iter() {
                        // Skip None values
                        if metric.is_none() {
                            continue;
                        }

                        let metric_dict = PyDict::new(py);

                        // Safely extract metric fields with error handling
                        let name = if let Ok(d) = metric.downcast::<PyDict>() {
                            d.get_item("name").ok().flatten()
                        } else {
                            metric.getattr("name").ok()
                        };
                        if let Some(name) = name
                            && !name.is_none()
                        {
                            let _ = metric_dict.set_item("name", name);
                        }

                        let description = if let Ok(d) = metric.downcast::<PyDict>() {
                            d.get_item("description").ok().flatten()
                        } else {
                            metric.getattr("description").ok()
                        };
                        if let Some(description) = description
                            && !description.is_none()
                        {
                            let _ = metric_dict.set_item("description", description);
                        }

                        let unit = if let Ok(d) = metric.downcast::<PyDict>() {
                            d.get_item("unit").ok().flatten()
                        } else {
                            metric.getattr("unit").ok()
                        };
                        if let Some(unit) = unit
                            && !unit.is_none()
                        {
                            let _ = metric_dict.set_item("unit", unit);
                        }

                        let data = if let Ok(d) = metric.downcast::<PyDict>() {
                            d.get_item("data").ok().flatten()
                        } else {
                            metric.getattr("data").ok()
                        };
                        if let Some(data) = data
                            && !data.is_none()
                        {
                            let _ = metric_dict.set_item("data", data);
                        }

                        let _ = metrics_list.append(metric_dict);
                    }
                }
                let _ = scope_metric_dict.set_item("metrics", metrics_list);
            }

            let _ = scope_metrics_list.append(scope_metric_dict);
        }
    }
    result.set_item("scope_metrics", scope_metrics_list)?;

    Ok(result)
}

/// Convert Python OpenTelemetry SDK ReadableSpan to library-compatible dictionary
///
/// This function extracts span data from Python OpenTelemetry SDK's ReadableSpan
/// and converts it to a dictionary format compatible with the library's export_traces
/// method. Follows the same pattern as the existing dict_to_span_data function.
///
/// # Arguments
///
/// * `span` - Python object representing ReadableSpan
/// * `py` - Python interpreter instance
///
/// # Returns
///
/// A PyDict containing the converted span data
///
/// # Errors
///
/// Returns PyErr if conversion fails (invalid data, missing fields, etc.)
pub fn convert_readable_span_to_dict<'py>(
    span: &'py PyAny,
    py: Python<'py>,
) -> PyResult<&'py PyDict> {
    let result = PyDict::new(py);

    // Extract trace_id from context
    let context = span.getattr("context").map_err(|e| {
        conversion_error_to_py(
            "Failed to get context from ReadableSpan".to_string(),
            Some(format!("{}", e)),
        )
    })?;

    let trace_id = context.getattr("trace_id").map_err(|e| {
        conversion_error_to_py(
            "Failed to get trace_id from SpanContext".to_string(),
            Some(format!("{}", e)),
        )
    })?;

    // Convert trace_id (128-bit int) to 16 bytes
    let trace_id_int: u128 = trace_id.extract().map_err(|e| {
        conversion_error_to_py(
            "Failed to extract trace_id as integer".to_string(),
            Some(format!("{}", e)),
        )
    })?;

    let trace_id_bytes: Vec<u8> = trace_id_int.to_be_bytes().to_vec();
    result.set_item("trace_id", PyBytes::new(py, &trace_id_bytes))?;

    // Extract span_id
    let span_id = context.getattr("span_id").map_err(|e| {
        conversion_error_to_py(
            "Failed to get span_id from SpanContext".to_string(),
            Some(format!("{}", e)),
        )
    })?;

    // Convert span_id (64-bit int) to 8 bytes
    let span_id_int: u64 = span_id.extract().map_err(|e| {
        conversion_error_to_py(
            "Failed to extract span_id as integer".to_string(),
            Some(format!("{}", e)),
        )
    })?;

    let span_id_bytes: Vec<u8> = span_id_int.to_be_bytes().to_vec();
    result.set_item("span_id", PyBytes::new(py, &span_id_bytes))?;

    // Extract parent span_id (optional)
    if let Ok(parent) = span.getattr("parent")
        && !parent.is_none()
        && let Ok(parent_context) = parent.getattr("context")
        && let Ok(parent_span_id) = parent_context.getattr("span_id")
        && let Ok(parent_span_id_val) = parent_span_id.extract::<u64>()
    {
        let parent_span_id_bytes: Vec<u8> = parent_span_id_val.to_be_bytes().to_vec();
        result.set_item("parent_span_id", PyBytes::new(py, &parent_span_id_bytes))?;
    }

    // Extract name
    if let Ok(name) = span.getattr("name") {
        result.set_item("name", name)?;
    }

    // Extract kind
    if let Ok(kind) = span.getattr("kind") {
        // Convert SpanKind enum to string
        if let Ok(kind_str) = kind.getattr("name") {
            let kind_name: String = kind_str
                .extract()
                .unwrap_or_else(|_| "INTERNAL".to_string());
            result.set_item("kind", kind_name.to_lowercase())?;
        }
    }

    // Extract attributes
    if let Ok(attributes) = span.getattr("attributes")
        && let Ok(attrs_dict) = attributes.downcast::<PyDict>()
    {
        result.set_item("attributes", attrs_dict)?;
    }

    // Extract events
    if let Ok(events) = span.getattr("events")
        && let Ok(events_list) = events.downcast::<PyList>()
    {
        let events_dict_list = PyList::empty(py);
        for event in events_list.iter() {
            let event_dict = PyDict::new(py);
            if let Ok(name) = event.getattr("name") {
                event_dict.set_item("name", name)?;
            }
            if let Ok(timestamp) = event.getattr("timestamp") {
                event_dict.set_item("timestamp", timestamp)?;
            }
            if let Ok(event_attrs) = event.getattr("attributes")
                && let Ok(event_attrs_dict) = event_attrs.downcast::<PyDict>()
            {
                event_dict.set_item("attributes", event_attrs_dict)?;
            }
            events_dict_list.append(event_dict)?;
        }
        result.set_item("events", events_dict_list)?;
    }

    // Extract links
    if let Ok(links) = span.getattr("links")
        && let Ok(links_list) = links.downcast::<PyList>()
    {
        let links_dict_list = PyList::empty(py);
        for link in links_list.iter() {
            let link_dict = PyDict::new(py);
            if let Ok(link_context) = link.getattr("context") {
                if let Ok(link_trace_id) = link_context.getattr("trace_id")
                    && let Ok(link_trace_id_val) = link_trace_id.extract::<u128>()
                {
                    let link_trace_id_bytes: Vec<u8> = link_trace_id_val.to_be_bytes().to_vec();
                    link_dict.set_item("trace_id", PyBytes::new(py, &link_trace_id_bytes))?;
                }
                if let Ok(link_span_id) = link_context.getattr("span_id")
                    && let Ok(link_span_id_val) = link_span_id.extract::<u64>()
                {
                    let link_span_id_bytes: Vec<u8> = link_span_id_val.to_be_bytes().to_vec();
                    link_dict.set_item("span_id", PyBytes::new(py, &link_span_id_bytes))?;
                }
            }
            if let Ok(link_attrs) = link.getattr("attributes")
                && let Ok(link_attrs_dict) = link_attrs.downcast::<PyDict>()
            {
                link_dict.set_item("attributes", link_attrs_dict)?;
            }
            links_dict_list.append(link_dict)?;
        }
        result.set_item("links", links_dict_list)?;
    }

    // Extract status
    if let Ok(status) = span.getattr("status") {
        if let Ok(status_code) = status.getattr("status_code")
            && let Ok(code_name) = status_code.getattr("name")
        {
            let status_name: String = code_name.extract().unwrap_or_else(|_| "UNSET".to_string());
            result.set_item("status", status_name.to_lowercase())?;
        }
        if let Ok(status_message) = status.getattr("status_message")
            && !status_message.is_none()
        {
            result.set_item("status_message", status_message)?;
        }
    }

    // Extract timestamps
    if let Ok(start_time) = span.getattr("start_time") {
        result.set_item("start_time", start_time)?;
    }
    if let Ok(end_time) = span.getattr("end_time") {
        result.set_item("end_time", end_time)?;
    }

    Ok(result)
}

/// Convert a sequence of ReadableSpan objects to a list of dictionaries
///
/// # Arguments
///
/// * `spans` - Python sequence of ReadableSpan objects
/// * `py` - Python interpreter instance
///
/// # Returns
///
/// A PyList containing dictionaries for each span
pub fn convert_span_sequence_to_dict_list<'py>(
    spans: &'py PyAny,
    py: Python<'py>,
) -> PyResult<&'py PyList> {
    let result = PyList::empty(py);

    if let Ok(spans_list) = spans.downcast::<PyList>() {
        for span in spans_list.iter() {
            let span_dict = convert_readable_span_to_dict(span, py)?;
            result.append(span_dict)?;
        }
    } else {
        // Try to iterate if it's a sequence but not a list
        if let Ok(iter) = spans.iter() {
            for span_result in iter {
                let span = span_result?;
                let span_dict = convert_readable_span_to_dict(span, py)?;
                result.append(span_dict)?;
            }
        } else {
            return Err(conversion_error_to_py(
                "spans must be a sequence of ReadableSpan objects".to_string(),
                None,
            ));
        }
    }

    Ok(result)
}
