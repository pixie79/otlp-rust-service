//! HTTP server for serving dashboard static files
//!
//! Serves static files from a configured directory via HTTP.

use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tracing::{error, info, warn};

/// HTTP server for serving dashboard static files and Arrow IPC files
pub struct DashboardServer {
    static_dir: PathBuf,
    output_dir: PathBuf,
    port: u16,
    bind_address: String,
}

impl DashboardServer {
    /// Create a new dashboard server
    pub fn new(
        static_dir: impl Into<PathBuf>,
        output_dir: impl Into<PathBuf>,
        port: u16,
        bind_address: impl Into<String>,
    ) -> Self {
        Self {
            static_dir: static_dir.into(),
            output_dir: output_dir.into(),
            port,
            bind_address: bind_address.into(),
        }
    }

    /// Start the HTTP server
    ///
    /// Returns a handle that can be used to shutdown the server
    ///
    /// Binds to the configured bind_address (default: 127.0.0.1 for local-only access).
    /// Use 0.0.0.0 to allow network access from other machines.
    pub async fn start(&self) -> Result<tokio::task::JoinHandle<()>, std::io::Error> {
        let addr = format!("{}:{}", self.bind_address, self.port);
        let listener = TcpListener::bind(&addr).await?;

        info!(
            port = self.port,
            bind_address = %self.bind_address,
            static_dir = %self.static_dir.display(),
            output_dir = %self.output_dir.display(),
            "Dashboard HTTP server started"
        );
        info!(
            "Dashboard available at http://{}:{}",
            self.bind_address, self.port
        );

        let static_dir = Arc::new(self.static_dir.clone());
        let output_dir = Arc::new(self.output_dir.clone());

        let handle = tokio::spawn(async move {
            loop {
                match listener.accept().await {
                    Ok((stream, addr)) => {
                        let static_dir = static_dir.clone();
                        let output_dir = output_dir.clone();
                        tokio::spawn(async move {
                            if let Err(e) = Self::handle_request(
                                stream,
                                static_dir.as_path(),
                                output_dir.as_path(),
                            )
                            .await
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
        output_dir: &Path,
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

        // Handle Arrow IPC files from output_dir
        if path.starts_with("/data/") {
            // Remove /data/ prefix and decode URL
            let relative_path = path.trim_start_matches("/data/");

            // Security: comprehensive path validation
            match Self::validate_and_normalize_path(relative_path) {
                Ok(normalized_path) => {
                    // Resolve file path in output_dir
                    let full_path = output_dir.join(&normalized_path);

                    // Additional security: canonicalize to resolve symlinks and ensure path stays within output_dir
                    match full_path.canonicalize() {
                        Ok(canonical_path) => {
                            // Verify canonical path is still within output_dir
                            if !canonical_path.starts_with(
                                output_dir
                                    .canonicalize()
                                    .unwrap_or_else(|_| output_dir.to_path_buf()),
                            ) {
                                warn!("Path traversal attempt detected: {}", relative_path);
                                return Self::send_response(
                                    &mut stream,
                                    403,
                                    "Forbidden",
                                    b"",
                                    None,
                                )
                                .await;
                            }
                            // Use canonical path for file operations
                            let full_path = canonical_path;

                            // Check if file exists
                            if !full_path.exists() {
                                return Self::send_response(
                                    &mut stream,
                                    404,
                                    "Not Found",
                                    b"",
                                    None,
                                )
                                .await;
                            }

                            // Read and serve Arrow file
                            match tokio::fs::read(&full_path).await {
                                Ok(content) => {
                                    // Set content type for Arrow files
                                    let content_type =
                                        if normalized_path.extension().and_then(|e| e.to_str())
                                            == Some("arrows")
                                        {
                                            "application/vnd.apache.arrow.stream"
                                        } else {
                                            "application/octet-stream"
                                        };
                                    return Self::send_response(
                                        &mut stream,
                                        200,
                                        "OK",
                                        &content,
                                        Some(content_type),
                                    )
                                    .await;
                                }
                                Err(e) => {
                                    error!(file = %full_path.display(), error = %e, "Failed to read Arrow file");
                                    return Self::send_response(
                                        &mut stream,
                                        500,
                                        "Internal Server Error",
                                        b"",
                                        None,
                                    )
                                    .await;
                                }
                            }
                        }
                        Err(_) => {
                            // canonicalize failed (path doesn't exist or permission denied)
                            // Fall back to non-canonical path but log warning
                            warn!("Failed to canonicalize path: {}", relative_path);
                            let full_path = output_dir.join(&normalized_path);
                            if !full_path.exists() {
                                return Self::send_response(
                                    &mut stream,
                                    404,
                                    "Not Found",
                                    b"",
                                    None,
                                )
                                .await;
                            }
                            match tokio::fs::read(&full_path).await {
                                Ok(content) => {
                                    let content_type =
                                        if normalized_path.extension().and_then(|e| e.to_str())
                                            == Some("arrows")
                                        {
                                            "application/vnd.apache.arrow.stream"
                                        } else {
                                            "application/octet-stream"
                                        };
                                    return Self::send_response(
                                        &mut stream,
                                        200,
                                        "OK",
                                        &content,
                                        Some(content_type),
                                    )
                                    .await;
                                }
                                Err(e) => {
                                    error!(file = %full_path.display(), error = %e, "Failed to read Arrow file");
                                    return Self::send_response(
                                        &mut stream,
                                        500,
                                        "Internal Server Error",
                                        b"",
                                        None,
                                    )
                                    .await;
                                }
                            }
                        }
                    }
                }
                Err(_) => {
                    // Path validation failed
                    warn!("Path validation failed: {}", relative_path);
                    return Self::send_response(&mut stream, 403, "Forbidden", b"", None).await;
                }
            }
        }

        // Handle static dashboard files
        // Normalize path
        let relative_path = if path == "/" {
            "index.html"
        } else {
            // Remove leading slash and decode URL
            path.trim_start_matches('/')
        };

        // Security: comprehensive path validation
        let file_path = match Self::validate_and_normalize_path(relative_path) {
            Ok(normalized_path) => normalized_path,
            Err(_) => {
                warn!("Path validation failed: {}", relative_path);
                return Self::send_response(&mut stream, 403, "Forbidden", b"", None).await;
            }
        };

        // Resolve file path
        let full_path = static_dir.join(&file_path);

        // Additional security: canonicalize to resolve symlinks
        let full_path = match full_path.canonicalize() {
            Ok(canonical_path) => {
                // Verify canonical path is still within static_dir
                if let Ok(canonical_static_dir) = static_dir.canonicalize()
                    && !canonical_path.starts_with(&canonical_static_dir)
                {
                    warn!("Path traversal attempt detected: {}", relative_path);
                    return Self::send_response(&mut stream, 403, "Forbidden", b"", None).await;
                }
                canonical_path
            }
            Err(_) => {
                // canonicalize failed, use original path
                full_path
            }
        };

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

        // Security headers - always include for defense-in-depth
        // Use default "DENY" for X-Frame-Options (can be enhanced later to use config)
        let security_headers = Self::get_security_headers(None);

        let response = format!(
            "HTTP/1.1 {} {}\r\n\
             Content-Length: {}\r\n\
             {}\
             {}\
             \r\n",
            status_code,
            status_text,
            body.len(),
            content_type_header,
            security_headers
        );

        stream.write_all(response.as_bytes()).await?;
        stream.write_all(body).await?;
        stream.flush().await?;

        Ok(())
    }

    /// Get security headers for HTTP responses
    ///
    /// Returns standard security headers to protect against common web vulnerabilities:
    /// - Content-Security-Policy: Prevents XSS attacks
    /// - X-Frame-Options: Prevents clickjacking (configurable via DashboardConfig)
    /// - X-Content-Type-Options: Prevents MIME type sniffing
    /// - X-XSS-Protection: Additional XSS protection (legacy browsers)
    ///
    /// Note: x_frame_options parameter allows customization, defaults to "DENY" if None
    fn get_security_headers(x_frame_options: Option<&str>) -> String {
        let xfo = x_frame_options.unwrap_or("DENY");
        format!(
            "Content-Security-Policy: default-src 'self'\r\n\
             X-Frame-Options: {}\r\n\
             X-Content-Type-Options: nosniff\r\n\
             X-XSS-Protection: 1; mode=block\r\n",
            xfo
        )
    }

    /// Validate and normalize a file path to prevent directory traversal attacks
    ///
    /// This function:
    /// - Rejects absolute paths
    /// - Rejects paths with parent directory components (`..`)
    /// - Rejects Windows UNC paths (`\\server\share`)
    /// - Normalizes path separators and removes redundant components
    /// - Returns the normalized path if valid, or an error if invalid
    fn validate_and_normalize_path(path: &str) -> Result<PathBuf, ()> {
        // Reject empty paths
        if path.is_empty() {
            return Err(());
        }

        // Reject paths starting with Windows UNC prefix
        if path.starts_with("\\\\") || path.starts_with("//") {
            return Err(());
        }

        // Create PathBuf and check components
        let path_buf = PathBuf::from(path);

        // Reject absolute paths (Unix: starts with `/`, Windows: starts with drive letter or `\`)
        if path_buf.is_absolute() {
            return Err(());
        }

        // Check for parent directory components
        if path_buf
            .components()
            .any(|c| matches!(c, std::path::Component::ParentDir))
        {
            return Err(());
        }

        // Normalize the path (remove `.` components, normalize separators)
        let mut normalized = PathBuf::new();
        for component in path_buf.components() {
            match component {
                std::path::Component::Prefix(_) | std::path::Component::RootDir => {
                    // Should not occur for relative paths, but reject if present
                    return Err(());
                }
                std::path::Component::CurDir => {
                    // Skip `.` components (current directory)
                    continue;
                }
                std::path::Component::ParentDir => {
                    // Already checked above, but double-check
                    return Err(());
                }
                std::path::Component::Normal(name) => {
                    normalized.push(name);
                }
            }
        }

        Ok(normalized)
    }

    /// Determine content type from file extension
    fn get_content_type(path: &Path) -> &'static str {
        match path.extension().and_then(|ext| ext.to_str()) {
            Some("html") => "text/html; charset=utf-8",
            Some("js") => "application/javascript; charset=utf-8",
            Some("css") => "text/css; charset=utf-8",
            Some("json") => "application/json; charset=utf-8",
            Some("wasm") => "application/wasm", // Required for WebAssembly compilation and extensions
            Some("png") => "image/png",
            Some("jpg") | Some("jpeg") => "image/jpeg",
            Some("svg") => "image/svg+xml",
            Some("ico") => "image/x-icon",
            Some("woff") => "font/woff",
            Some("woff2") => "font/woff2",
            Some("ttf") => "font/ttf",
            Some("map") => "application/json",
            Some("arrows") => "application/vnd.apache.arrow.stream",
            _ => "application/octet-stream",
        }
    }
}
