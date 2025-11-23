//! Standalone OTLP Arrow Service
//!
//! Runs as a standalone service that receives OTLP messages via gRPC
//! (both Protobuf and Arrow Flight) and writes them to Arrow IPC files.

use otlp_arrow_library::otlp::{OtlpArrowFlightServer, OtlpGrpcServer};
use otlp_arrow_library::{Config, OtlpLibrary};
use std::net::SocketAddr;
use tokio::io::AsyncWriteExt;
use tracing::{error, info};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .json()
        .init();

    // Load configuration (for now, use defaults)
    let config = Config::default();

    // Create library instance
    let library = OtlpLibrary::new(config.clone()).await?;

    let file_exporter = library.file_exporter();

    // Start gRPC Protobuf server if enabled
    let protobuf_handle = if config.protocols.protobuf_enabled {
        let protobuf_addr: SocketAddr = format!("0.0.0.0:{}", config.protocols.protobuf_port)
            .parse()
            .map_err(|e| {
                format!(
                    "Invalid Protobuf port {}: {}",
                    config.protocols.protobuf_port, e
                )
            })?;

        let protobuf_server = OtlpGrpcServer::new(file_exporter.clone());

        info!("Starting gRPC Protobuf server on {}", protobuf_addr);
        Some(tokio::spawn(async move {
            if let Err(e) = protobuf_server.start(protobuf_addr).await {
                error!("gRPC Protobuf server error: {}", e);
            }
        }))
    } else {
        info!("gRPC Protobuf server disabled");
        None
    };

    // Start gRPC Arrow Flight server if enabled
    let arrow_flight_handle = if config.protocols.arrow_flight_enabled {
        let arrow_flight_addr: SocketAddr =
            format!("0.0.0.0:{}", config.protocols.arrow_flight_port)
                .parse()
                .map_err(|e| {
                    format!(
                        "Invalid Arrow Flight port {}: {}",
                        config.protocols.arrow_flight_port, e
                    )
                })?;

        let arrow_flight_server = OtlpArrowFlightServer::new(file_exporter.clone());

        info!("Starting gRPC Arrow Flight server on {}", arrow_flight_addr);
        Some(tokio::spawn(async move {
            if let Err(e) = arrow_flight_server.start(arrow_flight_addr).await {
                error!("gRPC Arrow Flight server error: {}", e);
            }
        }))
    } else {
        info!("gRPC Arrow Flight server disabled");
        None
    };

    info!("OTLP Arrow Service started");
    if config.protocols.protobuf_enabled {
        info!(
            "  - gRPC Protobuf: listening on port {}",
            config.protocols.protobuf_port
        );
    }
    if config.protocols.arrow_flight_enabled {
        info!(
            "  - gRPC Arrow Flight: listening on port {}",
            config.protocols.arrow_flight_port
        );
    }
    info!("Listening for OTLP messages...");

    // Start health check endpoint (simple HTTP server on port 8080)
    let health_handle = tokio::spawn(async move {
        use std::net::SocketAddr;
        let addr: SocketAddr = "0.0.0.0:8080".parse().unwrap();
        let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
        info!("Health check endpoint listening on {}", addr);

        loop {
            match listener.accept().await {
                Ok((mut stream, _)) => {
                    let response = b"HTTP/1.1 200 OK\r\nContent-Length: 2\r\n\r\nOK";
                    let _ = stream.write_all(response).await;
                    let _ = stream.shutdown().await;
                }
                Err(e) => {
                    error!("Health check endpoint error: {}", e);
                }
            }
        }
    });

    // Keep the service running
    tokio::signal::ctrl_c().await?;

    info!("Shutting down...");

    // Abort server tasks
    if let Some(handle) = protobuf_handle {
        handle.abort();
    }
    if let Some(handle) = arrow_flight_handle {
        handle.abort();
    }
    health_handle.abort();

    library.shutdown().await?;

    Ok(())
}
