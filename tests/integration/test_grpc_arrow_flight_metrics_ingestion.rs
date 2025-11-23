//! Integration test for gRPC Arrow Flight metrics ingestion

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

/// Helper to create a test metric in Arrow RecordBatch format
/// Uses the same schema as convert_metrics_to_arrow_ipc
fn create_test_arrow_metrics_batch() -> RecordBatch {
    use std::time::{SystemTime, UNIX_EPOCH};

    // Create Arrow schema matching convert_metrics_to_arrow_ipc
    let schema = Schema::new(vec![
        Field::new("metric_name", DataType::Utf8, false),
        Field::new("value", DataType::Float64, false),
        Field::new("timestamp_unix_nano", DataType::UInt64, false),
        Field::new("metric_type", DataType::Utf8, false),
        Field::new("attributes", DataType::Utf8, true),
    ]);

    // Build arrays with test metric data
    let metric_name = "test.counter";
    let value = 42.5;
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos() as u64;
    let metric_type = "counter";
    
    // Create attributes JSON
    let mut attrs_obj = serde_json::Map::new();
    attrs_obj.insert("service.name".to_string(), serde_json::Value::String("test-service".to_string()));
    attrs_obj.insert("environment".to_string(), serde_json::Value::String("test".to_string()));
    let attrs_json = serde_json::to_string(&attrs_obj).unwrap();
    
    let name_array = Arc::new(StringArray::from(vec![Some(metric_name)]));
    let value_array = Arc::new(Float64Array::from(vec![value]));
    let timestamp_array = Arc::new(UInt64Array::from(vec![timestamp]));
    let type_array = Arc::new(StringArray::from(vec![Some(metric_type)]));
    let attributes_array = Arc::new(StringArray::from(vec![Some(attrs_json)]));

    RecordBatch::try_new(
        Arc::new(schema),
        vec![
            name_array,
            value_array,
            timestamp_array,
            type_array,
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
async fn test_grpc_arrow_flight_metrics_ingestion() {
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
    
    // Use a different port from trace test to avoid conflicts
    let addr = "127.0.0.1:4319".parse().unwrap();
    
    let server_handle = tokio::spawn(async move {
        // Start the server - this will block until shutdown
        let _ = server.start(addr).await;
    });

    // Wait a bit for server to be ready
    sleep(Duration::from_millis(500)).await;

    // Create Arrow Flight client and send metrics
    let channel = Channel::from_shared(format!("http://{}", addr))
        .unwrap()
        .connect()
        .await
        .expect("Failed to connect to server");
    
    let mut client = FlightServiceClient::new(channel);

    // Create Arrow RecordBatch with metrics data
    let batch = create_test_arrow_metrics_batch();
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
    let metrics_dir = temp_dir.path().join("otlp/metrics");
    let files: Vec<_> = std::fs::read_dir(&metrics_dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .collect();
    
    assert!(!files.is_empty(), "Expected at least one metrics file to be created");

    // Cleanup
    server_handle.abort();
    library.shutdown().await.unwrap();
}

