//! Batch writer for OTLP messages
//!
//! Buffers OTLP messages (traces and metrics) in memory and writes them to disk
//! at configurable intervals using Arrow IPC Streaming format.

use crate::error::OtlpError;
use opentelemetry_sdk::metrics::data::ResourceMetrics;
use opentelemetry_sdk::trace::SpanData;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;

/// Batch buffer for OTLP messages
#[derive(Debug)]
pub struct BatchBuffer {
    /// Buffered trace spans
    traces: Arc<Mutex<Vec<SpanData>>>,
    /// Buffered metrics
    metrics: Arc<Mutex<Vec<ResourceMetrics>>>,
    /// Write interval in seconds
    write_interval: Duration,
    /// Last write timestamp
    last_write: Arc<Mutex<std::time::SystemTime>>,
}

impl BatchBuffer {
    /// Create a new batch buffer with the specified write interval
    pub fn new(write_interval_secs: u64) -> Self {
        Self {
            traces: Arc::new(Mutex::new(Vec::new())),
            metrics: Arc::new(Mutex::new(Vec::new())),
            write_interval: Duration::from_secs(write_interval_secs),
            last_write: Arc::new(Mutex::new(std::time::SystemTime::now())),
        }
    }

    /// Add a trace span to the buffer
    pub async fn add_trace(&self, span: SpanData) -> Result<(), OtlpError> {
        let mut traces = self.traces.lock().await;
        traces.push(span);
        Ok(())
    }

    /// Add multiple trace spans to the buffer
    pub async fn add_traces(&self, spans: Vec<SpanData>) -> Result<(), OtlpError> {
        let mut traces = self.traces.lock().await;
        traces.extend(spans);
        Ok(())
    }

    /// Add metrics to the buffer
    pub async fn add_metrics(&self, metrics: ResourceMetrics) -> Result<(), OtlpError> {
        let mut buffered_metrics = self.metrics.lock().await;
        buffered_metrics.push(metrics);
        Ok(())
    }

    /// Take all buffered traces (clears the buffer)
    pub async fn take_traces(&self) -> Vec<SpanData> {
        let mut traces = self.traces.lock().await;
        std::mem::take(&mut *traces)
    }

    /// Take all buffered metrics (clears the buffer)
    pub async fn take_metrics(&self) -> Vec<ResourceMetrics> {
        let mut metrics = self.metrics.lock().await;
        std::mem::take(&mut *metrics)
    }

    /// Check if it's time to write based on interval
    pub async fn should_write(&self) -> bool {
        let last_write = self.last_write.lock().await;
        if let Ok(elapsed) = last_write.elapsed() {
            elapsed >= self.write_interval
        } else {
            // Clock went backwards, reset
            true
        }
    }

    /// Update the last write timestamp
    pub async fn update_last_write(&self) {
        let mut last_write = self.last_write.lock().await;
        *last_write = std::time::SystemTime::now();
    }

    /// Get the number of buffered traces
    pub async fn trace_count(&self) -> usize {
        let traces = self.traces.lock().await;
        traces.len()
    }

    /// Get the number of buffered metrics
    pub async fn metric_count(&self) -> usize {
        let metrics = self.metrics.lock().await;
        metrics.len()
    }
}
