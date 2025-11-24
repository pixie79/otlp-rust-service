use otlp_arrow_library::config::ConfigLoader;
use otlp_arrow_library::dashboard::server::DashboardServer;
use std::fs;
use std::io::Read;
use std::path::PathBuf;
use tempfile::TempDir;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

fn find_available_port() -> u16 {
    use std::net::TcpListener;
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    listener.local_addr().unwrap().port()
}

#[tokio::test]
async fn test_dashboard_server_starts() {
    let temp_dir = tempfile::tempdir().unwrap();
    let static_dir = temp_dir.path().join("dashboard").join("dist");
    fs::create_dir_all(&static_dir).unwrap();

    let port = find_available_port();
    let server = DashboardServer::new(static_dir, port);

    let handle = server.start().await.unwrap();

    // Give server a moment to start
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // Try to connect
    let addr = format!("127.0.0.1:{}", port);
    let mut stream = TcpStream::connect(&addr).await.unwrap();

    // Send a simple GET request
    let request = "GET / HTTP/1.1\r\nHost: localhost\r\n\r\n";
    stream.write_all(request.as_bytes()).await.unwrap();

    // Read response
    let mut buffer = [0; 1024];
    let n = stream.read(&mut buffer).await.unwrap();
    let response = String::from_utf8_lossy(&buffer[..n]);

    // Should get 404 since index.html doesn't exist, but server is responding
    assert!(response.contains("HTTP/1.1"));

    handle.abort();
}

#[tokio::test]
async fn test_dashboard_server_serves_index_html() {
    let temp_dir = tempfile::tempdir().unwrap();
    let static_dir = temp_dir.path().join("dashboard").join("dist");
    fs::create_dir_all(&static_dir).unwrap();

    // Create index.html
    let index_content = "<html><body>Test Dashboard</body></html>";
    fs::write(static_dir.join("index.html"), index_content).unwrap();

    let port = find_available_port();
    let server = DashboardServer::new(static_dir, port);

    let handle = server.start().await.unwrap();

    // Give server a moment to start
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // Connect and request root
    let addr = format!("127.0.0.1:{}", port);
    let mut stream = TcpStream::connect(&addr).await.unwrap();

    let request = "GET / HTTP/1.1\r\nHost: localhost\r\n\r\n";
    stream.write_all(request.as_bytes()).await.unwrap();

    // Read response
    let mut buffer = [0; 4096];
    let n = stream.read(&mut buffer).await.unwrap();
    let response = String::from_utf8_lossy(&buffer[..n]);

    // Should get 200 OK with index.html content
    assert!(response.contains("HTTP/1.1 200 OK"));
    assert!(response.contains("Test Dashboard"));
    assert!(response.contains("text/html"));

    handle.abort();
}

#[tokio::test]
async fn test_dashboard_server_serves_static_files() {
    let temp_dir = tempfile::tempdir().unwrap();
    let static_dir = temp_dir.path().join("dashboard").join("dist");
    fs::create_dir_all(&static_dir).unwrap();

    // Create test files
    fs::write(static_dir.join("test.js"), "console.log('test');").unwrap();
    fs::write(static_dir.join("test.css"), "body { color: red; }").unwrap();

    let port = find_available_port();
    let server = DashboardServer::new(static_dir, port);

    let handle = server.start().await.unwrap();

    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // Test JavaScript file
    let addr = format!("127.0.0.1:{}", port);
    let mut stream = TcpStream::connect(&addr).await.unwrap();

    let request = "GET /test.js HTTP/1.1\r\nHost: localhost\r\n\r\n";
    stream.write_all(request.as_bytes()).await.unwrap();

    let mut buffer = [0; 4096];
    let n = stream.read(&mut buffer).await.unwrap();
    let response = String::from_utf8_lossy(&buffer[..n]);

    assert!(response.contains("HTTP/1.1 200 OK"));
    assert!(response.contains("application/javascript"));
    assert!(response.contains("console.log('test');"));

    handle.abort();
}

#[tokio::test]
async fn test_dashboard_server_404_for_missing_file() {
    let temp_dir = tempfile::tempdir().unwrap();
    let static_dir = temp_dir.path().join("dashboard").join("dist");
    fs::create_dir_all(&static_dir).unwrap();

    let port = find_available_port();
    let server = DashboardServer::new(static_dir, port);

    let handle = server.start().await.unwrap();

    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    let addr = format!("127.0.0.1:{}", port);
    let mut stream = TcpStream::connect(&addr).await.unwrap();

    let request = "GET /nonexistent.html HTTP/1.1\r\nHost: localhost\r\n\r\n";
    stream.write_all(request.as_bytes()).await.unwrap();

    let mut buffer = [0; 1024];
    let n = stream.read(&mut buffer).await.unwrap();
    let response = String::from_utf8_lossy(&buffer[..n]);

    assert!(response.contains("HTTP/1.1 404 Not Found"));

    handle.abort();
}

#[tokio::test]
async fn test_dashboard_server_rejects_directory_traversal() {
    let temp_dir = tempfile::tempdir().unwrap();
    let static_dir = temp_dir.path().join("dashboard").join("dist");
    fs::create_dir_all(&static_dir).unwrap();

    let port = find_available_port();
    let server = DashboardServer::new(static_dir, port);

    let handle = server.start().await.unwrap();

    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    let addr = format!("127.0.0.1:{}", port);
    let mut stream = TcpStream::connect(&addr).await.unwrap();

    // Try directory traversal attack
    let request = "GET /../../etc/passwd HTTP/1.1\r\nHost: localhost\r\n\r\n";
    stream.write_all(request.as_bytes()).await.unwrap();

    let mut buffer = [0; 1024];
    let n = stream.read(&mut buffer).await.unwrap();
    let response = String::from_utf8_lossy(&buffer[..n]);

    // Should reject with 403 Forbidden
    assert!(response.contains("HTTP/1.1 403 Forbidden"));

    handle.abort();
}

#[tokio::test]
async fn test_dashboard_server_rejects_non_get_methods() {
    let temp_dir = tempfile::tempdir().unwrap();
    let static_dir = temp_dir.path().join("dashboard").join("dist");
    fs::create_dir_all(&static_dir).unwrap();

    let port = find_available_port();
    let server = DashboardServer::new(static_dir, port);

    let handle = server.start().await.unwrap();

    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    let addr = format!("127.0.0.1:{}", port);
    let mut stream = TcpStream::connect(&addr).await.unwrap();

    // Try POST request
    let request = "POST / HTTP/1.1\r\nHost: localhost\r\n\r\n";
    stream.write_all(request.as_bytes()).await.unwrap();

    let mut buffer = [0; 1024];
    let n = stream.read(&mut buffer).await.unwrap();
    let response = String::from_utf8_lossy(&buffer[..n]);

    // Should reject with 405 Method Not Allowed
    assert!(response.contains("HTTP/1.1 405 Method Not Allowed"));

    handle.abort();
}

