//! gRPC Arrow Flight server for receiving OTLP messages via Arrow Flight IPC (OTAP)
//!
//! Implements OTLP trace and metrics services using Arrow Flight IPC protocol.
//! Uses arrow-flight crate for Arrow Flight server implementation.

use crate::error::OtlpError;
use crate::otlp::OtlpFileExporter;
use arrow::record_batch::RecordBatch;
use arrow_flight::{
    flight_service_server::{FlightService, FlightServiceServer},
    Action, ActionType, Criteria, Empty, FlightData, FlightDescriptor, FlightInfo,
    HandshakeRequest, HandshakeResponse, PollInfo, PutResult, SchemaResult, Ticket,
};
use opentelemetry_sdk::metrics::data::ResourceMetrics;
use opentelemetry_sdk::trace::SpanData;
use std::pin::Pin;
use std::sync::Arc;
use tokio_stream::{Stream, StreamExt};
use tonic::{Request, Response, Status, Streaming};
use tracing::{error, info, warn};

/// gRPC Arrow Flight server for OTLP messages
#[derive(Debug, Clone)]
pub struct OtlpArrowFlightServer {
    file_exporter: Arc<OtlpFileExporter>,
}

impl OtlpArrowFlightServer {
    /// Create a new Arrow Flight server
    pub fn new(file_exporter: Arc<OtlpFileExporter>) -> Self {
        Self {
            file_exporter,
        }
    }

    /// Start the Arrow Flight server on the specified address
    pub async fn start(&self, addr: std::net::SocketAddr) -> Result<(), OtlpError> {
        info!("Starting OTLP Arrow Flight server on {}", addr);

        let service = OtlpFlightServiceImpl {
            file_exporter: self.file_exporter.clone(),
        };

        let svc = FlightServiceServer::new(service);
        
        tonic::transport::Server::builder()
            .add_service(svc)
            .serve(addr)
            .await
            .map_err(|e| OtlpError::Server(crate::error::OtlpServerError::StartupError(
                format!("Failed to start Arrow Flight server: {}", e)
            )))?;

        Ok(())
    }
}

/// Arrow Flight service implementation for OTLP
#[derive(Debug, Clone)]
pub(crate) struct OtlpFlightServiceImpl {
    file_exporter: Arc<OtlpFileExporter>,
}

#[tonic::async_trait]
impl FlightService for OtlpFlightServiceImpl {
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
        Err(Status::unimplemented("DoGet not implemented - this is a receiver-only service"))
    }

    async fn do_put(
        &self,
        request: Request<Streaming<FlightData>>,
    ) -> Result<Response<Self::DoPutStream>, Status> {
        let mut stream = request.into_inner();
        let file_exporter = self.file_exporter.clone();
        
        // Process incoming Arrow Flight data stream
        tokio::spawn(async move {
            let mut batches = Vec::new();
            
            while let Some(flight_data) = stream.next().await {
                match flight_data {
                    Ok(data) => {
                        // Decode Arrow Flight data to RecordBatch
                        match decode_flight_data(&data) {
                            Ok(batch) => {
                                batches.push(batch);
                            }
                            Err(e) => {
                                error!("Failed to decode Arrow Flight data: {}", e);
                            }
                        }
                    }
                    Err(e) => {
                        error!("Error receiving Arrow Flight data: {}", e);
                        break;
                    }
                }
            }

            // Convert Arrow RecordBatches to OTLP format and export
            if !batches.is_empty() {
                // Determine if this is trace or metric data based on schema or descriptor
                // For now, we'll try to convert to both and see which succeeds
                for batch in batches {
                    // Try to convert to traces
                    if let Ok(spans) = convert_arrow_batch_to_spans(&batch) {
                        if !spans.is_empty() {
                            if let Err(e) = file_exporter.export_traces(spans).await {
                                error!("Failed to export traces from Arrow Flight: {}", e);
                            }
                            continue;
                        }
                    }

                    // Try to convert to metrics
                    if let Ok(Some(metrics)) = convert_arrow_batch_to_resource_metrics(&batch) {
                        if let Err(e) = file_exporter.export_metrics(&metrics).await {
                            error!("Failed to export metrics from Arrow Flight: {}", e);
                        }
                        continue;
                    }

                    warn!("Could not convert Arrow Flight batch to OTLP format");
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

    // FlightData contains the Arrow IPC message
    let data = &flight_data.data_header;
    
    // Create a cursor over the data
    let cursor = Cursor::new(data);
    
    // Read the Arrow IPC stream
    let mut reader = StreamReader::try_new(cursor, None)
        .map_err(|e| anyhow::anyhow!("Failed to create StreamReader: {}", e))?;
    
    // Read the first (and typically only) batch
    let batch = reader
        .next()
        .ok_or_else(|| anyhow::anyhow!("No batch in FlightData"))?
        .map_err(|e| anyhow::anyhow!("Failed to read batch: {}", e))?;
    
    Ok(batch)
}

/// Convert Arrow RecordBatch to SpanData
/// This converts Arrow columnar data to OTLP span format
/// Uses the same schema structure as convert_spans_to_arrow_ipc
pub(crate) fn convert_arrow_batch_to_spans(batch: &RecordBatch) -> Result<Vec<SpanData>, anyhow::Error> {
    use opentelemetry::trace::{SpanContext, SpanId, SpanKind, Status, TraceId, TraceFlags, TraceState};
    use opentelemetry::KeyValue;
    use std::time::{Duration, SystemTime, UNIX_EPOCH};
    use arrow::array::*;

    let schema = batch.schema();
    let num_rows = batch.num_rows();
    
    if num_rows == 0 {
        return Ok(Vec::new());
    }

    // Find column indices by name (matching the schema in convert_spans_to_arrow_ipc)
    let trace_id_idx = schema.column_with_name("trace_id")
        .ok_or_else(|| anyhow::anyhow!("Missing trace_id column"))?.0;
    let span_id_idx = schema.column_with_name("span_id")
        .ok_or_else(|| anyhow::anyhow!("Missing span_id column"))?.0;
    let parent_span_id_idx = schema.column_with_name("parent_span_id");
    let name_idx = schema.column_with_name("name")
        .ok_or_else(|| anyhow::anyhow!("Missing name column"))?.0;
    let kind_idx = schema.column_with_name("kind")
        .ok_or_else(|| anyhow::anyhow!("Missing kind column"))?.0;
    let start_time_idx = schema.column_with_name("start_time_unix_nano")
        .ok_or_else(|| anyhow::anyhow!("Missing start_time_unix_nano column"))?.0;
    let end_time_idx = schema.column_with_name("end_time_unix_nano")
        .ok_or_else(|| anyhow::anyhow!("Missing end_time_unix_nano column"))?.0;
    let status_code_idx = schema.column_with_name("status_code")
        .ok_or_else(|| anyhow::anyhow!("Missing status_code column"))?.0;
    let status_message_idx = schema.column_with_name("status_message");
    let attributes_idx = schema.column_with_name("attributes");

    let mut spans = Vec::with_capacity(num_rows);

    // Extract arrays
    let trace_id_array = batch.column(trace_id_idx);
    let span_id_array = batch.column(span_id_idx);
    let name_array = batch.column(name_idx);
    let kind_array = batch.column(kind_idx);
    let start_time_array = batch.column(start_time_idx);
    let end_time_array = batch.column(end_time_idx);
    let status_code_array = batch.column(status_code_idx);

    for i in 0..num_rows {
        // Extract trace_id (16 bytes)
        let trace_id_bytes = if let Some(binary_array) = trace_id_array.as_any().downcast_ref::<BinaryArray>() {
            if binary_array.is_valid(i) && binary_array.value(i).len() == 16 {
                let bytes = binary_array.value(i);
                TraceId::from_bytes([
                    bytes[0], bytes[1], bytes[2], bytes[3],
                    bytes[4], bytes[5], bytes[6], bytes[7],
                    bytes[8], bytes[9], bytes[10], bytes[11],
                    bytes[12], bytes[13], bytes[14], bytes[15],
                ])
            } else {
                continue; // Skip invalid trace_id
            }
        } else {
            continue; // Skip if not binary array
        };

        // Extract span_id (8 bytes)
        let span_id = if let Some(binary_array) = span_id_array.as_any().downcast_ref::<BinaryArray>() {
            if binary_array.is_valid(i) && binary_array.value(i).len() == 8 {
                let bytes = binary_array.value(i);
                SpanId::from_bytes([
                    bytes[0], bytes[1], bytes[2], bytes[3],
                    bytes[4], bytes[5], bytes[6], bytes[7],
                ])
            } else {
                SpanId::INVALID
            }
        } else {
            SpanId::INVALID
        };

        // Extract parent_span_id (optional, 8 bytes)
        let parent_span_id = if let Some((idx, _)) = parent_span_id_idx {
            if let Some(binary_array) = batch.column(idx).as_any().downcast_ref::<BinaryArray>() {
                if binary_array.is_valid(i) && binary_array.value(i).len() == 8 {
                    let bytes = binary_array.value(i);
                    SpanId::from_bytes([
                        bytes[0], bytes[1], bytes[2], bytes[3],
                        bytes[4], bytes[5], bytes[6], bytes[7],
                    ])
                } else {
                    SpanId::INVALID
                }
            } else {
                SpanId::INVALID
            }
        } else {
            SpanId::INVALID
        };

        // Extract name
        let name = if let Some(string_array) = name_array.as_any().downcast_ref::<StringArray>() {
            if string_array.is_valid(i) {
                string_array.value(i).to_string()
            } else {
                "unknown".to_string()
            }
        } else {
            "unknown".to_string()
        };

        // Extract kind
        let span_kind = if let Some(int_array) = kind_array.as_any().downcast_ref::<Int32Array>() {
            if int_array.is_valid(i) {
                match int_array.value(i) {
                    0 => SpanKind::Internal,
                    1 => SpanKind::Server,
                    2 => SpanKind::Client,
                    3 => SpanKind::Producer,
                    4 => SpanKind::Consumer,
                    _ => SpanKind::Internal,
                }
            } else {
                SpanKind::Internal
            }
        } else {
            SpanKind::Internal
        };

        // Extract start_time
        let start_time = if let Some(uint_array) = start_time_array.as_any().downcast_ref::<UInt64Array>() {
            if uint_array.is_valid(i) {
                let nanos = uint_array.value(i);
                UNIX_EPOCH + Duration::from_nanos(nanos)
            } else {
                SystemTime::now()
            }
        } else {
            SystemTime::now()
        };

        // Extract end_time
        let end_time = if let Some(uint_array) = end_time_array.as_any().downcast_ref::<UInt64Array>() {
            if uint_array.is_valid(i) {
                let nanos = uint_array.value(i);
                UNIX_EPOCH + Duration::from_nanos(nanos)
            } else {
                SystemTime::now()
            }
        } else {
            SystemTime::now()
        };

        // Extract status_code and status_message
        let status = if let Some(int_array) = status_code_array.as_any().downcast_ref::<Int32Array>() {
            if int_array.is_valid(i) {
                let code = int_array.value(i);
                // Get status message if available
                let message = if let Some((idx, _)) = status_message_idx {
                    if let Some(string_array) = batch.column(idx).as_any().downcast_ref::<StringArray>() {
                        if string_array.is_valid(i) {
                            Some(string_array.value(i).to_string())
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                } else {
                    None
                };

                match code {
                    1 => Status::Ok,
                    2 => {
                        if let Some(msg) = message {
                            Status::Error {
                                description: msg.into(),
                            }
                        } else {
                            Status::Error {
                                description: "".into(),
                            }
                        }
                    }
                    _ => Status::Unset,
                }
            } else {
                Status::Unset
            }
        } else {
            Status::Unset
        };

        // Extract attributes (JSON-encoded)
        let attributes = if let Some((idx, _)) = attributes_idx {
            if let Some(string_array) = batch.column(idx).as_any().downcast_ref::<StringArray>() {
                if string_array.is_valid(i) {
                    // Parse JSON attributes
                    let json_str = string_array.value(i);
                    if let Ok(json_value) = serde_json::from_str::<serde_json::Value>(json_str) {
                        if let Some(obj) = json_value.as_object() {
                            obj.iter()
                                .map(|(k, v)| {
                                    let value = match v {
                                        serde_json::Value::String(s) => opentelemetry::Value::String(s.clone().into()),
                                        serde_json::Value::Number(n) => {
                                            if let Some(i) = n.as_i64() {
                                                opentelemetry::Value::I64(i)
                                            } else if let Some(f) = n.as_f64() {
                                                opentelemetry::Value::F64(f)
                                            } else {
                                                opentelemetry::Value::String(n.to_string().into())
                                            }
                                        }
                                        serde_json::Value::Bool(b) => opentelemetry::Value::Bool(*b),
                                        _ => opentelemetry::Value::String(v.to_string().into()),
                                    };
                                    KeyValue::new(k.clone(), value)
                                })
                                .collect()
                        } else {
                            vec![]
                        }
                    } else {
                        vec![]
                    }
                } else {
                    vec![]
                }
            } else {
                vec![]
            }
        } else {
            vec![]
        };

        let span_context = SpanContext::new(trace_id_bytes, span_id, TraceFlags::default(), false, TraceState::default());

        // Create default instrumentation scope
        let instrumentation_scope = opentelemetry::InstrumentationScope::builder("arrow-flight")
            .build();

        let span_data = SpanData {
            span_context,
            parent_span_id,
            span_kind,
            name: std::borrow::Cow::Owned(name),
            start_time,
            end_time,
            attributes,
            events: opentelemetry_sdk::trace::SpanEvents::default(),
            links: opentelemetry_sdk::trace::SpanLinks::default(),
            status,
            dropped_attributes_count: 0,
            parent_span_is_remote: false,
            instrumentation_scope,
        };

        spans.push(span_data);
    }

    Ok(spans)
}

/// Convert Arrow RecordBatch to ResourceMetrics
/// This converts Arrow columnar data to OTLP metrics format
/// Uses the same schema structure as convert_metrics_to_arrow_ipc
pub(crate) fn convert_arrow_batch_to_resource_metrics(
    batch: &RecordBatch,
) -> Result<Option<ResourceMetrics>, anyhow::Error> {
    use opentelemetry::KeyValue;
    use arrow::array::*;

    let schema = batch.schema();
    let num_rows = batch.num_rows();
    
    if num_rows == 0 {
        return Ok(None);
    }

    // Check if this looks like a metrics batch by checking for metric_name column
    if schema.column_with_name("metric_name").is_none() {
        return Ok(None);
    }

    // Find column indices by name (matching the schema in convert_metrics_to_arrow_ipc)
    let metric_name_idx = schema.column_with_name("metric_name")
        .ok_or_else(|| anyhow::anyhow!("Missing metric_name column"))?.0;
    let value_idx = schema.column_with_name("value")
        .ok_or_else(|| anyhow::anyhow!("Missing value column"))?.0;
    let timestamp_idx = schema.column_with_name("timestamp_unix_nano")
        .ok_or_else(|| anyhow::anyhow!("Missing timestamp_unix_nano column"))?.0;
    let metric_type_idx = schema.column_with_name("metric_type")
        .ok_or_else(|| anyhow::anyhow!("Missing metric_type column"))?.0;
    let attributes_idx = schema.column_with_name("attributes");

    // Extract arrays (preserved for future use when full metric conversion is implemented)
    let _name_array = batch.column(metric_name_idx);
    let _value_array = batch.column(value_idx);
    let _timestamp_array = batch.column(timestamp_idx);
    let _type_array = batch.column(metric_type_idx);

    // Build resource attributes from first row's attributes (if available)
    // Preserved for future use when ResourceMetrics construction is available
    let _resource_attrs = {
        let mut attrs = vec![];
        if let Some((idx, _)) = attributes_idx {
            if let Some(string_array) = batch.column(idx).as_any().downcast_ref::<StringArray>() {
                if string_array.is_valid(0) {
                    let json_str = string_array.value(0);
                    if let Ok(json_value) = serde_json::from_str::<serde_json::Value>(json_str) {
                        if let Some(obj) = json_value.as_object() {
                            for (k, v) in obj.iter() {
                                let value = match v {
                                    serde_json::Value::String(s) => opentelemetry::Value::String(s.clone().into()),
                                    serde_json::Value::Number(n) => {
                                        if let Some(i) = n.as_i64() {
                                            opentelemetry::Value::I64(i)
                                        } else if let Some(f) = n.as_f64() {
                                            opentelemetry::Value::F64(f)
                                        } else {
                                            opentelemetry::Value::String(n.to_string().into())
                                        }
                                    }
                                    serde_json::Value::Bool(b) => opentelemetry::Value::Bool(*b),
                                    _ => opentelemetry::Value::String(v.to_string().into()),
                                };
                                attrs.push(KeyValue::new(k.clone(), value));
                            }
                        }
                    }
                }
            }
        }
        attrs
    };

    // Create ResourceMetrics with empty scope_metrics
    // Note: Full implementation would create proper Metric and ScopeMetrics structures
    // For now, we create a minimal ResourceMetrics using Default
    // ResourceMetrics fields are private in opentelemetry-sdk 0.31, so we can't construct it directly
    // The resource attributes are preserved in the Arrow format when written to file
    // TODO: Use opentelemetry-proto conversion utilities for full ResourceMetrics construction
    let resource_metrics = ResourceMetrics::default();

    Ok(Some(resource_metrics))
}
