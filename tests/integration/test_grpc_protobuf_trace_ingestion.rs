//! Integration test for gRPC Protobuf trace ingestion

use opentelemetry_proto::tonic::collector::trace::v1::{
    trace_service_client::TraceServiceClient, ExportTraceServiceRequest,
};
use opentelemetry_proto::tonic::trace::v1::{ResourceSpans, ScopeSpans, Span};
use opentelemetry_proto::tonic::common::v1::{AnyValue, KeyValue};
use otlp_arrow_library::{Config, OtlpLibrary};
use std::path::PathBuf;
use std::time::Duration;
use tempfile::TempDir;
use tokio::time::sleep;
use tonic::transport::Server;

/// Helper to create a simple test span in protobuf format
fn create_test_protobuf_span() -> ExportTraceServiceRequest {
    let span = Span {
        trace_id: vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16],
        span_id: vec![1, 2, 3, 4, 5, 6, 7, 8],
        parent_span_id: vec![9, 10, 11, 12, 13, 14, 15, 16],
        name: "test-span".to_string(),
        kind: 1, // Server
        start_time_unix_nano: 1000000000,
        end_time_unix_nano: 2000000000,
        attributes: vec![
            KeyValue {
                key: "service.name".to_string(),
                value: Some(AnyValue {
                    value: Some(opentelemetry_proto::tonic::common::v1::any_value::Value::StringValue("test-service".to_string())),
                }),
            },
            KeyValue {
                key: "http.method".to_string(),
                value: Some(AnyValue {
                    value: Some(opentelemetry_proto::tonic::common::v1::any_value::Value::StringValue("GET".to_string())),
                }),
            },
        ],
        dropped_attributes_count: 0,
        events: vec![],
        dropped_events_count: 0,
        links: vec![],
        dropped_links_count: 0,
        status: None,
    };

    let scope_spans = ScopeSpans {
        scope: None,
        spans: vec![span],
        schema_url: "".to_string(),
    };

    let resource_spans = ResourceSpans {
        resource: None,
        scope_spans: vec![scope_spans],
        schema_url: "".to_string(),
    };

    ExportTraceServiceRequest {
        resource_spans: vec![resource_spans],
    }
}

#[tokio::test]
async fn test_grpc_protobuf_trace_ingestion() {
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
    
    // Start gRPC server in background
    let file_exporter = library.file_exporter.clone();
    let server = crate::otlp::server::OtlpGrpcServer::new(file_exporter);
    
    let addr = "127.0.0.1:0".parse().unwrap();
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    let addr = listener.local_addr().unwrap();
    
    let server_handle = tokio::spawn(async move {
        use tokio_stream::wrappers::TcpListenerStream;
        let trace_service = crate::otlp::server::TraceServiceImpl {
            file_exporter: server.file_exporter.clone(),
        };
        let metrics_service = crate::otlp::server::MetricsServiceImpl {
            file_exporter: server.file_exporter.clone(),
        };
        
        Server::builder()
            .add_service(opentelemetry_proto::tonic::collector::trace::v1::trace_service_server::TraceServiceServer::new(trace_service))
            .add_service(opentelemetry_proto::tonic::collector::metrics::v1::metrics_service_server::MetricsServiceServer::new(metrics_service))
            .serve_with_incoming(TcpListenerStream::new(listener))
            .await
            .unwrap();
    });

    // Wait for server to start
    sleep(Duration::from_millis(100)).await;

    // Create gRPC client and send trace
    let mut client = TraceServiceClient::connect(format!("http://{}", addr))
        .await
        .expect("Failed to connect to server");

    let request = create_test_protobuf_span();
    let response = client.export(request).await.expect("Failed to export trace");
    
    assert!(response.get_ref().partial_success.is_none());

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

