//! Standalone service example
//!
//! This example demonstrates how to run the OTLP library as a standalone service
//! with gRPC servers for both Protobuf and Arrow Flight protocols.

use otlp_arrow_library::otlp::{OtlpArrowFlightServer, OtlpGrpcServer};
use otlp_arrow_library::{Config, OtlpLibrary};
use std::path::PathBuf;
use tokio::signal;
use tracing::{info, warn};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    otlp_arrow_library::init_logging();

    // Load configuration
    // In production, you would load from YAML file or environment variables
    let config = Config {
        output_dir: PathBuf::from("./output_dir"),
        write_interval_secs: 5,
        trace_cleanup_interval_secs: 600,
        metric_cleanup_interval_secs: 3600,
        protocols: Default::default(), // Both Protobuf and Arrow Flight enabled
        forwarding: None,              // No forwarding by default
        dashboard: Default::default(), // Dashboard disabled by default
    };

    // Create library instance (clone config since we need it later)
    let config_clone = config.clone();
    let library = OtlpLibrary::new(config).await?;
    let file_exporter = library.file_exporter();

    // Start gRPC Protobuf server if enabled
    let protobuf_handle = if config_clone.protocols.protobuf_enabled {
        let file_exporter_clone = file_exporter.clone();
        let addr = format!("0.0.0.0:{}", config_clone.protocols.protobuf_port)
            .parse()
            .unwrap();

        info!("Starting gRPC Protobuf server on {}", addr);

        Some(tokio::spawn(async move {
            let server = OtlpGrpcServer::new(file_exporter_clone);
            if let Err(e) = server.start(addr).await {
                warn!("Protobuf server error: {}", e);
            }
        }))
    } else {
        None
    };

    // Start gRPC Arrow Flight server if enabled
    let arrow_flight_handle = if config_clone.protocols.arrow_flight_enabled {
        let file_exporter_clone = file_exporter.clone();
        let addr = format!("0.0.0.0:{}", config_clone.protocols.arrow_flight_port)
            .parse()
            .unwrap();

        info!("Starting gRPC Arrow Flight server on {}", addr);

        Some(tokio::spawn(async move {
            let server = OtlpArrowFlightServer::new(file_exporter_clone);
            if let Err(e) = server.start(addr).await {
                warn!("Arrow Flight server error: {}", e);
            }
        }))
    } else {
        None
    };

    info!("OTLP service started. Press Ctrl+C to shutdown.");

    // Wait for shutdown signal
    signal::ctrl_c().await?;
    info!("Shutdown signal received, shutting down gracefully...");

    // Shutdown library
    library.shutdown().await?;

    // Stop servers
    if let Some(handle) = protobuf_handle {
        handle.abort();
    }
    if let Some(handle) = arrow_flight_handle {
        handle.abort();
    }

    info!("Shutdown complete");
    Ok(())
}
