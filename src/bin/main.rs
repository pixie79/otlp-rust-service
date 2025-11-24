//! Standalone OTLP Arrow Service
//!
//! Runs as a standalone service that receives OTLP messages via gRPC
//! (both Protobuf and Arrow Flight) and writes them to Arrow IPC files.

use otlp_arrow_library::config::ConfigLoader;
use otlp_arrow_library::dashboard::server::DashboardServer;
use otlp_arrow_library::otlp::{OtlpArrowFlightServer, OtlpGrpcServer};
use otlp_arrow_library::{Config, OtlpLibrary};
use std::net::SocketAddr;
use tokio::io::AsyncWriteExt;
use tracing::{error, info, warn};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .json()
        .init();

    // Load configuration from environment variables (with defaults)
    let config = ConfigLoader::from_env().unwrap_or_else(|e| {
        warn!(error = %e, "Failed to load configuration from environment, using defaults");
        Config::default()
    });

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

    // Start dashboard HTTP server if enabled
    let dashboard_handle = if config.dashboard.enabled {
        let dashboard_server = DashboardServer::new(
            config.dashboard.static_dir.clone(),
            config.dashboard.port,
            config.dashboard.bind_address.clone(),
        );

        match dashboard_server.start().await {
            Ok(handle) => {
                info!(
                    port = config.dashboard.port,
                    bind_address = %config.dashboard.bind_address,
                    static_dir = %config.dashboard.static_dir.display(),
                    "Dashboard HTTP server started"
                );
                Some(handle)
            }
            Err(e) => {
                error!(
                    error = %e,
                    "Failed to start dashboard HTTP server, continuing without dashboard"
                );
                None
            }
        }
    } else {
        info!("Dashboard disabled (default)");
        None
    };

    // Start health check endpoint (simple HTTP server on port 8081 to avoid conflict with dashboard)
    let health_port = if config.dashboard.enabled && config.dashboard.port == 8080 {
        8081
    } else {
        8080
    };

    let health_handle = tokio::spawn(async move {
        let addr: SocketAddr = format!("0.0.0.0:{}", health_port).parse().unwrap();
        let listener = match tokio::net::TcpListener::bind(&addr).await {
            Ok(l) => l,
            Err(e) => {
                error!(error = %e, "Failed to bind health check endpoint");
                return;
            }
        };
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
    if let Some(handle) = dashboard_handle {
        handle.abort();
        info!("Dashboard HTTP server stopped");
    }
    health_handle.abort();

    library.shutdown().await?;

    Ok(())
}
