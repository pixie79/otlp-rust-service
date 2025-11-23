//! Integration test for mock service gRPC Arrow Flight interface

use arrow::array::*;
use arrow::datatypes::*;
use arrow::ipc::writer::StreamWriter;
use arrow::record_batch::RecordBatch;
use arrow_flight::flight_service_client::FlightServiceClient;
use arrow_flight::FlightData;
use otlp_arrow_library::MockOtlpService;
use std::io::Cursor;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;

/// Helper to create a test trace in Arrow RecordBatch format
fn create_test_arrow_trace_batch() -> RecordBatch {
    use std::time::{SystemTime, UNIX_EPOCH};

    let schema = Schema::new(vec![
        Field::new("trace_id", DataType::Binary, false),
        Field::new("span_id", DataType::Binary, false),
        Field::new("parent_span_id", DataType::Binary, true),
        Field::new("name", DataType::Utf8, false),
        Field::new("kind", DataType::Int32, false),
        Field::new("start_time_unix_nano", DataType::UInt64, false),
        Field::new("end_time_unix_nano", DataType::UInt64, false),
        Field::new("status_code", DataType::Utf8, false),
        Field::new("status_message", DataType::Utf8, true),
        Field::new("attributes", DataType::Utf8, true),
    ]);

    let trace_id = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16];
    let span_id = vec![1, 2, 3, 4, 5, 6, 7, 8];
    let parent_span_id = vec![9, 10, 11, 12, 13, 14, 15, 16];
    let name = "test-span";
    let kind = 1i32; // Server
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos() as u64;
    let status_code = "OK";
    let status_message = None::<String>;
    
    // Create attributes JSON
    let mut attrs_obj = serde_json::Map::new();
    attrs_obj.insert("service.name".to_string(), serde_json::Value::String("test-service".to_string()));
    let attrs_json = serde_json::to_string(&attrs_obj).unwrap();
    
    let trace_id_array = Arc::new(BinaryArray::from(vec![Some(trace_id)]));
    let span_id_array = Arc::new(BinaryArray::from(vec![Some(span_id)]));
    let parent_span_id_array = Arc::new(BinaryArray::from(vec![Some(parent_span_id)]));
    let name_array = Arc::new(StringArray::from(vec![Some(name)]));
    let kind_array = Arc::new(Int32Array::from(vec![kind]));
    let start_time_array = Arc::new(UInt64Array::from(vec![timestamp]));
    let end_time_array = Arc::new(UInt64Array::from(vec![timestamp + 1000000000]));
    let status_code_array = Arc::new(StringArray::from(vec![Some(status_code)]));
    let status_message_array = Arc::new(StringArray::from(vec![status_message]));
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
async fn test_mock_service_grpc_arrow_flight_trace() {
    let service = MockOtlpService::new();
    
    // Start the mock service
    let addresses = service.start().await.expect("Failed to start mock service");
    
    // Wait for server to start
    sleep(Duration::from_millis(200)).await;
    
    // Create Arrow Flight client
    let mut client = FlightServiceClient::connect(addresses.arrow_flight_addr.clone())
        .await
        .expect("Failed to connect to mock service");
    
    // Create test batch
    let batch = create_test_arrow_trace_batch();
    let flight_data = record_batch_to_flight_data(&batch)
        .expect("Failed to convert batch to FlightData");
    
    // Send via DoPut
    let mut stream = client
        .do_put(tonic::Request::new(futures::stream::once(async { Ok(flight_data) })))
        .await
        .expect("Failed to call do_put")
        .into_inner();
    
    // Consume the response stream
    while let Some(_) = stream.next().await.transpose().expect("Stream error") {}
    
    // Wait for processing
    sleep(Duration::from_millis(200)).await;
    
    // Verify trace was received
    assert!(service.assert_traces_received(1).await.is_ok());
    assert_eq!(service.grpc_calls_count().await, 1);
    assert_eq!(service.api_calls_count().await, 0);
}

