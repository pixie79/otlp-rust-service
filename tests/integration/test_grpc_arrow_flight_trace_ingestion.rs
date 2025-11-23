//! Integration test for gRPC Arrow Flight trace ingestion

use arrow::array::*;
use arrow::datatypes::*;
use arrow::ipc::writer::StreamWriter;
use arrow::record_batch::RecordBatch;
use arrow_flight::flight_service_client::FlightServiceClient;
use arrow_flight::FlightData;
use otlp_arrow_library::{Config, OtlpLibrary};
use std::io::Cursor;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tempfile::TempDir;
use tokio::time::sleep;
use tonic::transport::Channel;

/// Helper to create a test span in Arrow RecordBatch format
/// Uses the same schema as convert_spans_to_arrow_ipc
fn create_test_arrow_trace_batch() -> RecordBatch {
    use opentelemetry::trace::{TraceId, SpanId};
    use std::time::UNIX_EPOCH;

    // Create a test span
    let trace_id = TraceId::from_bytes([1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16]);
    let span_id = SpanId::from_bytes([1, 2, 3, 4, 5, 6, 7, 8]);
    let parent_span_id = SpanId::from_bytes([9, 10, 11, 12, 13, 14, 15, 16]);
    
    // Create Arrow schema matching convert_spans_to_arrow_ipc
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
        Field::new("attributes", DataType::Utf8, true),
    ]);

    // Build arrays
    let trace_id_bytes = trace_id.to_bytes().to_vec();
    let span_id_bytes = span_id.to_bytes().to_vec();
    let parent_span_id_bytes = parent_span_id.to_bytes().to_vec();
    
    let trace_id_array = Arc::new(BinaryArray::from(vec![Some(trace_id_bytes.as_slice())]));
    let span_id_array = Arc::new(BinaryArray::from(vec![Some(span_id_bytes.as_slice())]));
    let parent_span_id_array = Arc::new(BinaryArray::from(vec![Some(parent_span_id_bytes.as_slice())]));
    let name_array = Arc::new(StringArray::from(vec![Some("test-span")]));
    let kind_array = Arc::new(Int32Array::from(vec![1i32])); // Server
    let start_time_array = Arc::new(UInt64Array::from(vec![1000000000u64]));
    let end_time_array = Arc::new(UInt64Array::from(vec![2000000000u64]));
    let status_code_array = Arc::new(Int32Array::from(vec![1i32])); // Ok
    let status_message_array = Arc::new(StringArray::from(vec![None::<String>]));
    
    // Create attributes JSON
    let mut attrs_obj = serde_json::Map::new();
    attrs_obj.insert("service.name".to_string(), serde_json::Value::String("test-service".to_string()));
    attrs_obj.insert("http.method".to_string(), serde_json::Value::String("GET".to_string()));
    let attrs_json = serde_json::to_string(&attrs_obj).unwrap();
    let attributes_array = Arc::new(StringArray::from(vec![Some(attrs_json)]));

    RecordBatch::try_new(
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
    )
    .unwrap()
}

/// Convert RecordBatch to Arrow Flight FlightData
fn record_batch_to_flight_data(batch: &RecordBatch) -> Result<FlightData, Box<dyn std::error::Error>> {
    let mut buffer = Vec::new();
    let cursor = Cursor::new(&mut buffer);
    let mut writer = StreamWriter::try_new(cursor, batch.schema().as_ref())?;
    writer.write(batch)?;
    writer.finish()?;
    
    Ok(FlightData {
        data_header: buffer,
        flight_descriptor: None,
        app_metadata: vec![],
    })
}

#[tokio::test]
async fn test_grpc_arrow_flight_trace_ingestion() {
    // Create a temporary directory for testing
    let temp_dir = TempDir::new().unwrap();
    
    let config = Config {
        output_dir: PathBuf::from(temp_dir.path()),
        write_interval_secs: 1, // Short interval for testing
        trace_cleanup_interval_secs: 600,
        metric_cleanup_interval_secs: 3600,
        protocols: Default::default(),
        forwarding: None,
    };

    // Create library instance
    let library = OtlpLibrary::new(config.clone()).await.unwrap();
    
    // Start Arrow Flight server in background
    let file_exporter = library.file_exporter();
    let server = otlp_arrow_library::otlp::OtlpArrowFlightServer::new(file_exporter);
    
    // Use a fixed port for testing (in production, use 0 to get a random port)
    let addr = "127.0.0.1:4318".parse().unwrap();
    
    let server_handle = tokio::spawn(async move {
        // Start the server - this will block until shutdown
        let _ = server.start(addr).await;
    });

    // Wait a bit for server to be ready
    sleep(Duration::from_millis(500)).await;

    // Create Arrow Flight client and send trace
    let channel = Channel::from_shared(format!("http://{}", addr))
        .unwrap()
        .connect()
        .await
        .expect("Failed to connect to server");
    
    let mut client = FlightServiceClient::new(channel);

    // Create Arrow RecordBatch with trace data
    let batch = create_test_arrow_trace_batch();
    let flight_data = record_batch_to_flight_data(&batch).unwrap();
    
    // Send via DoPut
    let request = tonic::Request::new(
        futures::stream::iter(vec![Ok(flight_data)])
    );
    let mut stream = client.do_put(request).await.expect("Failed to call do_put");
    
    // Consume the response stream
    while let Some(result) = stream.get_mut().message().await {
        match result {
            Ok(_) => {
                // Process response messages - empty stream is expected
            }
            Err(_) => {
                // Stream ended or error - that's OK for DoPut
                break;
            }
        }
    }
    
    // Wait for batch write
    sleep(Duration::from_secs(2)).await;
    
    // Flush to ensure all writes are complete
    library.flush().await.expect("Failed to flush");

    // Verify file was created
    let traces_dir = temp_dir.path().join("otlp/traces");
    let files: Vec<_> = std::fs::read_dir(&traces_dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .collect();
    
    assert!(!files.is_empty(), "Expected at least one trace file to be created");

    // Cleanup
    server_handle.abort();
    library.shutdown().await.unwrap();
}
