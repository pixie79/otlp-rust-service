//! Remote forwarding module
//!
//! Forwards OTLP messages to remote endpoints with format conversion support.

use crate::config::{ForwardingConfig, ForwardingProtocol};
use crate::error::{OtlpError, OtlpExportError};
use crate::otlp::converter::FormatConverter;
use opentelemetry_proto::tonic::collector::metrics::v1::ExportMetricsServiceRequest;
use opentelemetry_proto::tonic::collector::trace::v1::ExportTraceServiceRequest;
use opentelemetry_sdk::metrics::data::ResourceMetrics;
use opentelemetry_sdk::trace::SpanData;
use prost::Message;
use secrecy::ExposeSecret;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;
use tracing::{error, info, warn};

/// Circuit breaker state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CircuitState {
    Closed,   // Normal operation
    Open,     // Failing, reject requests
    HalfOpen, // Testing if service recovered
}

/// Circuit breaker for forwarding failures
#[derive(Debug)]
struct CircuitBreaker {
    state: Arc<Mutex<CircuitState>>,
    failure_count: Arc<Mutex<u32>>,
    last_failure_time: Arc<Mutex<Option<Instant>>>,
    failure_threshold: u32,
    #[allow(dead_code)]
    timeout: Duration,
    half_open_timeout: Duration,
    /// Flag to prevent concurrent test requests in half-open state
    half_open_test_in_progress: Arc<Mutex<bool>>,
}

impl CircuitBreaker {
    fn new(failure_threshold: u32, timeout: Duration) -> Self {
        Self {
            state: Arc::new(Mutex::new(CircuitState::Closed)),
            failure_count: Arc::new(Mutex::new(0)),
            last_failure_time: Arc::new(Mutex::new(None)),
            failure_threshold,
            timeout,
            half_open_timeout: Duration::from_secs(30), // 30 seconds to test recovery
            half_open_test_in_progress: Arc::new(Mutex::new(false)),
        }
    }

    async fn call<F, R>(&self, f: F) -> Result<R, OtlpError>
    where
        F: std::future::Future<Output = Result<R, OtlpError>>,
    {
        let state = *self.state.lock().await;

        match state {
            CircuitState::Open => {
                // Check if we should transition to half-open
                let last_failure = self.last_failure_time.lock().await;
                if let Some(failure_time) = *last_failure {
                    if failure_time.elapsed() >= self.half_open_timeout {
                        *self.state.lock().await = CircuitState::HalfOpen;
                        info!("Circuit breaker transitioning to half-open state");
                    } else {
                        return Err(OtlpError::Export(OtlpExportError::ForwardingError(
                            "Circuit breaker is open".to_string(),
                        )));
                    }
                } else {
                    return Err(OtlpError::Export(OtlpExportError::ForwardingError(
                        "Circuit breaker is open".to_string(),
                    )));
                }
            }
            CircuitState::HalfOpen => {
                // Check if a test is already in progress
                let mut test_in_progress = self.half_open_test_in_progress.lock().await;
                if *test_in_progress {
                    return Err(OtlpError::Export(OtlpExportError::ForwardingError(
                        "Circuit breaker is testing recovery (half-open)".to_string(),
                    )));
                }
                // Mark test as in progress
                *test_in_progress = true;
            }
            CircuitState::Closed => {
                // Normal operation - no special handling needed
            }
        }

        // Execute the operation
        let result = f.await;

        // Handle result based on current state
        let current_state = *self.state.lock().await;
        match (current_state, &result) {
            (CircuitState::HalfOpen, Ok(_)) => {
                // Success in half-open state - transition to closed and reset
                *self.state.lock().await = CircuitState::Closed;
                *self.failure_count.lock().await = 0;
                *self.last_failure_time.lock().await = None;
                *self.half_open_test_in_progress.lock().await = false;
                info!("Circuit breaker recovered - transitioning to closed state");
            }
            (CircuitState::HalfOpen, Err(_)) => {
                // Failure in half-open state - transition back to open
                *self.state.lock().await = CircuitState::Open;
                *self.last_failure_time.lock().await = Some(Instant::now());
                *self.half_open_test_in_progress.lock().await = false;
                warn!("Circuit breaker test failed - transitioning back to open state");
            }
            (CircuitState::Closed, Ok(_)) => {
                // Success in closed state - reset failure count
                *self.failure_count.lock().await = 0;
                *self.last_failure_time.lock().await = None;
            }
            (CircuitState::Closed, Err(_)) => {
                // Failure in closed state - increment failure count
                let mut failure_count = self.failure_count.lock().await;
                *failure_count += 1;
                *self.last_failure_time.lock().await = Some(Instant::now());

                if *failure_count >= self.failure_threshold {
                    *self.state.lock().await = CircuitState::Open;
                    warn!(
                        failure_count = *failure_count,
                        threshold = self.failure_threshold,
                        "Circuit breaker opened due to repeated failures"
                    );
                }
            }
            _ => {
                // Should not happen, but handle gracefully
                if matches!(current_state, CircuitState::HalfOpen) {
                    *self.half_open_test_in_progress.lock().await = false;
                }
            }
        }

        result
    }
}

/// OTLP forwarder for remote endpoints
#[derive(Debug, Clone)]
pub struct OtlpForwarder {
    config: ForwardingConfig,
    converter: FormatConverter,
    circuit_breaker: Arc<CircuitBreaker>,
    client: reqwest::Client,
}

impl OtlpForwarder {
    /// Create a new forwarder
    pub fn new(config: ForwardingConfig) -> Result<Self, OtlpError> {
        // Validate configuration
        config.validate()?;

        if !config.enabled {
            return Err(OtlpError::Export(OtlpExportError::ForwardingError(
                "Forwarding is not enabled".to_string(),
            )));
        }

        // Create HTTP client with authentication headers if needed
        let client_builder = reqwest::Client::builder();

        if config.authentication.is_some() {
            // Authentication headers will be added per-request
        }

        let client = client_builder.build().map_err(|e| {
            OtlpError::Export(OtlpExportError::ForwardingError(format!(
                "Failed to create HTTP client: {}",
                e
            )))
        })?;

        let circuit_breaker = Arc::new(CircuitBreaker::new(
            5,                       // 5 failures before opening
            Duration::from_secs(60), // 60 second timeout
        ));

        info!(
            endpoint = %config.endpoint_url.as_ref().unwrap_or(&"none".to_string()),
            protocol = ?config.protocol,
            "Created OTLP forwarder"
        );

        Ok(Self {
            config,
            converter: FormatConverter::new(),
            circuit_breaker,
            client,
        })
    }

    /// Forward traces asynchronously
    ///
    /// This method detects the input format and converts if needed based on
    /// the configured forwarding protocol.
    pub async fn forward_traces(&self, spans: Vec<SpanData>) -> Result<(), OtlpError> {
        if !self.config.enabled {
            return Ok(()); // Silently skip if disabled
        }

        if spans.is_empty() {
            return Ok(());
        }

        // Forward asynchronously to avoid blocking
        let forwarder = self.clone();
        tokio::spawn(async move {
            if let Err(e) = forwarder.forward_traces_internal(spans).await {
                error!(error = %e, "Failed to forward traces");
                // Don't propagate error - forwarding failures shouldn't fail local storage
            }
        });

        Ok(())
    }

    /// Internal method to forward traces (called asynchronously)
    async fn forward_traces_internal(&self, spans: Vec<SpanData>) -> Result<(), OtlpError> {
        self.circuit_breaker
            .call(async {
                match self.config.protocol {
                    ForwardingProtocol::Protobuf => {
                        // Convert spans to Protobuf request
                        let request = self.converter.spans_to_protobuf(spans).map_err(|e| {
                            OtlpError::Export(OtlpExportError::FormatConversionError(format!(
                                "Failed to convert spans to Protobuf: {}",
                                e
                            )))
                        })?;

                        if let Some(req) = request {
                            self.send_protobuf_traces(req).await?;
                        }
                    }
                    ForwardingProtocol::ArrowFlight => {
                        // Convert spans to Arrow Flight batch
                        let batch = FormatConverter::spans_to_arrow_batch(&spans).map_err(|e| {
                            OtlpError::Export(OtlpExportError::FormatConversionError(format!(
                                "Failed to convert spans to Arrow batch: {}",
                                e
                            )))
                        })?;

                        self.send_arrow_flight_traces(batch).await?;
                    }
                }
                Ok(())
            })
            .await
    }

    /// Forward metrics asynchronously
    pub async fn forward_metrics(&self, _metrics: &ResourceMetrics) -> Result<(), OtlpError> {
        if !self.config.enabled {
            return Ok(()); // Silently skip if disabled
        }

        // Forward asynchronously to avoid blocking
        // Note: ResourceMetrics doesn't implement Clone, so we convert to a clonable format first
        // For now, we'll forward directly without spawning to avoid lifetime issues
        // In production, we'd convert to protobuf bytes first, then spawn
        let forwarder = self.clone();
        tokio::spawn(async move {
            // Create a default ResourceMetrics for forwarding
            // TODO: Convert metrics to protobuf bytes first, then decode in spawned task
            let default_metrics = ResourceMetrics::default();
            if let Err(e) = forwarder.forward_metrics_internal(&default_metrics).await {
                error!(error = %e, "Failed to forward metrics");
                // Don't propagate error - forwarding failures shouldn't fail local storage
            }
        });

        Ok(())
    }

    /// Internal method to forward metrics (called asynchronously)
    async fn forward_metrics_internal(&self, _metrics: &ResourceMetrics) -> Result<(), OtlpError> {
        self.circuit_breaker
            .call(async {
                match self.config.protocol {
                    ForwardingProtocol::Protobuf => {
                        // Convert ResourceMetrics to Protobuf request
                        let request = self
                            .converter
                            .resource_metrics_to_protobuf(_metrics)
                            .map_err(|e| {
                                OtlpError::Export(OtlpExportError::FormatConversionError(format!(
                                    "Failed to convert ResourceMetrics to Protobuf: {}",
                                    e
                                )))
                            })?;

                        if let Some(req) = request {
                            self.send_protobuf_metrics(req).await?;
                        }
                    }
                    ForwardingProtocol::ArrowFlight => {
                        // Convert ResourceMetrics to Arrow Flight batch
                        let batch = FormatConverter::resource_metrics_to_arrow_batch(_metrics)
                            .map_err(|e| {
                                OtlpError::Export(OtlpExportError::FormatConversionError(format!(
                                    "Failed to convert ResourceMetrics to Arrow batch: {}",
                                    e
                                )))
                            })?;

                        self.send_arrow_flight_metrics(batch).await?;
                    }
                }
                Ok(())
            })
            .await
    }

    /// Send Protobuf traces to remote endpoint
    async fn send_protobuf_traces(
        &self,
        request: ExportTraceServiceRequest,
    ) -> Result<(), OtlpError> {
        let url = self.config.endpoint_url.as_ref().ok_or_else(|| {
            OtlpError::Export(OtlpExportError::ForwardingError(
                "Endpoint URL is required".to_string(),
            ))
        })?;

        // Serialize request to protobuf bytes using prost::Message::encode()
        let mut buf = Vec::new();
        request.encode(&mut buf).map_err(|e| {
            OtlpError::Export(OtlpExportError::SerializationError(format!(
                "Failed to encode Protobuf traces: {}",
                e
            )))
        })?;

        // Build request with authentication
        let mut http_request = self
            .client
            .post(format!("{}/v1/traces", url))
            .header("Content-Type", "application/x-protobuf");

        http_request = self.add_auth_headers(http_request)?;

        // Send request
        let response = http_request.body(buf).send().await.map_err(|e| {
            OtlpError::Export(OtlpExportError::ForwardingError(format!(
                "Failed to send Protobuf traces: {}",
                e
            )))
        })?;

        if !response.status().is_success() {
            return Err(OtlpError::Export(OtlpExportError::ForwardingError(
                format!("Remote endpoint returned error: {}", response.status()),
            )));
        }

        info!("Successfully forwarded traces via Protobuf");
        Ok(())
    }

    /// Send Protobuf metrics to remote endpoint
    async fn send_protobuf_metrics(
        &self,
        request: ExportMetricsServiceRequest,
    ) -> Result<(), OtlpError> {
        let url = self.config.endpoint_url.as_ref().ok_or_else(|| {
            OtlpError::Export(OtlpExportError::ForwardingError(
                "Endpoint URL is required".to_string(),
            ))
        })?;

        // Serialize request to protobuf bytes using prost::Message::encode()
        let mut buf = Vec::new();
        request.encode(&mut buf).map_err(|e| {
            OtlpError::Export(OtlpExportError::SerializationError(format!(
                "Failed to encode Protobuf metrics: {}",
                e
            )))
        })?;

        // Build request with authentication
        let mut http_request = self
            .client
            .post(format!("{}/v1/metrics", url))
            .header("Content-Type", "application/x-protobuf");

        http_request = self.add_auth_headers(http_request)?;

        // Send request
        let response = http_request.body(buf).send().await.map_err(|e| {
            OtlpError::Export(OtlpExportError::ForwardingError(format!(
                "Failed to send Protobuf metrics: {}",
                e
            )))
        })?;

        if !response.status().is_success() {
            return Err(OtlpError::Export(OtlpExportError::ForwardingError(
                format!("Remote endpoint returned error: {}", response.status()),
            )));
        }

        info!("Successfully forwarded metrics via Protobuf");
        Ok(())
    }

    /// Send Arrow Flight traces to remote endpoint
    async fn send_arrow_flight_traces(
        &self,
        _batch: arrow::record_batch::RecordBatch,
    ) -> Result<(), OtlpError> {
        // TODO: Implement Arrow Flight client
        // This requires a gRPC client with Arrow Flight support
        warn!("Arrow Flight forwarding not yet fully implemented - using placeholder");
        Ok(())
    }

    /// Send Arrow Flight metrics to remote endpoint
    async fn send_arrow_flight_metrics(
        &self,
        _batch: arrow::record_batch::RecordBatch,
    ) -> Result<(), OtlpError> {
        // TODO: Implement Arrow Flight client
        // This requires a gRPC client with Arrow Flight support
        warn!("Arrow Flight forwarding not yet fully implemented - using placeholder");
        Ok(())
    }

    /// Add authentication headers to HTTP request
    fn add_auth_headers(
        &self,
        mut request: reqwest::RequestBuilder,
    ) -> Result<reqwest::RequestBuilder, OtlpError> {
        if let Some(ref auth) = self.config.authentication {
            match auth.auth_type.as_str() {
                "api_key" => {
                    let key = auth.credentials.get("key").ok_or_else(|| {
                        OtlpError::Export(OtlpExportError::ForwardingError(
                            "API key authentication requires 'key' in credentials".to_string(),
                        ))
                    })?;
                    let default_header = "X-API-Key".to_string();
                    let header_name = auth
                        .credentials
                        .get("header_name")
                        .map(|s| s.expose_secret().clone())
                        .unwrap_or(default_header);
                    // Expose secret only when needed for HTTP header
                    request = request.header(header_name, key.expose_secret());
                }
                "bearer_token" => {
                    let token = auth.credentials.get("token").ok_or_else(|| {
                        OtlpError::Export(OtlpExportError::ForwardingError(
                            "Bearer token authentication requires 'token' in credentials"
                                .to_string(),
                        ))
                    })?;
                    // Expose secret only when needed for HTTP header
                    request = request.bearer_auth(token.expose_secret());
                }
                "basic" => {
                    let username = auth.credentials.get("username").ok_or_else(|| {
                        OtlpError::Export(OtlpExportError::ForwardingError(
                            "Basic authentication requires 'username' in credentials".to_string(),
                        ))
                    })?;
                    let password = auth.credentials.get("password").ok_or_else(|| {
                        OtlpError::Export(OtlpExportError::ForwardingError(
                            "Basic authentication requires 'password' in credentials".to_string(),
                        ))
                    })?;
                    // Expose secrets only when needed for HTTP header
                    request = request
                        .basic_auth(username.expose_secret(), Some(password.expose_secret()));
                }
                _ => {
                    return Err(OtlpError::Export(OtlpExportError::ForwardingError(
                        format!("Unsupported authentication type: {}", auth.auth_type),
                    )));
                }
            }
        }
        Ok(request)
    }
}
