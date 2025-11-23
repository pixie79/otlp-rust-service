//! Public API for embedded library usage
//!
//! Provides programmatic API methods for sending OTLP messages without using gRPC.

use crate::config::Config;
use crate::error::OtlpError;
use crate::otlp::{BatchBuffer, OtlpFileExporter};
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time::{interval, Duration};
use tracing::{info, warn};

/// Main library instance for embedded usage
///
/// The `OtlpLibrary` provides a programmatic API for sending OpenTelemetry Protocol (OTLP)
/// traces and metrics without using gRPC. It handles buffering, batch writing, file rotation,
/// and optional remote forwarding.
///
/// # Features
///
/// - **Buffered Export**: Messages are buffered and written in batches at configurable intervals
/// - **File Storage**: Writes OTLP data to local files in Arrow IPC Streaming format
/// - **Automatic Cleanup**: Removes old files based on configurable retention intervals
/// - **Optional Forwarding**: Can forward messages to remote OTLP endpoints with format conversion
/// - **Dual Protocol Support**: Supports both gRPC Protobuf and gRPC Arrow Flight protocols
///
/// # Example
///
/// ```no_run
/// use otlp_arrow_library::{OtlpLibrary, Config};
///
/// # async fn example() -> Result<(), otlp_arrow_library::OtlpError> {
/// let config = Config::default();
/// let library = OtlpLibrary::new(config).await?;
///
/// // Export a trace span
/// // library.export_trace(span).await?;
///
/// // Flush all pending writes
/// library.flush().await?;
///
/// // Shutdown gracefully
/// library.shutdown().await?;
/// # Ok(())
/// # }
/// ```
#[derive(Debug)]
pub struct OtlpLibrary {
    config: Config,
    file_exporter: Arc<OtlpFileExporter>,
    batch_buffer: Arc<BatchBuffer>,
    write_handle: Arc<Mutex<Option<tokio::task::JoinHandle<()>>>>,
    cleanup_handles: Arc<Mutex<Vec<tokio::task::JoinHandle<()>>>>,
}

impl OtlpLibrary {
    /// Create a new OTLP library instance with the provided configuration
    ///
    /// This method initializes the library with the given configuration, creates output
    /// directories, starts background tasks for batch writing and file cleanup, and optionally
    /// sets up remote forwarding if configured.
    ///
    /// # Arguments
    ///
    /// * `config` - Configuration specifying output directory, write intervals, cleanup intervals,
    ///   protocol settings, and optional forwarding configuration
    ///
    /// # Returns
    ///
    /// Returns `Ok(OtlpLibrary)` if initialization succeeds, or `Err(OtlpError)` if:
    /// - Configuration validation fails
    /// - Output directories cannot be created
    /// - File exporter initialization fails
    ///
    /// # Example
    ///
    /// ```no_run
    /// use otlp_arrow_library::{OtlpLibrary, Config};
    ///
    /// # async fn example() -> Result<(), otlp_arrow_library::OtlpError> {
    /// let config = Config::default();
    /// let library = OtlpLibrary::new(config).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn new(config: Config) -> Result<Self, OtlpError> {
        // Validate configuration
        config.validate().map_err(OtlpError::from)?;

        // Create output directories
        std::fs::create_dir_all(&config.output_dir.join("otlp/traces")).map_err(|e| {
            OtlpError::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Failed to create traces directory: {}", e),
            ))
        })?;

        std::fs::create_dir_all(&config.output_dir.join("otlp/metrics")).map_err(|e| {
            OtlpError::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Failed to create metrics directory: {}", e),
            ))
        })?;

        // Create file exporter
        let file_exporter = Arc::new(OtlpFileExporter::new(&config).map_err(|e| {
            OtlpError::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                e.to_string(),
            ))
        })?);

        // Create batch buffer
        let batch_buffer = Arc::new(BatchBuffer::new(config.write_interval_secs));

        // Start background write task
        let write_handle = Arc::new(Mutex::new(None));
        let file_exporter_clone = file_exporter.clone();
        let batch_buffer_clone = batch_buffer.clone();
        let write_interval = Duration::from_secs(config.write_interval_secs);
        let handle = tokio::spawn(async move {
            let mut interval_timer = interval(write_interval);
            loop {
                interval_timer.tick().await;

                // Check if we should write
                if batch_buffer_clone.should_write().await {
                    // Take buffered traces
                    let traces = batch_buffer_clone.take_traces().await;
                    if !traces.is_empty() {
                        if let Err(e) = file_exporter_clone.export_traces(traces).await {
                            warn!("Failed to export traces: {}", e);
                        }
                    }

                    // Take buffered metrics
                    let metrics = batch_buffer_clone.take_metrics().await;
                    for metric in metrics {
                        if let Err(e) = file_exporter_clone.export_metrics(&metric).await {
                            warn!("Failed to export metrics: {}", e);
                        }
                    }

                    batch_buffer_clone.update_last_write().await;
                }
            }
        });

        {
            let mut handle_guard = write_handle.lock().await;
            *handle_guard = Some(handle);
        }

        // Start background cleanup tasks
        let file_exporter_traces_cleanup = file_exporter.clone();
        let trace_cleanup_interval = Duration::from_secs(config.trace_cleanup_interval_secs);
        let trace_cleanup_handle = tokio::spawn(async move {
            let mut interval_timer = interval(trace_cleanup_interval);
            loop {
                interval_timer.tick().await;
                if let Err(e) = file_exporter_traces_cleanup
                    .cleanup_traces(config.trace_cleanup_interval_secs)
                    .await
                {
                    warn!("Failed to cleanup trace files: {}", e);
                }
            }
        });

        let file_exporter_metrics_cleanup = file_exporter.clone();
        let metric_cleanup_interval = Duration::from_secs(config.metric_cleanup_interval_secs);
        let metric_cleanup_handle = tokio::spawn(async move {
            let mut interval_timer = interval(metric_cleanup_interval);
            loop {
                interval_timer.tick().await;
                if let Err(e) = file_exporter_metrics_cleanup
                    .cleanup_metrics(config.metric_cleanup_interval_secs)
                    .await
                {
                    warn!("Failed to cleanup metric files: {}", e);
                }
            }
        });

        // Store cleanup handles (we'll need to abort them on shutdown)
        let cleanup_handles = Arc::new(Mutex::new(vec![
            trace_cleanup_handle,
            metric_cleanup_handle,
        ]));

        info!(
            "OTLP library initialized with output directory: {}",
            config.output_dir.display()
        );

        Ok(Self {
            config,
            file_exporter,
            batch_buffer,
            write_handle,
            cleanup_handles,
        })
    }

    /// Create a configuration builder for programmatic configuration
    ///
    /// Returns a `ConfigBuilder` that allows fluent construction of configuration with
    /// method chaining.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use otlp_arrow_library::OtlpLibrary;
    ///
    /// # async fn example() -> Result<(), otlp_arrow_library::OtlpError> {
    /// let config = OtlpLibrary::with_config_builder()
    ///     .output_dir("./custom_output")
    ///     .write_interval_secs(10)
    ///     .build()?;
    /// let library = OtlpLibrary::new(config).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn with_config_builder() -> crate::config::ConfigBuilder {
        crate::config::ConfigBuilder::new()
    }

    /// Export a single trace span
    ///
    /// Adds a trace span to the internal buffer. The span will be written to disk
    /// when the write interval elapses or when `flush()` is called.
    ///
    /// # Arguments
    ///
    /// * `span` - The OpenTelemetry span data to export
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if the span was successfully buffered, or `Err(OtlpError)` if
    /// the buffer is full or an error occurs.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use otlp_arrow_library::OtlpLibrary;
    /// use opentelemetry_sdk::trace::SpanData;
    ///
    /// # async fn example(library: OtlpLibrary, span: SpanData) -> Result<(), otlp_arrow_library::OtlpError> {
    /// library.export_trace(span).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn export_trace(
        &self,
        span: opentelemetry_sdk::trace::SpanData,
    ) -> Result<(), OtlpError> {
        self.batch_buffer.add_trace(span).await
    }

    /// Export multiple trace spans
    ///
    /// Adds multiple trace spans to the internal buffer in a single operation. This is
    /// more efficient than calling `export_trace()` multiple times.
    ///
    /// # Arguments
    ///
    /// * `spans` - A vector of OpenTelemetry span data to export
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if all spans were successfully buffered, or `Err(OtlpError)` if
    /// the buffer is full or an error occurs.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use otlp_arrow_library::OtlpLibrary;
    /// use opentelemetry_sdk::trace::SpanData;
    ///
    /// # async fn example(library: OtlpLibrary, spans: Vec<SpanData>) -> Result<(), otlp_arrow_library::OtlpError> {
    /// library.export_traces(spans).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn export_traces(
        &self,
        spans: Vec<opentelemetry_sdk::trace::SpanData>,
    ) -> Result<(), OtlpError> {
        self.batch_buffer.add_traces(spans).await
    }

    /// Export metrics
    ///
    /// Adds metrics to the internal buffer. The metrics will be written to disk when
    /// the write interval elapses or when `flush()` is called.
    ///
    /// # Arguments
    ///
    /// * `metrics` - The OpenTelemetry resource metrics to export
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if the metrics were successfully buffered, or `Err(OtlpError)` if
    /// the buffer is full or an error occurs.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use otlp_arrow_library::OtlpLibrary;
    /// use opentelemetry_sdk::metrics::data::ResourceMetrics;
    ///
    /// # async fn example(library: OtlpLibrary, metrics: ResourceMetrics) -> Result<(), otlp_arrow_library::OtlpError> {
    /// library.export_metrics(metrics).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn export_metrics(
        &self,
        metrics: opentelemetry_sdk::metrics::data::ResourceMetrics,
    ) -> Result<(), OtlpError> {
        self.batch_buffer.add_metrics(metrics).await
    }

    /// Force immediate flush of all buffered messages to disk
    ///
    /// This method immediately writes all buffered traces and metrics to disk, bypassing
    /// the normal write interval. Useful for ensuring data is persisted before shutdown
    /// or at critical points in your application.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if all buffered data was successfully written, or `Err(OtlpError)`
    /// if a write error occurs.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use otlp_arrow_library::OtlpLibrary;
    ///
    /// # async fn example(library: OtlpLibrary) -> Result<(), otlp_arrow_library::OtlpError> {
    /// // Export some data
    /// // library.export_trace(span).await?;
    ///
    /// // Force immediate write
    /// library.flush().await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn flush(&self) -> Result<(), OtlpError> {
        // Take all buffered data and write immediately
        let traces = self.batch_buffer.take_traces().await;
        if !traces.is_empty() {
            self.file_exporter.export_traces(traces).await?;
        }

        let metrics = self.batch_buffer.take_metrics().await;
        for metric in metrics {
            self.file_exporter.export_metrics(&metric).await?;
        }

        // Flush file writers
        self.file_exporter.flush().await?;
        self.batch_buffer.update_last_write().await;

        Ok(())
    }

    /// Get a reference to the file exporter (for server initialization)
    ///
    /// This method is primarily used internally by gRPC servers to access the file exporter.
    /// It returns a clone of the internal `Arc<OtlpFileExporter>`.
    ///
    /// # Returns
    ///
    /// Returns an `Arc<OtlpFileExporter>` that can be shared with gRPC server implementations.
    pub fn file_exporter(&self) -> Arc<OtlpFileExporter> {
        self.file_exporter.clone()
    }

    /// Gracefully shut down the library, flushing all pending writes
    ///
    /// This method performs a graceful shutdown by:
    /// 1. Flushing all buffered traces and metrics to disk
    /// 2. Stopping all background tasks (batch writing, cleanup)
    ///
    /// After calling this method, the library instance should not be used further.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if shutdown completes successfully, or `Err(OtlpError)` if an
    /// error occurs during shutdown.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use otlp_arrow_library::OtlpLibrary;
    ///
    /// # async fn example(library: OtlpLibrary) -> Result<(), otlp_arrow_library::OtlpError> {
    /// // Use library...
    ///
    /// // Shutdown gracefully
    /// library.shutdown().await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn shutdown(&self) -> Result<(), OtlpError> {
        // Flush all pending writes
        self.flush().await?;

        // Stop background write task
        let mut handle_guard = self.write_handle.lock().await;
        if let Some(handle) = handle_guard.take() {
            handle.abort();
        }

        // Stop cleanup tasks
        let mut cleanup_guard = self.cleanup_handles.lock().await;
        for handle in cleanup_guard.drain(..) {
            handle.abort();
        }

        info!("OTLP library shutdown complete");
        Ok(())
    }
}
