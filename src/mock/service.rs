//! Mock OTLP service for testing
//!
//! Provides a mock service that can receive OTLP messages via both gRPC interface
//! and public API methods for end-to-end testing.

use opentelemetry_proto::tonic::collector::metrics::v1::{
    metrics_service_server::{MetricsService, MetricsServiceServer},
    ExportMetricsServiceRequest, ExportMetricsServiceResponse,
};
use opentelemetry_proto::tonic::collector::trace::v1::{
    trace_service_server::{TraceService, TraceServiceServer},
    ExportTraceServiceRequest, ExportTraceServiceResponse,
};
use opentelemetry_sdk::metrics::data::ResourceMetrics;
use opentelemetry_sdk::trace::SpanData;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio_stream::wrappers::TcpListenerStream;
use tonic::{Request, Response, Status};
use tracing::info;

// Re-export Arrow Flight types for mock service
use arrow_flight::{
    flight_service_server::{FlightService, FlightServiceServer},
    Action, ActionType, Criteria, Empty, FlightData, FlightDescriptor, FlightInfo,
    HandshakeRequest, HandshakeResponse, PollInfo, PutResult, SchemaResult, Ticket,
};
use std::pin::Pin;
use tokio_stream::{Stream, StreamExt};
use tonic::Streaming;
use arrow::record_batch::RecordBatch;

/// Mock OTLP service state
#[derive(Debug, Default)]
struct MockServiceState {
    /// Traces received via mock service
    received_traces: Vec<SpanData>,
    /// Metrics received via mock service
    received_metrics: Vec<ResourceMetrics>,
    /// Count of gRPC calls received
    grpc_calls: u64,
    /// Count of public API calls received
    api_calls: u64,
}

/// Mock OTLP service for testing
#[derive(Debug, Clone)]
pub struct MockOtlpService {
    state: Arc<RwLock<MockServiceState>>,
}

/// Result type for mock service start
pub struct MockServiceAddresses {
    /// Protobuf gRPC server address
    pub protobuf_addr: String,
    /// Arrow Flight gRPC server address
    pub arrow_flight_addr: String,
}

impl MockOtlpService {
    /// Create a new mock OTLP service
    pub fn new() -> Self {
        Self {
            state: Arc::new(RwLock::new(MockServiceState::default())),
        }
    }

    /// Start the mock service with both Protobuf and Arrow Flight servers
    /// Returns the addresses where the servers are listening
    pub async fn start(&self) -> Result<MockServiceAddresses, String> {
        // Start Protobuf server
        let protobuf_addr = "127.0.0.1:0".parse::<SocketAddr>()
            .map_err(|e| format!("Failed to parse protobuf address: {}", e))?;
        let protobuf_listener = tokio::net::TcpListener::bind(&protobuf_addr)
            .await
            .map_err(|e| format!("Failed to bind protobuf listener: {}", e))?;
        let protobuf_addr = protobuf_listener.local_addr()
            .map_err(|e| format!("Failed to get protobuf local address: {}", e))?;
        let protobuf_addr_str = format!("http://{}", protobuf_addr);

        // Start Arrow Flight server
        let arrow_flight_addr = "127.0.0.1:0".parse::<SocketAddr>()
            .map_err(|e| format!("Failed to parse arrow flight address: {}", e))?;
        let arrow_flight_listener = tokio::net::TcpListener::bind(&arrow_flight_addr)
            .await
            .map_err(|e| format!("Failed to bind arrow flight listener: {}", e))?;
        let arrow_flight_addr = arrow_flight_listener.local_addr()
            .map_err(|e| format!("Failed to get arrow flight local address: {}", e))?;
        let arrow_flight_addr_str = format!("http://{}", arrow_flight_addr);

        let state = self.state.clone();

        // Start Protobuf server in background
        let protobuf_state = state.clone();
        tokio::spawn(async move {
            let trace_service = MockTraceServiceImpl {
                state: protobuf_state.clone(),
            };
            let metrics_service = MockMetricsServiceImpl {
                state: protobuf_state,
            };

            let server = tonic::transport::Server::builder()
                .add_service(TraceServiceServer::new(trace_service))
                .add_service(MetricsServiceServer::new(metrics_service))
                .serve_with_incoming(TcpListenerStream::new(protobuf_listener))
                .await;

            if let Err(e) = server {
                eprintln!("Protobuf server error: {}", e);
            }
        });

        // Start Arrow Flight server in background
        let arrow_flight_state = state;
        tokio::spawn(async move {
            let flight_service = MockFlightServiceImpl {
                state: arrow_flight_state,
            };

            let svc = FlightServiceServer::new(flight_service);
            let server = tonic::transport::Server::builder()
                .add_service(svc)
                .serve_with_incoming(TcpListenerStream::new(arrow_flight_listener))
                .await;

            if let Err(e) = server {
                eprintln!("Arrow Flight server error: {}", e);
            }
        });

        info!(
            protobuf_addr = %protobuf_addr_str,
            arrow_flight_addr = %arrow_flight_addr_str,
            "Mock OTLP service started"
        );

        Ok(MockServiceAddresses {
            protobuf_addr: protobuf_addr_str,
            arrow_flight_addr: arrow_flight_addr_str,
        })
    }

    /// Receive a trace via public API
    pub async fn receive_trace(&self, span: SpanData) {
        let mut state = self.state.write().await;
        state.received_traces.push(span);
        state.api_calls += 1;
    }

    /// Receive metrics via public API
    pub async fn receive_metric(&self, metrics: ResourceMetrics) {
        let mut state = self.state.write().await;
        state.received_metrics.push(metrics);
        state.api_calls += 1;
    }

    /// Assert that the expected number of traces were received
    pub async fn assert_traces_received(&self, expected_count: usize) -> Result<(), String> {
        let state = self.state.read().await;
        if state.received_traces.len() != expected_count {
            Err(format!(
                "Expected {} traces, but received {}",
                expected_count,
                state.received_traces.len()
            ))
        } else {
            Ok(())
        }
    }

    /// Assert that the expected number of metrics were received
    pub async fn assert_metrics_received(&self, expected_count: usize) -> Result<(), String> {
        let state = self.state.read().await;
        if state.received_metrics.len() != expected_count {
            Err(format!(
                "Expected {} metrics, but received {}",
                expected_count,
                state.received_metrics.len()
            ))
        } else {
            Ok(())
        }
    }

    /// Get the number of gRPC calls received
    pub async fn grpc_calls_count(&self) -> u64 {
        let state = self.state.read().await;
        state.grpc_calls
    }

    /// Get the number of API calls received
    pub async fn api_calls_count(&self) -> u64 {
        let state = self.state.read().await;
        state.api_calls
    }

    /// Reset the mock service state (for test isolation)
    pub async fn reset(&self) {
        let mut state = self.state.write().await;
        *state = MockServiceState::default();
    }
}

impl Default for MockOtlpService {
    fn default() -> Self {
        Self::new()
    }
}

/// Mock Trace Service implementation for Protobuf
#[derive(Debug, Clone)]
struct MockTraceServiceImpl {
    state: Arc<RwLock<MockServiceState>>,
}

#[tonic::async_trait]
impl TraceService for MockTraceServiceImpl {
    async fn export(
        &self,
        request: Request<ExportTraceServiceRequest>,
    ) -> Result<Response<ExportTraceServiceResponse>, Status> {
        let req = request.into_inner();

        // Convert protobuf to spans using the same conversion as the real server
        let spans = crate::otlp::server::convert_trace_request_to_spans(&req)
            .map_err(|e| Status::internal(format!("Failed to convert traces: {}", e)))?;

        // Store in mock state
        {
            let mut state = self.state.write().await;
            state.received_traces.extend(spans);
            state.grpc_calls += 1;
        }

        Ok(Response::new(ExportTraceServiceResponse {
            partial_success: None,
        }))
    }
}

/// Mock Metrics Service implementation for Protobuf
#[derive(Debug, Clone)]
struct MockMetricsServiceImpl {
    state: Arc<RwLock<MockServiceState>>,
}

#[tonic::async_trait]
impl MetricsService for MockMetricsServiceImpl {
    async fn export(
        &self,
        request: Request<ExportMetricsServiceRequest>,
    ) -> Result<Response<ExportMetricsServiceResponse>, Status> {
        let req = request.into_inner();

        // Convert protobuf to ResourceMetrics using the same conversion as the real server
        let resource_metrics = crate::otlp::server::convert_metrics_request_to_resource_metrics(&req)
            .map_err(|e| Status::internal(format!("Failed to convert metrics: {}", e)))?;

        // Store in mock state
        if let Some(metrics) = resource_metrics {
            let mut state = self.state.write().await;
            state.received_metrics.push(metrics);
            state.grpc_calls += 1;
        }

        Ok(Response::new(ExportMetricsServiceResponse {
            partial_success: None,
        }))
    }
}

/// Mock Arrow Flight Service implementation
#[derive(Debug, Clone)]
struct MockFlightServiceImpl {
    state: Arc<RwLock<MockServiceState>>,
}

#[tonic::async_trait]
impl FlightService for MockFlightServiceImpl {
    type HandshakeStream = Pin<Box<dyn Stream<Item = Result<HandshakeResponse, Status>> + Send>>;
    type DoGetStream = Pin<Box<dyn Stream<Item = Result<FlightData, Status>> + Send>>;
    type DoPutStream = Pin<Box<dyn Stream<Item = Result<PutResult, Status>> + Send>>;
    type DoActionStream = Pin<Box<dyn Stream<Item = Result<arrow_flight::Result, Status>> + Send>>;
    type DoExchangeStream = Pin<Box<dyn Stream<Item = Result<FlightData, Status>> + Send>>;
    type ListActionsStream = Pin<Box<dyn Stream<Item = Result<ActionType, Status>> + Send>>;
    type ListFlightsStream = Pin<Box<dyn Stream<Item = Result<FlightInfo, Status>> + Send>>;

    async fn handshake(
        &self,
        _request: Request<Streaming<HandshakeRequest>>,
    ) -> Result<Response<Self::HandshakeStream>, Status> {
        Err(Status::unimplemented("Handshake not implemented"))
    }

    async fn list_flights(
        &self,
        _request: Request<Criteria>,
    ) -> Result<Response<Self::ListFlightsStream>, Status> {
        Err(Status::unimplemented("ListFlights not implemented"))
    }

    async fn get_flight_info(
        &self,
        _request: Request<FlightDescriptor>,
    ) -> Result<Response<FlightInfo>, Status> {
        Err(Status::unimplemented("GetFlightInfo not implemented"))
    }

    async fn poll_flight_info(
        &self,
        _request: Request<FlightDescriptor>,
    ) -> Result<Response<PollInfo>, Status> {
        Err(Status::unimplemented("PollFlightInfo not implemented"))
    }

    async fn get_schema(
        &self,
        _request: Request<FlightDescriptor>,
    ) -> Result<Response<SchemaResult>, Status> {
        Err(Status::unimplemented("GetSchema not implemented"))
    }

    async fn do_get(
        &self,
        _request: Request<Ticket>,
    ) -> Result<Response<Self::DoGetStream>, Status> {
        Err(Status::unimplemented("DoGet not implemented"))
    }

    async fn do_put(
        &self,
        request: Request<Streaming<FlightData>>,
    ) -> Result<Response<Self::DoPutStream>, Status> {
        let mut stream = request.into_inner();
        let state = self.state.clone();

        // Process incoming FlightData stream
        tokio::spawn(async move {
            let mut batches = Vec::new();

            while let Some(Ok(flight_data)) = stream.next().await {
                // Decode FlightData to RecordBatch
                if let Ok(batch) = decode_flight_data(&flight_data) {
                    batches.push(batch);
                }
            }

            // Convert batches to spans/metrics and store in mock state
            for batch in batches {
                // Try to convert to traces
                if let Ok(spans) = crate::otlp::server_arrow::convert_arrow_batch_to_spans(&batch) {
                    if !spans.is_empty() {
                        let mut state = state.write().await;
                        state.received_traces.extend(spans);
                        state.grpc_calls += 1;
                        continue;
                    }
                }

                // Try to convert to metrics
                if let Ok(Some(metrics)) = crate::otlp::server_arrow::convert_arrow_batch_to_resource_metrics(&batch) {
                    let mut state = state.write().await;
                    state.received_metrics.push(metrics);
                    state.grpc_calls += 1;
                }
            }
        });

        // Return empty stream as acknowledgment
        let output = futures::stream::empty();
        Ok(Response::new(Box::pin(output)))
    }

    async fn do_action(
        &self,
        _request: Request<Action>,
    ) -> Result<Response<Self::DoActionStream>, Status> {
        Err(Status::unimplemented("DoAction not implemented"))
    }

    async fn list_actions(
        &self,
        _request: Request<Empty>,
    ) -> Result<Response<Self::ListActionsStream>, Status> {
        Err(Status::unimplemented("ListActions not implemented"))
    }

    async fn do_exchange(
        &self,
        _request: Request<Streaming<FlightData>>,
    ) -> Result<Response<Self::DoExchangeStream>, Status> {
        Err(Status::unimplemented("DoExchange not implemented"))
    }
}

/// Decode Arrow Flight data to RecordBatch
fn decode_flight_data(flight_data: &FlightData) -> Result<RecordBatch, anyhow::Error> {
    use arrow::ipc::reader::StreamReader;
    use std::io::Cursor;

    let data = &flight_data.data_header;
    let cursor = Cursor::new(data);
    let mut reader = StreamReader::try_new(cursor, None)
        .map_err(|e| anyhow::anyhow!("Failed to create StreamReader: {}", e))?;
    
    let batch = reader
        .next()
        .ok_or_else(|| anyhow::anyhow!("No batch in FlightData"))?
        .map_err(|e| anyhow::anyhow!("Failed to read batch: {}", e))?;
    
    Ok(batch)
}
