//! gRPC server for receiving OTLP messages
//!
//! Implements OTLP TraceService and MetricsService using tonic gRPC framework.

use crate::error::OtlpError;
use crate::otlp::OtlpFileExporter;
use opentelemetry_proto::tonic::collector::metrics::v1::{
    ExportMetricsServiceRequest, ExportMetricsServiceResponse,
    metrics_service_server::{MetricsService, MetricsServiceServer},
};
use opentelemetry_proto::tonic::collector::trace::v1::{
    ExportTraceServiceRequest, ExportTraceServiceResponse,
    trace_service_server::{TraceService, TraceServiceServer},
};
use opentelemetry_sdk::metrics::data::ResourceMetrics;
use opentelemetry_sdk::trace::SpanData;
use std::sync::Arc;
use tonic::Request;
use tonic::Response;
use tonic::Status;
use tonic::transport::Server;
use tracing::{error, info};

/// gRPC server for OTLP messages
#[derive(Debug, Clone)]
pub struct OtlpGrpcServer {
    file_exporter: Arc<OtlpFileExporter>,
}

impl OtlpGrpcServer {
    /// Create a new gRPC server
    pub fn new(file_exporter: Arc<OtlpFileExporter>) -> Self {
        Self { file_exporter }
    }

    /// Start the gRPC server on the specified address
    pub async fn start(&self, addr: std::net::SocketAddr) -> Result<(), OtlpError> {
        info!("Starting OTLP gRPC server on {}", addr);

        let trace_service = TraceServiceImpl {
            file_exporter: self.file_exporter.clone(),
        };

        let metrics_service = MetricsServiceImpl {
            file_exporter: self.file_exporter.clone(),
        };

        Server::builder()
            .add_service(TraceServiceServer::new(trace_service))
            .add_service(MetricsServiceServer::new(metrics_service))
            .serve(addr)
            .await
            .map_err(|e| {
                OtlpError::Server(crate::error::OtlpServerError::StartupError(e.to_string()))
            })?;

        Ok(())
    }
}

/// Trace service implementation
#[derive(Debug, Clone)]
pub struct TraceServiceImpl {
    pub(crate) file_exporter: Arc<OtlpFileExporter>,
}

#[tonic::async_trait]
impl TraceService for TraceServiceImpl {
    async fn export(
        &self,
        request: Request<ExportTraceServiceRequest>,
    ) -> Result<Response<ExportTraceServiceResponse>, Status> {
        let req = request.into_inner();

        // Convert OTLP protobuf to SpanData
        // This is a simplified conversion - full implementation would use opentelemetry-proto conversion utilities
        let spans = convert_trace_request_to_spans(&req)
            .map_err(|e| Status::internal(format!("Failed to convert traces: {}", e)))?;

        if !spans.is_empty() {
            // Export spans using the file exporter directly
            // TODO: Use proper opentelemetry-proto conversion when spans are properly converted
            if let Err(e) = self.file_exporter.export_traces(spans).await {
                error!("Failed to export traces: {}", e);
                return Err(Status::internal(format!("Failed to export traces: {}", e)));
            }
        }

        Ok(Response::new(ExportTraceServiceResponse {
            partial_success: None,
        }))
    }
}

/// Helper function to create a temporary metrics server for extraction
///
/// This creates a temporary gRPC server that can capture ExportMetricsServiceRequest.
/// Used by the metrics extraction function to convert ResourceMetrics to protobuf.
pub(crate) async fn create_temporary_metrics_server(
    capture_tx: tokio::sync::oneshot::Sender<ExportMetricsServiceRequest>,
) -> Result<(tokio::task::JoinHandle<()>, String), anyhow::Error> {
    use opentelemetry_proto::tonic::collector::metrics::v1::metrics_service_server::{
        MetricsService, MetricsServiceServer,
    };
    use std::sync::Arc;
    use tonic::{Request, Response, Status};

    // Create a capture service that reuses the same pattern as MetricsServiceImpl
    struct CaptureMetricsService {
        tx: Arc<
            tokio::sync::Mutex<Option<tokio::sync::oneshot::Sender<ExportMetricsServiceRequest>>>,
        >,
    }

    #[tonic::async_trait]
    impl MetricsService for CaptureMetricsService {
        async fn export(
            &self,
            request: Request<ExportMetricsServiceRequest>,
        ) -> Result<Response<ExportMetricsServiceResponse>, Status> {
            let req = request.into_inner();

            // Send the captured request
            if let Some(sender) = self.tx.lock().await.take() {
                let _ = sender.send(req.clone());
            }

            Ok(Response::new(ExportMetricsServiceResponse {
                partial_success: None,
            }))
        }
    }

    // Find an available port using tokio::net::TcpListener
    // This must be called from within a Tokio runtime context
    // If we're not in a runtime, we need to spawn the server creation in a task
    // that has access to a runtime (the library's main runtime)
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .map_err(|e| anyhow::anyhow!("Failed to bind to port: {}", e))?;
    let addr = listener
        .local_addr()
        .map_err(|e| anyhow::anyhow!("Failed to get local address: {}", e))?;
    let addr_str = format!("http://{}", addr);

    // Create the capture service
    let capture_service = CaptureMetricsService {
        tx: Arc::new(tokio::sync::Mutex::new(Some(capture_tx))),
    };

    // Start the temporary server in a background task
    let server_handle = tokio::spawn(async move {
        let server = Server::builder()
            .add_service(MetricsServiceServer::new(capture_service))
            .serve_with_incoming(tokio_stream::wrappers::TcpListenerStream::new(listener))
            .await;

        if let Err(e) = server {
            tracing::error!("Temporary gRPC server error: {}", e);
        }
    });

    Ok((server_handle, addr_str))
}

/// Metrics service implementation
#[derive(Debug, Clone)]
pub struct MetricsServiceImpl {
    pub(crate) file_exporter: Arc<OtlpFileExporter>,
}

#[tonic::async_trait]
impl MetricsService for MetricsServiceImpl {
    async fn export(
        &self,
        request: Request<ExportMetricsServiceRequest>,
    ) -> Result<Response<ExportMetricsServiceResponse>, Status> {
        let req = request.into_inner();

        // As an OpenTelemetry endpoint, we receive Protobuf (ExportMetricsServiceRequest) directly via gRPC.
        // We don't receive ResourceMetrics - that's an SDK-internal structure that only exists in the client.
        // So we can convert Protobuf → InternalResourceMetrics → Arrow directly (NO PROXY NEEDED!)
        if let Err(e) = self.file_exporter.export_metrics_from_protobuf(&req).await {
            error!("Failed to export metrics: {}", e);
            return Err(Status::internal(format!("Failed to export metrics: {}", e)));
        }

        Ok(Response::new(ExportMetricsServiceResponse {
            partial_success: None,
        }))
    }
}

/// Convert OTLP trace request to SpanData
/// Converts protobuf ResourceSpans to SDK SpanData format
pub(crate) fn convert_trace_request_to_spans(
    req: &ExportTraceServiceRequest,
) -> Result<Vec<SpanData>, anyhow::Error> {
    use opentelemetry::KeyValue;
    use opentelemetry::trace::{
        SpanContext, SpanId, SpanKind, Status, TraceFlags, TraceId, TraceState,
    };
    use std::time::{Duration, UNIX_EPOCH};

    let mut spans = Vec::new();

    for resource_span in &req.resource_spans {
        // Extract resource attributes
        let resource_attrs = if let Some(ref resource) = resource_span.resource {
            resource
                .attributes
                .iter()
                .map(|kv| {
                    let value = kv.value.as_ref().and_then(|v| match &v.value {
                        Some(
                            opentelemetry_proto::tonic::common::v1::any_value::Value::StringValue(
                                s,
                            ),
                        ) => Some(opentelemetry::Value::String(s.clone().into())),
                        Some(
                            opentelemetry_proto::tonic::common::v1::any_value::Value::IntValue(i),
                        ) => Some(opentelemetry::Value::I64(*i)),
                        Some(
                            opentelemetry_proto::tonic::common::v1::any_value::Value::DoubleValue(
                                d,
                            ),
                        ) => Some(opentelemetry::Value::F64(*d)),
                        Some(
                            opentelemetry_proto::tonic::common::v1::any_value::Value::BoolValue(b),
                        ) => Some(opentelemetry::Value::Bool(*b)),
                        _ => None,
                    });
                    KeyValue::new(
                        kv.key.clone(),
                        value.unwrap_or(opentelemetry::Value::String("".to_string().into())),
                    )
                })
                .collect()
        } else {
            vec![]
        };

        for scope_span in &resource_span.scope_spans {
            for span in &scope_span.spans {
                // Extract trace and span IDs
                if span.trace_id.len() != 16 || span.span_id.len() != 8 {
                    continue; // Skip invalid spans
                }

                let trace_id = TraceId::from_bytes([
                    span.trace_id[0],
                    span.trace_id[1],
                    span.trace_id[2],
                    span.trace_id[3],
                    span.trace_id[4],
                    span.trace_id[5],
                    span.trace_id[6],
                    span.trace_id[7],
                    span.trace_id[8],
                    span.trace_id[9],
                    span.trace_id[10],
                    span.trace_id[11],
                    span.trace_id[12],
                    span.trace_id[13],
                    span.trace_id[14],
                    span.trace_id[15],
                ]);

                let span_id = SpanId::from_bytes([
                    span.span_id[0],
                    span.span_id[1],
                    span.span_id[2],
                    span.span_id[3],
                    span.span_id[4],
                    span.span_id[5],
                    span.span_id[6],
                    span.span_id[7],
                ]);

                let parent_span_id = if span.parent_span_id.len() == 8 {
                    SpanId::from_bytes([
                        span.parent_span_id[0],
                        span.parent_span_id[1],
                        span.parent_span_id[2],
                        span.parent_span_id[3],
                        span.parent_span_id[4],
                        span.parent_span_id[5],
                        span.parent_span_id[6],
                        span.parent_span_id[7],
                    ])
                } else {
                    SpanId::INVALID
                };

                let span_context = SpanContext::new(
                    trace_id,
                    span_id,
                    TraceFlags::default(),
                    false,
                    TraceState::default(),
                );

                // Convert span kind
                let span_kind = match span.kind {
                    0 => SpanKind::Internal,
                    1 => SpanKind::Server,
                    2 => SpanKind::Client,
                    3 => SpanKind::Producer,
                    4 => SpanKind::Consumer,
                    _ => SpanKind::Internal,
                };

                // Convert timestamps
                let start_time = UNIX_EPOCH + Duration::from_nanos(span.start_time_unix_nano);
                let end_time = UNIX_EPOCH + Duration::from_nanos(span.end_time_unix_nano);

                // Convert attributes
                let mut attributes = resource_attrs.clone();
                for attr in &span.attributes {
                    let value = attr.value.as_ref().and_then(|v| match &v.value {
                        Some(
                            opentelemetry_proto::tonic::common::v1::any_value::Value::StringValue(
                                s,
                            ),
                        ) => Some(opentelemetry::Value::String(s.clone().into())),
                        Some(
                            opentelemetry_proto::tonic::common::v1::any_value::Value::IntValue(i),
                        ) => Some(opentelemetry::Value::I64(*i)),
                        Some(
                            opentelemetry_proto::tonic::common::v1::any_value::Value::DoubleValue(
                                d,
                            ),
                        ) => Some(opentelemetry::Value::F64(*d)),
                        Some(
                            opentelemetry_proto::tonic::common::v1::any_value::Value::BoolValue(b),
                        ) => Some(opentelemetry::Value::Bool(*b)),
                        _ => None,
                    });
                    if let Some(val) = value {
                        attributes.push(KeyValue::new(attr.key.clone(), val));
                    }
                }

                // Convert status
                let status = if let Some(ref s) = span.status {
                    match s.code {
                        1 => Status::Ok,
                        2 => Status::Error {
                            description: s.message.clone().into(),
                        },
                        _ => Status::Unset,
                    }
                } else {
                    Status::Unset
                };

                // Get instrumentation scope
                let instrumentation_scope = if let Some(ref scope) = scope_span.scope {
                    opentelemetry::InstrumentationScope::builder(scope.name.clone())
                        .with_version(scope.version.clone())
                        .build()
                } else {
                    opentelemetry::InstrumentationScope::builder("unknown").build()
                };

                let span_data = SpanData {
                    span_context,
                    parent_span_id,
                    span_kind,
                    name: std::borrow::Cow::Owned(span.name.clone()),
                    start_time,
                    end_time,
                    attributes,
                    events: opentelemetry_sdk::trace::SpanEvents::default(),
                    links: opentelemetry_sdk::trace::SpanLinks::default(),
                    status,
                    dropped_attributes_count: span.dropped_attributes_count,
                    parent_span_is_remote: false,
                    instrumentation_scope,
                };

                spans.push(span_data);
            }
        }
    }

    Ok(spans)
}

/// Convert OTLP metrics request to ResourceMetrics
/// Converts protobuf ResourceMetrics to SDK ResourceMetrics format
pub(crate) fn convert_metrics_request_to_resource_metrics(
    req: &ExportMetricsServiceRequest,
) -> Result<Option<ResourceMetrics>, anyhow::Error> {
    use opentelemetry::KeyValue;

    if req.resource_metrics.is_empty() {
        return Ok(None);
    }

    // Convert the first ResourceMetrics (simplified - full implementation would handle all)
    let resource_metric = &req.resource_metrics[0];

    // Extract resource attributes (preserved for future use when ResourceMetrics construction is available)
    let _resource_attrs = if let Some(ref resource) = resource_metric.resource {
        resource
            .attributes
            .iter()
            .filter_map(|kv| {
                let value = kv.value.as_ref().and_then(|v| match &v.value {
                    Some(
                        opentelemetry_proto::tonic::common::v1::any_value::Value::StringValue(s),
                    ) => Some(opentelemetry::Value::String(s.clone().into())),
                    Some(opentelemetry_proto::tonic::common::v1::any_value::Value::IntValue(i)) => {
                        Some(opentelemetry::Value::I64(*i))
                    }
                    Some(
                        opentelemetry_proto::tonic::common::v1::any_value::Value::DoubleValue(d),
                    ) => Some(opentelemetry::Value::F64(*d)),
                    Some(opentelemetry_proto::tonic::common::v1::any_value::Value::BoolValue(
                        b,
                    )) => Some(opentelemetry::Value::Bool(*b)),
                    _ => None,
                });
                value.map(|val| KeyValue::new(kv.key.clone(), val))
            })
            .collect()
    } else {
        vec![]
    };

    // Create ResourceMetrics
    // Note: Full implementation would convert ScopeMetrics and Metric data structures
    // For now, we create a minimal ResourceMetrics using Default
    // ResourceMetrics fields are private in opentelemetry-sdk 0.31, so we can't construct it directly
    // The resource attributes are preserved in the Arrow format when written to file
    // TODO: Use opentelemetry-proto conversion utilities for full ResourceMetrics construction
    let resource_metrics = ResourceMetrics::default();

    Ok(Some(resource_metrics))
}
