//! OpenTelemetry Protocol (OTLP) module
//!
//! Provides OTLP message processing, file export, and server functionality.

use std::time::SystemTime;

/// Message type enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessageType {
    /// OpenTelemetry trace data
    Trace,
    /// OpenTelemetry metric data
    Metric,
}

/// OTLP message representation
#[derive(Debug, Clone)]
pub struct OtlpMessage {
    /// Message type (trace or metric)
    pub message_type: MessageType,
    /// Timestamp when message was received
    pub received_at: SystemTime,
}

impl OtlpMessage {
    /// Create a new trace message
    pub fn trace() -> Self {
        Self {
            message_type: MessageType::Trace,
            received_at: SystemTime::now(),
        }
    }

    /// Create a new metric message
    pub fn metric() -> Self {
        Self {
            message_type: MessageType::Metric,
            received_at: SystemTime::now(),
        }
    }
}

pub mod batch_writer;
pub mod converter;
pub mod exporter;
pub mod forwarder;
pub mod metrics_converter;
pub mod metrics_data;
pub mod metrics_extractor;
pub mod server;
pub mod server_arrow;

pub use batch_writer::BatchBuffer;
pub use exporter::{ExportFormat, FileSpanExporter, OtlpFileExporter, OtlpSpanExporter};
pub use server::{MetricsServiceImpl, OtlpGrpcServer, TraceServiceImpl};
pub use server_arrow::OtlpArrowFlightServer;
