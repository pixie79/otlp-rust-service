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
use futures::future::BoxFuture;
use futures::FutureExt;
use opentelemetry_sdk::metrics::data::ResourceMetrics;
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
    // Metrics collection
    messages_received: Arc<Mutex<u64>>,
    files_written: Arc<Mutex<u64>>,
    errors_count: Arc<Mutex<u64>>,
    format_conversions: Arc<Mutex<u64>>,
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

#[derive(Debug)]
struct TracesWriter {
    current_file: Option<File>,
    current_size: u64,
    output_dir: PathBuf,
    sequence: u64,
}

#[derive(Debug)]
struct MetricsWriter {
    current_file: Option<File>,
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
                current_size: 0,
                output_dir: traces_dir,
                sequence: 0,
            })),
            metrics_writer: Arc::new(Mutex::new(MetricsWriter {
                current_file: None,
                current_size: 0,
                output_dir: metrics_dir,
                sequence: 0,
            })),
            output_dir,
            max_file_size,
            format: ExportFormat::Arrow,
            forwarder,
            messages_received: Arc::new(Mutex::new(0)),
            files_written: Arc::new(Mutex::new(0)),
            errors_count: Arc::new(Mutex::new(0)),
            format_conversions: Arc::new(Mutex::new(0)),
        };

        Ok(exporter)
    }

    /// Export traces to file
    pub async fn export_traces(&self, spans: Vec<SpanData>) -> Result<(), OtlpError> {
        if spans.is_empty() {
            return Ok(());
        }

        // Update metrics
        {
            let mut count = self.messages_received.lock().await;
            *count += spans.len() as u64;
        }

        // Convert spans to Arrow IPC format
        let data = convert_spans_to_arrow_ipc(&spans)
            .map_err(|e| OtlpError::Export(OtlpExportError::ArrowConversionError(e.to_string())))?;

        let data_size = data.len() as u64;
        let mut writer = self.traces_writer.lock().await;
        let output_dir = writer.output_dir.clone();

        // Check if we need to rotate
        if writer.current_size + data_size > self.max_file_size {
            writer
                .rotate_file(&output_dir, "traces", self.format)
                .map_err(|e| OtlpError::Io(std::io::Error::other(e.to_string())))?;
        }

        // Open file if needed
        if writer.current_file.is_none() {
            writer
                .open_new_file(&output_dir, "traces", self.format)
                .map_err(|e| OtlpError::Io(std::io::Error::other(e.to_string())))?;
        }

        // Write data
        if let Some(ref mut file) = writer.current_file {
            file.write_all(&data).map_err(OtlpError::Io)?;
            file.flush().map_err(OtlpError::Io)?;
            writer.current_size += data_size;
            trace!(
                "Wrote {} spans ({} bytes) to trace file",
                spans.len(),
                data_size
            );

            // Update metrics
            {
                let mut count = self.files_written.lock().await;
                *count += 1;
            }
        }

        // Forward traces if forwarding is enabled
        if let Some(ref forwarder) = self.forwarder {
            if let Err(e) = forwarder.forward_traces(spans).await {
                warn!(error = %e, "Failed to forward traces, but local storage succeeded");
                // Update error metrics
                {
                    let mut count = self.errors_count.lock().await;
                    *count += 1;
                }
                // Don't fail - forwarding is best-effort
            } else {
                // Update format conversion metrics if conversion occurred
                {
                    let mut count = self.format_conversions.lock().await;
                    *count += 1;
                }
            }
        }

        Ok(())
    }

    /// Export metrics to file
    pub async fn export_metrics(&self, metrics: &ResourceMetrics) -> Result<(), OtlpError> {
        // Update metrics
        {
            let mut count = self.messages_received.lock().await;
            *count += 1;
        }

        // Convert metrics to Arrow IPC format
        let data = convert_metrics_to_arrow_ipc(metrics)
            .map_err(|e| OtlpError::Export(OtlpExportError::ArrowConversionError(e.to_string())))?;

        let data_size = data.len() as u64;
        let mut writer = self.metrics_writer.lock().await;
        let output_dir = writer.output_dir.clone();

        // Check if we need to rotate
        if writer.current_size + data_size > self.max_file_size {
            writer
                .rotate_file(&output_dir, "metrics", self.format)
                .map_err(|e| OtlpError::Io(std::io::Error::other(e.to_string())))?;
        }

        // Open file if needed
        if writer.current_file.is_none() {
            writer
                .open_new_file(&output_dir, "metrics", self.format)
                .map_err(|e| OtlpError::Io(std::io::Error::other(e.to_string())))?;
        }

        // Write data
        if let Some(ref mut file) = writer.current_file {
            file.write_all(&data).map_err(OtlpError::Io)?;
            file.flush().map_err(OtlpError::Io)?;
            writer.current_size += data_size;
            trace!("Wrote metrics ({} bytes) to file", data_size);

            // Update metrics
            {
                let mut count = self.files_written.lock().await;
                *count += 1;
            }
        }

        // Forward metrics if forwarding is enabled
        if let Some(ref forwarder) = self.forwarder {
            if let Err(e) = forwarder.forward_metrics(metrics).await {
                warn!(error = %e, "Failed to forward metrics, but local storage succeeded");
                // Update error metrics
                {
                    let mut count = self.errors_count.lock().await;
                    *count += 1;
                }
                // Don't fail - forwarding is best-effort
            } else {
                // Update format conversion metrics if conversion occurred
                {
                    let mut count = self.format_conversions.lock().await;
                    *count += 1;
                }
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
        let messages = *self.messages_received.lock().await;
        let files = *self.files_written.lock().await;
        let errors = *self.errors_count.lock().await;
        let conversions = *self.format_conversions.lock().await;
        (messages, files, errors, conversions)
    }

    /// Flush all pending writes
    pub async fn flush(&self) -> Result<(), OtlpError> {
        let mut traces_writer = self.traces_writer.lock().await;
        if let Some(ref mut file) = traces_writer.current_file {
            file.flush().map_err(OtlpError::Io)?;
        }

        let mut metrics_writer = self.metrics_writer.lock().await;
        if let Some(ref mut file) = metrics_writer.current_file {
            file.flush().map_err(OtlpError::Io)?;
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
}

impl TracesWriter {
    fn open_new_file(
        &mut self,
        output_dir: &Path,
        prefix: &str,
        _format: ExportFormat,
    ) -> Result<()> {
        let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
        let extension = "arrow";
        let filename = format!(
            "otlp_{}_{}_{:04}.{}",
            prefix, timestamp, self.sequence, extension
        );
        let file_path = output_dir.join(&filename);

        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&file_path)
            .map_err(|e| {
                anyhow::anyhow!("Failed to create OTLP file {}: {}", file_path.display(), e)
            })?;

        self.current_file = Some(file);
        self.current_size = 0;

        info!(
            file = %file_path.display(),
            "Opened new OTLP {} file",
            prefix
        );

        Ok(())
    }

    fn rotate_file(&mut self, output_dir: &Path, prefix: &str, format: ExportFormat) -> Result<()> {
        if let Some(ref mut file) = self.current_file {
            file.flush()?;
        }
        self.current_file = None;
        self.sequence += 1;
        self.open_new_file(output_dir, prefix, format)
    }
}

impl MetricsWriter {
    fn open_new_file(
        &mut self,
        output_dir: &Path,
        prefix: &str,
        _format: ExportFormat,
    ) -> Result<()> {
        let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
        let extension = "arrow";
        let filename = format!(
            "otlp_{}_{}_{:04}.{}",
            prefix, timestamp, self.sequence, extension
        );
        let file_path = output_dir.join(&filename);

        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&file_path)
            .map_err(|e| {
                anyhow::anyhow!("Failed to create OTLP file {}: {}", file_path.display(), e)
            })?;

        self.current_file = Some(file);
        self.current_size = 0;

        info!(
            file = %file_path.display(),
            "Opened new OTLP {} file",
            prefix
        );

        Ok(())
    }

    fn rotate_file(&mut self, output_dir: &Path, prefix: &str, format: ExportFormat) -> Result<()> {
        if let Some(ref mut file) = self.current_file {
            file.flush()?;
        }
        self.current_file = None;
        self.sequence += 1;
        self.open_new_file(output_dir, prefix, format)
    }
}

/// Convert spans to Arrow IPC format
fn convert_spans_to_arrow_ipc(spans: &[SpanData]) -> Result<Vec<u8>> {
    use arrow::record_batch::RecordBatch;
    use std::io::Cursor;
    use std::sync::Arc;

    if spans.is_empty() {
        return Ok(Vec::new());
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

    // Serialize to Arrow IPC stream format
    let mut buffer = Vec::new();
    let cursor = Cursor::new(&mut buffer);
    let mut writer = StreamWriter::try_new(cursor, batch.schema().as_ref())?;
    writer.write(&batch)?;
    writer.finish()?;

    Ok(buffer)
}

/// Convert metrics Data to Arrow IPC format
fn convert_metrics_to_arrow_ipc(metrics: &ResourceMetrics) -> Result<Vec<u8>> {
    use arrow::record_batch::RecordBatch;
    use std::io::Cursor;
    use std::sync::Arc;

    // Since ResourceMetrics fields are private, we'll use Debug format to extract information
    // This is a simplified approach - a full implementation would use opentelemetry-proto
    let metrics_debug = format!("{:?}", metrics);

    // Create a simple record with the debug string
    // Note: Arrow arrays require Vec, not arrays, so we use vec! here
    #[allow(clippy::useless_vec)]
    let metric_names = vec![Some("resource_metrics".to_string())];
    let values = vec![0.0]; // Placeholder
    let timestamps = vec![Some(
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64,
    )];
    #[allow(clippy::useless_vec)]
    let metric_types = vec![Some("debug".to_string())];
    #[allow(clippy::useless_vec)]
    let attributes = vec![Some(metrics_debug)];

    // If no metrics, return empty batch
    if metric_names.is_empty() {
        let schema = Schema::new(vec![
            Field::new("metric_name", DataType::Utf8, false),
            Field::new("value", DataType::Float64, false),
            Field::new("timestamp_unix_nano", DataType::UInt64, false),
            Field::new("metric_type", DataType::Utf8, false),
            Field::new("attributes", DataType::Utf8, true),
        ]);

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

        let mut buffer = Vec::new();
        let cursor = Cursor::new(&mut buffer);
        let mut writer = StreamWriter::try_new(cursor, batch.schema().as_ref())?;
        writer.write(&batch)?;
        writer.finish()?;

        return Ok(buffer);
    }

    // Build Arrow arrays
    let name_refs: Vec<Option<&str>> = metric_names.iter().map(|opt| opt.as_deref()).collect();
    let type_refs: Vec<Option<&str>> = metric_types.iter().map(|opt| opt.as_deref()).collect();
    let attr_refs: Vec<Option<&str>> = attributes.iter().map(|opt| opt.as_deref()).collect();

    let schema = Schema::new(vec![
        Field::new("metric_name", DataType::Utf8, false),
        Field::new("value", DataType::Float64, false),
        Field::new("timestamp_unix_nano", DataType::UInt64, false),
        Field::new("metric_type", DataType::Utf8, false),
        Field::new("attributes", DataType::Utf8, true),
    ]);

    let batch = RecordBatch::try_new(
        Arc::new(schema),
        vec![
            Arc::new(StringArray::from(name_refs)),
            Arc::new(Float64Array::from(values)),
            Arc::new(UInt64Array::from(timestamps)),
            Arc::new(StringArray::from(type_refs)),
            Arc::new(StringArray::from(attr_refs)),
        ],
    )?;

    // Serialize to Arrow IPC stream format
    let mut buffer = Vec::new();
    let cursor = Cursor::new(&mut buffer);
    let mut writer = StreamWriter::try_new(cursor, batch.schema().as_ref())?;
    writer.write(&batch)?;
    writer.finish()?;

    Ok(buffer)
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
            match file_exporter.export_traces(batch).await {
                Ok(()) => Ok(()),
                Err(e) => {
                    warn!("Failed to export traces to file: {}", e);
                    Err(opentelemetry_sdk::error::OTelSdkError::InternalFailure(
                        e.to_string(),
                    ))
                }
            }
        }
        .boxed()
    }

    fn shutdown(&mut self) -> opentelemetry_sdk::error::OTelSdkResult {
        // Flush any pending writes
        // Try to get current runtime handle
        if let Ok(handle) = tokio::runtime::Handle::try_current() {
            handle.block_on(async {
                self.file_exporter.flush().await.map_err(|e| {
                    opentelemetry_sdk::error::OTelSdkError::InternalFailure(e.to_string())
                })
            })
        } else {
            // No runtime available, can't flush
            Ok(())
        }
    }
}

/// File-based MetricsExporter implementation
#[derive(Debug)]
pub struct FileMetricExporter {
    file_exporter: Arc<OtlpFileExporter>,
}

impl FileMetricExporter {
    /// Create a new FileMetricExporter with the given file exporter
    pub fn new(file_exporter: Arc<OtlpFileExporter>) -> Self {
        Self { file_exporter }
    }
}

impl opentelemetry_sdk::metrics::exporter::PushMetricExporter for FileMetricExporter {
    fn export(
        &self,
        metrics: &ResourceMetrics,
    ) -> impl std::future::Future<Output = opentelemetry_sdk::error::OTelSdkResult> + Send {
        let file_exporter = self.file_exporter.clone();

        async move {
            // Convert and write metrics
            match file_exporter.export_metrics(metrics).await {
                Ok(()) => Ok(()),
                Err(e) => {
                    warn!("Failed to export metrics to file: {}", e);
                    Err(opentelemetry_sdk::error::OTelSdkError::InternalFailure(
                        e.to_string(),
                    ))
                }
            }
        }
    }

    fn force_flush(&self) -> opentelemetry_sdk::error::OTelSdkResult {
        if let Ok(handle) = tokio::runtime::Handle::try_current() {
            handle.block_on(async {
                self.file_exporter.flush().await.map_err(|e| {
                    opentelemetry_sdk::error::OTelSdkError::InternalFailure(e.to_string())
                })
            })
        } else {
            Ok(())
        }
    }

    fn shutdown_with_timeout(
        &self,
        _timeout: std::time::Duration,
    ) -> opentelemetry_sdk::error::OTelSdkResult {
        self.force_flush()
    }

    fn temporality(&self) -> opentelemetry_sdk::metrics::Temporality {
        opentelemetry_sdk::metrics::Temporality::Cumulative
    }
}

/// OtlpLibrary-based MetricExporter implementation
///
/// This exporter wraps an `OtlpLibrary` instance and implements `PushMetricExporter`
/// for seamless integration with OpenTelemetry SDK's `PeriodicReader`.
#[derive(Clone, Debug)]
pub struct OtlpMetricExporter {
    library: Arc<crate::api::public::OtlpLibrary>,
}

impl OtlpMetricExporter {
    /// Create a new OtlpMetricExporter with the given library instance
    pub(crate) fn new(library: Arc<crate::api::public::OtlpLibrary>) -> Self {
        Self { library }
    }
}

impl opentelemetry_sdk::metrics::exporter::PushMetricExporter for OtlpMetricExporter {
    fn export(
        &self,
        metrics: &ResourceMetrics,
    ) -> impl std::future::Future<Output = opentelemetry_sdk::error::OTelSdkResult> + Send {
        let library = self.library.clone();

        async move {
            match library.export_metrics_ref(metrics).await {
                Ok(()) => Ok(()),
                Err(e) => {
                    warn!("Failed to export metrics via OtlpLibrary: {}", e);
                    Err(opentelemetry_sdk::error::OTelSdkError::InternalFailure(
                        format!("OtlpLibrary export failed: {}", e),
                    ))
                }
            }
        }
    }

    fn force_flush(&self) -> opentelemetry_sdk::error::OTelSdkResult {
        // Try to get current runtime handle
        if let Ok(handle) = tokio::runtime::Handle::try_current() {
            // If we're in an async context, spawn the flush task
            // Note: This is best-effort - the flush will happen asynchronously
            let library = self.library.clone();
            handle.spawn(async move {
                let _ = library.flush().await;
            });
            Ok(())
        } else {
            // No runtime available, can't flush
            Ok(())
        }
    }

    fn shutdown_with_timeout(
        &self,
        _timeout: std::time::Duration,
    ) -> opentelemetry_sdk::error::OTelSdkResult {
        // Shutdown is handled by OtlpLibrary::shutdown() separately
        Ok(())
    }

    fn temporality(&self) -> opentelemetry_sdk::metrics::Temporality {
        opentelemetry_sdk::metrics::Temporality::Cumulative
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
            match library.export_traces(batch).await {
                Ok(()) => Ok(()),
                Err(e) => {
                    warn!("Failed to export traces via OtlpLibrary: {}", e);
                    Err(opentelemetry_sdk::error::OTelSdkError::InternalFailure(
                        format!("OtlpLibrary export failed: {}", e),
                    ))
                }
            }
        }
        .boxed()
    }

    fn shutdown(&mut self) -> opentelemetry_sdk::error::OTelSdkResult {
        // Shutdown is handled by OtlpLibrary::shutdown() separately
        Ok(())
    }
}
