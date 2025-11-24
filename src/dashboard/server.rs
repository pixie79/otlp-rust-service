//! HTTP server for serving dashboard static files
//!
//! Serves static files from a configured directory via HTTP.

use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tracing::{error, info, warn};

/// HTTP server for serving dashboard static files
pub struct DashboardServer {
    static_dir: PathBuf,
    port: u16,
}

impl DashboardServer {
    /// Create a new dashboard server
    pub fn new(static_dir: impl Into<PathBuf>, port: u16) -> Self {
        Self {
            static_dir: static_dir.into(),
            port,
        }
    }

    /// Start the HTTP server
    ///
    /// Returns a handle that can be used to shutdown the server
    pub async fn start(&self) -> Result<tokio::task::JoinHandle<()>, std::io::Error> {
        let addr = format!("0.0.0.0:{}", self.port);
        let listener = TcpListener::bind(&addr).await?;

        info!(
            port = self.port,
            static_dir = %self.static_dir.display(),
            "Dashboard HTTP server started"
        );
        info!("Dashboard available at http://localhost:{}", self.port);

        let static_dir = Arc::new(self.static_dir.clone());

        let handle = tokio::spawn(async move {
            loop {
                match listener.accept().await {
                    Ok((stream, addr)) => {
                        let static_dir = static_dir.clone();
                        tokio::spawn(async move {
                            if let Err(e) = Self::handle_request(stream, static_dir.as_path()).await
                            {
                                warn!(client = %addr, error = %e, "Error handling dashboard request");
                            }
                        });
                    }
                    Err(e) => {
                        error!(error = %e, "Failed to accept dashboard connection");
                    }
                }
            }
        });

        Ok(handle)
    }

    /// Handle an HTTP request
    async fn handle_request(
        mut stream: TcpStream,
        static_dir: &Path,
    ) -> Result<(), std::io::Error> {
        let mut buffer = [0; 8192];
        let n = stream.read(&mut buffer).await?;

        if n == 0 {
            return Ok(());
        }

        let request = String::from_utf8_lossy(&buffer[..n]);
        let request_lines: Vec<&str> = request.lines().collect();

        if request_lines.is_empty() {
            return Self::send_response(&mut stream, 400, "Bad Request", b"", None).await;
        }

        // Parse request line (e.g., "GET /index.html HTTP/1.1")
        let request_line = request_lines[0];
        let parts: Vec<&str> = request_line.split_whitespace().collect();

        if parts.len() < 2 {
            return Self::send_response(&mut stream, 400, "Bad Request", b"", None).await;
        }

        let method = parts[0];
        let path = parts[1];

        // Only support GET requests
        if method != "GET" {
            return Self::send_response(&mut stream, 405, "Method Not Allowed", b"", None).await;
        }

        // Normalize path
        let file_path = if path == "/" {
            "index.html"
        } else {
            // Remove leading slash and decode URL
            path.trim_start_matches('/')
        };

        // Security: prevent directory traversal
        let file_path = PathBuf::from(file_path);
        if file_path
            .components()
            .any(|c| matches!(c, std::path::Component::ParentDir))
        {
            return Self::send_response(&mut stream, 403, "Forbidden", b"", None).await;
        }

        // Resolve file path
        let full_path = static_dir.join(&file_path);

        // Check if file exists
        if !full_path.exists() {
            return Self::send_response(&mut stream, 404, "Not Found", b"", None).await;
        }

        // Read file
        match tokio::fs::read(&full_path).await {
            Ok(content) => {
                let content_type = Self::get_content_type(&file_path);
                Self::send_response(&mut stream, 200, "OK", &content, Some(content_type)).await
            }
            Err(e) => {
                error!(file = %full_path.display(), error = %e, "Failed to read dashboard file");
                Self::send_response(&mut stream, 500, "Internal Server Error", b"", None).await
            }
        }
    }

    /// Send HTTP response
    async fn send_response(
        stream: &mut TcpStream,
        status_code: u16,
        status_text: &str,
        body: &[u8],
        content_type: Option<&str>,
    ) -> Result<(), std::io::Error> {
        let content_type_header = content_type
            .map(|ct| format!("Content-Type: {}\r\n", ct))
            .unwrap_or_default();

        let response = format!(
            "HTTP/1.1 {} {}\r\n\
             Content-Length: {}\r\n\
             {}\
             \r\n",
            status_code,
            status_text,
            body.len(),
            content_type_header
        );

        stream.write_all(response.as_bytes()).await?;
        stream.write_all(body).await?;
        stream.flush().await?;

        Ok(())
    }

    /// Determine content type from file extension
    fn get_content_type(path: &Path) -> &'static str {
        match path.extension().and_then(|ext| ext.to_str()) {
            Some("html") => "text/html; charset=utf-8",
            Some("js") => "application/javascript; charset=utf-8",
            Some("css") => "text/css; charset=utf-8",
            Some("json") => "application/json; charset=utf-8",
            Some("png") => "image/png",
            Some("jpg") | Some("jpeg") => "image/jpeg",
            Some("svg") => "image/svg+xml",
            Some("ico") => "image/x-icon",
            Some("woff") => "font/woff",
            Some("woff2") => "font/woff2",
            Some("ttf") => "font/ttf",
            Some("map") => "application/json",
            _ => "application/octet-stream",
        }
    }
}
