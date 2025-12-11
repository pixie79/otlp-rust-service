//! File-based OTLP exporter
//!
//! Writes OTLP traces and metrics to files in Arrow IPC Streaming format with rotation and cleanup.
//!
//! Based on cap-gl-consumer-rust/src/otlp/file_exporter.rs patterns.

use crate::config::Config;
use crate::error::{OtlpError, OtlpExportError};
use anyhow::Result;
use arrow::array::*;
use arrow::datatypes::*;
use arrow::ipc::writer::StreamWriter;
use futures::FutureExt;
use futures::future::BoxFuture;
use opentelemetry_sdk::trace::{SpanData, SpanExporter};
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{info, trace, warn};

/// Export format for OTLP data
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExportFormat {
    /// Arrow IPC stream format
    Arrow,
}

/// File-based OTLP exporter for traces and metrics
#[derive(Clone)]
pub struct OtlpFileExporter {
    traces_writer: Arc<Mutex<TracesWriter>>,
    metrics_writer: Arc<Mutex<MetricsWriter>>,
    output_dir: PathBuf,
    max_file_size: u64,
    format: ExportFormat,
    forwarder: Option<Arc<crate::otlp::forwarder::OtlpForwarder>>,
    // Metrics collection - grouped to reduce lock acquisitions
    metrics: Arc<Mutex<ExporterMetrics>>,
    /// Temporality mode for metric exporters (default: Cumulative)
    temporality: opentelemetry_sdk::metrics::Temporality,
}

/// Grouped exporter metrics to reduce lock contention
#[derive(Debug, Default)]
struct ExporterMetrics {
    messages_received: u64,
    files_written: u64,
    errors_count: u64,
    format_conversions: u64,
}

impl std::fmt::Debug for OtlpFileExporter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OtlpFileExporter")
            .field("output_dir", &self.output_dir)
            .field("max_file_size", &self.max_file_size)
            .field("format", &self.format)
            .finish_non_exhaustive()
    }
}

struct TracesWriter {
    current_file: Option<File>,
    current_writer: Option<StreamWriter<std::io::BufWriter<File>>>,
    current_schema: Option<Arc<Schema>>,
    current_size: u64,
    output_dir: PathBuf,
    sequence: u64,
}

struct MetricsWriter {
    current_file: Option<File>,
    current_writer: Option<StreamWriter<std::io::BufWriter<File>>>,
    current_schema: Option<Arc<Schema>>,
    current_size: u64,
    output_dir: PathBuf,
    sequence: u64,
}

impl OtlpFileExporter {
    /// Create a new file-based OTLP exporter
    pub fn new(config: &Config) -> Result<Self, OtlpError> {
        let output_dir = config.output_dir.join("otlp");
        let max_file_size = 100 * 1024 * 1024; // 100MB default

        // Create output directory if it doesn't exist
        std::fs::create_dir_all(&output_dir).map_err(OtlpError::Io)?;

        info!(
            output_dir = %output_dir.display(),
            max_file_size_mb = max_file_size / 1024 / 1024,
            "Initializing OTLP file exporter"
        );

        // Create subdirectories for traces and metrics
        let traces_dir = output_dir.join("traces");
        let metrics_dir = output_dir.join("metrics");
        std::fs::create_dir_all(&traces_dir).map_err(OtlpError::Io)?;
        std::fs::create_dir_all(&metrics_dir).map_err(OtlpError::Io)?;

        // Create forwarder if forwarding is enabled
        let forwarder = if let Some(ref forwarding_config) = config.forwarding {
            if forwarding_config.enabled {
                match crate::otlp::forwarder::OtlpForwarder::new(forwarding_config.clone()) {
                    Ok(f) => {
                        info!(
                            "Forwarding enabled to {}",
                            forwarding_config
                                .endpoint_url
                                .as_ref()
                                .unwrap_or(&"unknown".to_string())
                        );
                        Some(Arc::new(f))
                    }
                    Err(e) => {
                        warn!(error = %e, "Failed to create forwarder, continuing without forwarding");
                        None
                    }
                }
            } else {
                None
            }
        } else {
            None
        };

        let exporter = Self {
            traces_writer: Arc::new(Mutex::new(TracesWriter {
                current_file: None,
                current_writer: None,
                current_schema: None,
                current_size: 0,
                output_dir: traces_dir,
                sequence: 0,
            })),
            metrics_writer: Arc::new(Mutex::new(MetricsWriter {
                current_file: None,
                current_writer: None,
                current_schema: None,
                current_size: 0,
                output_dir: metrics_dir,
                sequence: 0,
            })),
            output_dir,
            max_file_size,
            format: ExportFormat::Arrow,
            forwarder,
            metrics: Arc::new(Mutex::new(ExporterMetrics::default())),
            temporality: config
                .metric_temporality
                .unwrap_or(opentelemetry_sdk::metrics::Temporality::Cumulative),
        };

        Ok(exporter)
    }

    /// Export traces to file
    pub async fn export_traces(&self, spans: Vec<SpanData>) -> Result<(), OtlpError> {
        if spans.is_empty() {
            return Ok(());
        }

        // Update metrics (single lock acquisition)
        {
            let mut metrics = self.metrics.lock().await;
            metrics.messages_received += spans.len() as u64;
        }

        // Convert spans to Arrow RecordBatch
        let batch = convert_spans_to_arrow_batch(&spans)
            .map_err(|e| OtlpError::Export(OtlpExportError::ArrowConversionError(e.to_string())))?;

        // Write batch directly to file using persistent StreamWriter
        self.write_traces_arrow_batch(&batch).await?;

        // Forward traces if forwarding is enabled
        if let Some(ref forwarder) = self.forwarder {
            if let Err(e) = forwarder.forward_traces(spans).await {
                warn!(error = %e, "Failed to forward traces, but local storage succeeded");
                // Update error metrics (single lock acquisition)
                {
                    let mut metrics = self.metrics.lock().await;
                    metrics.errors_count += 1;
                }
                // Don't fail - forwarding is best-effort
            } else {
                // Update format conversion metrics if conversion occurred (single lock acquisition)
                {
                    let mut metrics = self.metrics.lock().await;
                    metrics.format_conversions += 1;
                }
            }
        }

        Ok(())
    }

    /// Write Arrow RecordBatch to traces file using persistent StreamWriter
    ///
    /// Uses a persistent StreamWriter that writes multiple batches to the same stream.
    /// Only finishes the stream when rotating the file.
    pub(crate) async fn write_traces_arrow_batch(
        &self,
        batch: &arrow::record_batch::RecordBatch,
    ) -> Result<(), OtlpError> {
        let mut writer = self.traces_writer.lock().await;
        // Clone output_dir to avoid borrow checker issues
        let output_dir = writer.output_dir.clone();
        let schema = batch.schema();

        // Estimate batch size (rough estimate)
        let estimated_size = batch.get_array_memory_size() as u64;

        // Check if we need to rotate (before writing)
        if writer.current_size + estimated_size > self.max_file_size {
            // Finish current writer before rotating
            if writer.current_writer.is_some() {
                // Save schema reference before rotation
                let saved_schema = writer.current_schema.as_ref().cloned();
                writer
                    .rotate_file(&output_dir, "traces", self.format)
                    .map_err(|e| OtlpError::Io(std::io::Error::other(e.to_string())))?;
                // Reopen with the same schema
                if let Some(schema) = saved_schema {
                    writer
                        .open_new_file(&output_dir, "traces", self.format, schema)
                        .map_err(|e| OtlpError::Io(std::io::Error::other(e.to_string())))?;
                }
            }
        }

        // Open file and create StreamWriter if needed
        if writer.current_writer.is_none() {
            writer
                .open_new_file(&output_dir, "traces", self.format, schema)
                .map_err(|e| OtlpError::Io(std::io::Error::other(e.to_string())))?;
        }

        // Write batch to StreamWriter
        if let Some(ref mut stream_writer) = writer.current_writer {
            stream_writer.write(batch).map_err(|e| {
                OtlpError::Export(OtlpExportError::ArrowConversionError(format!(
                    "Failed to write Arrow batch: {}",
                    e
                )))
            })?;

            // Flush the underlying file
            stream_writer.get_mut().flush().map_err(OtlpError::Io)?;

            writer.current_size += estimated_size;
            trace!(
                "Wrote {} spans ({} rows, ~{} bytes) to trace file",
                batch.num_rows(),
                batch.num_rows(),
                estimated_size
            );

            // Update metrics (single lock acquisition)
            {
                let mut metrics = self.metrics.lock().await;
                metrics.files_written += 1;
            }
        }

        Ok(())
    }

    /// Export metrics from Protobuf format
    ///
    /// Converts Protobuf ExportMetricsServiceRequest to Arrow IPC and writes to file.
    ///
    /// This method uses our InternalResourceMetrics (with public fields) - NO PROXY NEEDED.
    /// Flow: Protobuf → InternalResourceMetrics → Arrow RecordBatch → Arrow IPC
    /// Internal format: Arrow RecordBatch (for storage)
    pub(crate) async fn export_metrics_from_protobuf(
        &self,
        request: &opentelemetry_proto::tonic::collector::metrics::v1::ExportMetricsServiceRequest,
    ) -> Result<(), OtlpError> {
        // Check if metrics are empty - if so, return early (empty metrics are valid)
        if request.resource_metrics.is_empty() {
            return Ok(());
        }

        use crate::otlp::metrics_extractor::extract_from_protobuf;

        // Convert Protobuf → InternalResourceMetrics (our structure with public fields, no proxy)
        let internal_metrics = extract_from_protobuf(request).map_err(|e| {
            OtlpError::Export(OtlpExportError::FormatConversionError(format!(
                "Failed to extract metrics from protobuf: {}",
                e
            )))
        })?;

        // Convert InternalResourceMetrics → Arrow (direct conversion, no proxy)
        let arrow_batch = internal_metrics.to_arrow_batch().map_err(|e| {
            OtlpError::Export(OtlpExportError::ArrowConversionError(format!(
                "Failed to convert metrics to Arrow: {}",
                e
            )))
        })?;

        // Write Arrow batch directly to file using persistent StreamWriter
        self.write_metrics_arrow_batch(&arrow_batch).await
    }

    /// Write Arrow RecordBatch to metrics file using persistent StreamWriter
    ///
    /// Uses a persistent StreamWriter that writes multiple batches to the same stream.
    /// Only finishes the stream when rotating the file.
    pub(crate) async fn write_metrics_arrow_batch(
        &self,
        batch: &arrow::record_batch::RecordBatch,
    ) -> Result<(), OtlpError> {
        let mut writer = self.metrics_writer.lock().await;
        // Clone output_dir to avoid borrow checker issues
        let output_dir = writer.output_dir.clone();
        let schema = batch.schema();

        // Estimate batch size (rough estimate)
        let estimated_size = batch.get_array_memory_size() as u64;

        // Check if we need to rotate (before writing)
        if writer.current_size + estimated_size > self.max_file_size {
            // Finish current writer before rotating
            if writer.current_writer.is_some() {
                // Save schema reference before rotation
                let saved_schema = writer.current_schema.as_ref().cloned();
                writer
                    .rotate_file(&output_dir, "metrics", self.format)
                    .map_err(|e| OtlpError::Io(std::io::Error::other(e.to_string())))?;
                // Reopen with the same schema
                if let Some(schema) = saved_schema {
                    writer
                        .open_new_file(&output_dir, "metrics", self.format, schema)
                        .map_err(|e| OtlpError::Io(std::io::Error::other(e.to_string())))?;
                }
            }
        }

        // Open file and create StreamWriter if needed
        if writer.current_writer.is_none() {
            writer
                .open_new_file(&output_dir, "metrics", self.format, schema)
                .map_err(|e| OtlpError::Io(std::io::Error::other(e.to_string())))?;
        }

        // Write batch to StreamWriter
        if let Some(ref mut stream_writer) = writer.current_writer {
            stream_writer.write(batch).map_err(|e| {
                OtlpError::Export(OtlpExportError::ArrowConversionError(format!(
                    "Failed to write Arrow batch: {}",
                    e
                )))
            })?;

            // Flush the underlying file
            stream_writer.get_mut().flush().map_err(OtlpError::Io)?;

            writer.current_size += estimated_size;
            trace!(
                "Wrote metrics batch ({} rows, ~{} bytes) to file",
                batch.num_rows(),
                estimated_size
            );

            // Update metrics (single lock acquisition)
            {
                let mut metrics = self.metrics.lock().await;
                metrics.files_written += 1;
            }
        }

        Ok(())
    }

    /// Get library operation metrics
    ///
    /// Returns current counts for:
    /// - Messages received (traces + metrics)
    /// - Files written
    /// - Errors encountered
    /// - Format conversions performed
    ///
    /// # Returns
    ///
    /// Returns a tuple of (messages_received, files_written, errors_count, format_conversions)
    pub async fn get_metrics(&self) -> (u64, u64, u64, u64) {
        let metrics = self.metrics.lock().await;
        (
            metrics.messages_received,
            metrics.files_written,
            metrics.errors_count,
            metrics.format_conversions,
        )
    }

    /// Flush all pending writes
    pub async fn flush(&self) -> Result<(), OtlpError> {
        let mut traces_writer = self.traces_writer.lock().await;
        if let Some(ref mut stream_writer) = traces_writer.current_writer {
            stream_writer.get_mut().flush().map_err(OtlpError::Io)?;
        }

        let mut metrics_writer = self.metrics_writer.lock().await;
        if let Some(ref mut stream_writer) = metrics_writer.current_writer {
            stream_writer.get_mut().flush().map_err(OtlpError::Io)?;
        }

        Ok(())
    }

    /// Clean up old trace files based on modification time
    pub async fn cleanup_traces(&self, cleanup_interval_secs: u64) -> Result<(), OtlpError> {
        let traces_dir = self.traces_writer.lock().await.output_dir.clone();
        Self::cleanup_old_files(&traces_dir, cleanup_interval_secs, "traces").await
    }

    /// Clean up old metric files based on modification time
    pub async fn cleanup_metrics(&self, cleanup_interval_secs: u64) -> Result<(), OtlpError> {
        let metrics_dir = self.metrics_writer.lock().await.output_dir.clone();
        Self::cleanup_old_files(&metrics_dir, cleanup_interval_secs, "metrics").await
    }

    /// Clean up old files in a directory based on modification time
    async fn cleanup_old_files(
        dir: &Path,
        cleanup_interval_secs: u64,
        file_type: &str,
    ) -> Result<(), OtlpError> {
        use std::time::{Duration, SystemTime};

        let cutoff_time = SystemTime::now()
            .checked_sub(Duration::from_secs(cleanup_interval_secs))
            .ok_or_else(|| {
                OtlpError::Export(OtlpExportError::CleanupError(
                    "Invalid cleanup interval".to_string(),
                ))
            })?;

        let entries = std::fs::read_dir(dir).map_err(OtlpError::Io)?;

        let mut deleted_count = 0;
        let mut error_count = 0;

        for entry in entries {
            let entry = match entry {
                Ok(e) => e,
                Err(e) => {
                    warn!("Failed to read directory entry: {}", e);
                    error_count += 1;
                    continue;
                }
            };

            let path = entry.path();

            // Only process files, not directories
            if !path.is_file() {
                continue;
            }

            // Only process .arrow files
            if path.extension().and_then(|s| s.to_str()) != Some("arrow") {
                continue;
            }

            // Check file modification time
            let metadata = match entry.metadata() {
                Ok(m) => m,
                Err(e) => {
                    warn!("Failed to read metadata for {}: {}", path.display(), e);
                    error_count += 1;
                    continue;
                }
            };

            let modified = match metadata.modified() {
                Ok(m) => m,
                Err(e) => {
                    warn!(
                        "Failed to get modification time for {}: {}",
                        path.display(),
                        e
                    );
                    error_count += 1;
                    continue;
                }
            };

            // Delete file if it's older than the cutoff time
            if modified < cutoff_time {
                match std::fs::remove_file(&path) {
                    Ok(()) => {
                        deleted_count += 1;
                        trace!(
                            file = %path.display(),
                            age_secs = cleanup_interval_secs,
                            "Deleted old {} file",
                            file_type
                        );
                    }
                    Err(e) => {
                        warn!("Failed to delete old file {}: {}", path.display(), e);
                        error_count += 1;
                    }
                }
            }
        }

        if deleted_count > 0 {
            info!(
                file_type = file_type,
                deleted = deleted_count,
                errors = error_count,
                "Cleaned up old {} files",
                file_type
            );
        }

        if error_count > 0 {
            warn!(
                file_type = file_type,
                errors = error_count,
                "Encountered {} errors during {} cleanup",
                error_count,
                file_type
            );
        }

        Ok(())
    }

    /// Get the configured temporality mode for metric exporters
    ///
    /// This method is required by the OpenTelemetry SDK's PushMetricExporter trait.
    pub fn temporality(&self) -> opentelemetry_sdk::metrics::Temporality {
        self.temporality
    }
}

impl TracesWriter {
    fn open_new_file(
        &mut self,
        output_dir: &Path,
        prefix: &str,
        _format: ExportFormat,
        schema: Arc<Schema>,
    ) -> Result<()> {
        use std::io::BufWriter;

        let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
        let extension = "arrows";
        let filename = format!(
            "otlp_{}_{}_{:04}.{}",
            prefix, timestamp, self.sequence, extension
        );
        let file_path = output_dir.join(&filename);

        let file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&file_path)
            .map_err(|e| {
                anyhow::anyhow!("Failed to create OTLP file {}: {}", file_path.display(), e)
            })?;

        let buf_writer = BufWriter::new(file);
        let stream_writer = StreamWriter::try_new(buf_writer, &schema)
            .map_err(|e| anyhow::anyhow!("Failed to create Arrow StreamWriter: {}", e))?;

        self.current_file = None; // We use the file through the StreamWriter now
        self.current_writer = Some(stream_writer);
        self.current_schema = Some(schema);
        self.current_size = 0;

        info!(
            file = %file_path.display(),
            "Opened new OTLP {} file with StreamWriter",
            prefix
        );

        Ok(())
    }

    fn rotate_file(
        &mut self,
        _output_dir: &Path,
        _prefix: &str,
        _format: ExportFormat,
    ) -> Result<()> {
        // Finish the current StreamWriter if it exists
        if let Some(ref mut stream_writer) = self.current_writer {
            stream_writer
                .finish()
                .map_err(|e| anyhow::anyhow!("Failed to finish Arrow StreamWriter: {}", e))?;
        }

        self.current_writer = None;
        self.current_schema = None;
        self.current_file = None;
        self.sequence += 1;

        // We need the schema to open a new file, but we don't have it here
        // This will be handled by the caller providing the schema
        Ok(())
    }
}

impl MetricsWriter {
    fn open_new_file(
        &mut self,
        output_dir: &Path,
        prefix: &str,
        _format: ExportFormat,
        schema: Arc<Schema>,
    ) -> Result<()> {
        use std::io::BufWriter;

        let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
        let extension = "arrows";
        let filename = format!(
            "otlp_{}_{}_{:04}.{}",
            prefix, timestamp, self.sequence, extension
        );
        let file_path = output_dir.join(&filename);

        let file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&file_path)
            .map_err(|e| {
                anyhow::anyhow!("Failed to create OTLP file {}: {}", file_path.display(), e)
            })?;

        let buf_writer = BufWriter::new(file);
        let stream_writer = StreamWriter::try_new(buf_writer, &schema)
            .map_err(|e| anyhow::anyhow!("Failed to create Arrow StreamWriter: {}", e))?;

        self.current_file = None; // We use the file through the StreamWriter now
        self.current_writer = Some(stream_writer);
        self.current_schema = Some(schema);
        self.current_size = 0;

        info!(
            file = %file_path.display(),
            "Opened new OTLP {} file with StreamWriter",
            prefix
        );

        Ok(())
    }

    fn rotate_file(
        &mut self,
        _output_dir: &Path,
        _prefix: &str,
        _format: ExportFormat,
    ) -> Result<()> {
        // Finish the current StreamWriter if it exists
        if let Some(ref mut stream_writer) = self.current_writer {
            stream_writer
                .finish()
                .map_err(|e| anyhow::anyhow!("Failed to finish Arrow StreamWriter: {}", e))?;
        }

        self.current_writer = None;
        self.current_schema = None;
        self.current_file = None;
        self.sequence += 1;

        // We need the schema to open a new file, but we don't have it here
        // This will be handled by the caller providing the schema
        Ok(())
    }
}

/// Convert spans to Arrow RecordBatch
fn convert_spans_to_arrow_batch(spans: &[SpanData]) -> Result<arrow::record_batch::RecordBatch> {
    use arrow::record_batch::RecordBatch;
    use std::sync::Arc;

    if spans.is_empty() {
        return Err(anyhow::anyhow!("Cannot create batch from empty spans"));
    }

    // Create Arrow schema for spans
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
        names.push(Some(span_data.name.clone()));
        kinds.push(span_data.span_kind.clone() as i32);
        start_times.push(
            span_data
                .start_time
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos() as u64,
        );
        end_times.push(
            span_data
                .end_time
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos() as u64,
        );
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
                opentelemetry::Value::F64(f) => serde_json::Value::Number(
                    serde_json::Number::from_f64(*f).unwrap_or(serde_json::Number::from(0)),
                ),
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
    let parent_span_id_refs: Vec<Option<&[u8]>> =
        parent_span_ids.iter().map(|opt| opt.as_deref()).collect();
    let name_refs: Vec<Option<&str>> = names
        .iter()
        .map(|opt| opt.as_ref().map(|s| s.as_ref()))
        .collect();

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

/// File-based SpanExporter implementation
#[derive(Debug)]
pub struct FileSpanExporter {
    file_exporter: Arc<OtlpFileExporter>,
}

impl FileSpanExporter {
    /// Create a new FileSpanExporter with the given file exporter
    pub fn new(file_exporter: Arc<OtlpFileExporter>) -> Self {
        Self { file_exporter }
    }
}

impl SpanExporter for FileSpanExporter {
    #[allow(refining_impl_trait_reachable)]
    fn export(
        &self,
        batch: Vec<SpanData>,
    ) -> BoxFuture<'static, opentelemetry_sdk::error::OTelSdkResult> {
        let file_exporter = self.file_exporter.clone();

        async move {
            // Export traces asynchronously
            file_exporter.export_traces(batch).await.map_err(|e| {
                warn!("Failed to export traces to file: {}", e);
                opentelemetry_sdk::error::OTelSdkError::InternalFailure(e.to_string())
            })
        }
        .boxed()
    }

    fn shutdown(&mut self) -> opentelemetry_sdk::error::OTelSdkResult {
        // Flush any pending writes
        // Try to get current runtime handle
        match tokio::runtime::Handle::try_current() {
            Ok(handle) => handle.block_on(async {
                self.file_exporter.flush().await.map_err(|e| {
                    opentelemetry_sdk::error::OTelSdkError::InternalFailure(e.to_string())
                })
            }),
            _ => {
                // No runtime available, can't flush
                Ok(())
            }
        }
    }
}

/// OtlpLibrary-based SpanExporter implementation
///
/// This exporter wraps an `OtlpLibrary` instance and implements `SpanExporter`
/// for seamless integration with OpenTelemetry SDK's `TracerProvider`.
#[derive(Clone, Debug)]
pub struct OtlpSpanExporter {
    library: Arc<crate::api::public::OtlpLibrary>,
}

impl OtlpSpanExporter {
    /// Create a new OtlpSpanExporter with the given library instance
    pub(crate) fn new(library: Arc<crate::api::public::OtlpLibrary>) -> Self {
        Self { library }
    }
}

impl SpanExporter for OtlpSpanExporter {
    #[allow(refining_impl_trait_reachable)]
    fn export(
        &self,
        batch: Vec<SpanData>,
    ) -> BoxFuture<'static, opentelemetry_sdk::error::OTelSdkResult> {
        let library = self.library.clone();

        async move {
            library.export_traces(batch).await.map_err(|e| {
                warn!("Failed to export traces via OtlpLibrary: {}", e);
                opentelemetry_sdk::error::OTelSdkError::InternalFailure(format!(
                    "OtlpLibrary export failed: {}",
                    e
                ))
            })
        }
        .boxed()
    }

    fn shutdown(&mut self) -> opentelemetry_sdk::error::OTelSdkResult {
        // Shutdown is handled by OtlpLibrary::shutdown() separately
        Ok(())
    }
}
