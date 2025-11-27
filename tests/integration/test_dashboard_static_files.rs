use otlp_arrow_library::dashboard::server::DashboardServer;
use std::fs;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

fn find_available_port() -> u16 {
    use std::net::TcpListener;
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    listener.local_addr().unwrap().port()
}

#[tokio::test]
async fn test_dashboard_serves_html_with_correct_content_type() {
    let temp_dir = tempfile::tempdir().unwrap();
    let static_dir = temp_dir.path().join("dashboard").join("dist");
    fs::create_dir_all(&static_dir).unwrap();

    fs::write(static_dir.join("index.html"), "<html><body>Test</body></html>").unwrap();

    let port = find_available_port();
    let server = DashboardServer::new(static_dir, port, "127.0.0.1");
    let handle = server.start().await.unwrap();

    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    let addr = format!("127.0.0.1:{}", port);
    let mut stream = TcpStream::connect(&addr).await.unwrap();

    let request = "GET / HTTP/1.1\r\nHost: localhost\r\n\r\n";
    stream.write_all(request.as_bytes()).await.unwrap();

    let mut buffer = [0; 4096];
    let n = stream.read(&mut buffer).await.unwrap();
    let response = String::from_utf8_lossy(&buffer[..n]);

    assert!(response.contains("Content-Type: text/html; charset=utf-8"));
    assert!(response.contains("<html><body>Test</body></html>"));

    handle.abort();
}

#[tokio::test]
async fn test_dashboard_serves_js_with_correct_content_type() {
    let temp_dir = tempfile::tempdir().unwrap();
    let static_dir = temp_dir.path().join("dashboard").join("dist");
    fs::create_dir_all(&static_dir).unwrap();

    fs::write(static_dir.join("app.js"), "console.log('app');").unwrap();

    let port = find_available_port();
    let server = DashboardServer::new(static_dir, port, "127.0.0.1");
    let handle = server.start().await.unwrap();

    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    let addr = format!("127.0.0.1:{}", port);
    let mut stream = TcpStream::connect(&addr).await.unwrap();

    let request = "GET /app.js HTTP/1.1\r\nHost: localhost\r\n\r\n";
    stream.write_all(request.as_bytes()).await.unwrap();

    let mut buffer = [0; 4096];
    let n = stream.read(&mut buffer).await.unwrap();
    let response = String::from_utf8_lossy(&buffer[..n]);

    assert!(response.contains("Content-Type: application/javascript; charset=utf-8"));
    assert!(response.contains("console.log('app');"));

    handle.abort();
}

#[tokio::test]
async fn test_dashboard_serves_css_with_correct_content_type() {
    let temp_dir = tempfile::tempdir().unwrap();
    let static_dir = temp_dir.path().join("dashboard").join("dist");
    fs::create_dir_all(&static_dir).unwrap();

    fs::write(static_dir.join("styles.css"), "body { margin: 0; }").unwrap();

    let port = find_available_port();
    let server = DashboardServer::new(static_dir, port, "127.0.0.1");
    let handle = server.start().await.unwrap();

    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    let addr = format!("127.0.0.1:{}", port);
    let mut stream = TcpStream::connect(&addr).await.unwrap();

    let request = "GET /styles.css HTTP/1.1\r\nHost: localhost\r\n\r\n";
    stream.write_all(request.as_bytes()).await.unwrap();

    let mut buffer = [0; 4096];
    let n = stream.read(&mut buffer).await.unwrap();
    let response = String::from_utf8_lossy(&buffer[..n]);

    assert!(response.contains("Content-Type: text/css; charset=utf-8"));
    assert!(response.contains("body { margin: 0; }"));

    handle.abort();
}

#[tokio::test]
async fn test_dashboard_serves_nested_files() {
    let temp_dir = tempfile::tempdir().unwrap();
    let static_dir = temp_dir.path().join("dashboard").join("dist");
    fs::create_dir_all(&static_dir).unwrap();

    let nested_dir = static_dir.join("assets");
    fs::create_dir_all(&nested_dir).unwrap();
    fs::write(nested_dir.join("logo.png"), b"fake png data").unwrap();

    let port = find_available_port();
    let server = DashboardServer::new(static_dir, port, "127.0.0.1");
    let handle = server.start().await.unwrap();

    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    let addr = format!("127.0.0.1:{}", port);
    let mut stream = TcpStream::connect(&addr).await.unwrap();

    let request = "GET /assets/logo.png HTTP/1.1\r\nHost: localhost\r\n\r\n";
    stream.write_all(request.as_bytes()).await.unwrap();

    let mut buffer = [0; 4096];
    let n = stream.read(&mut buffer).await.unwrap();
    let response = String::from_utf8_lossy(&buffer[..n]);

    assert!(response.contains("HTTP/1.1 200 OK"));
    assert!(response.contains("Content-Type: image/png"));
    assert!(response.contains("fake png data"));

    handle.abort();
}

#[tokio::test]
async fn test_dashboard_serves_wasm_with_correct_content_type() {
    let temp_dir = tempfile::tempdir().unwrap();
    let static_dir = temp_dir.path().join("dashboard").join("dist");
    fs::create_dir_all(&static_dir).unwrap();

    // Create a minimal WASM file (just the header bytes: 0x00 0x61 0x73 0x6d = "\0asm")
    let wasm_header = [0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00];
    fs::write(static_dir.join("test.wasm"), &wasm_header).unwrap();

    let port = find_available_port();
    let server = DashboardServer::new(static_dir, port, "127.0.0.1");
    let handle = server.start().await.unwrap();

    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    let addr = format!("127.0.0.1:{}", port);
    let mut stream = TcpStream::connect(&addr).await.unwrap();

    let request = "GET /test.wasm HTTP/1.1\r\nHost: localhost\r\n\r\n";
    stream.write_all(request.as_bytes()).await.unwrap();

    let mut buffer = [0; 4096];
    let n = stream.read(&mut buffer).await.unwrap();
    let response = String::from_utf8_lossy(&buffer[..n]);

    // Verify correct MIME type for WASM files (required for WebAssembly compilation)
    assert!(
        response.contains("Content-Type: application/wasm"),
        "Response should contain 'Content-Type: application/wasm', got: {}",
        response
    );
    assert!(response.contains("HTTP/1.1 200 OK"));

    handle.abort();
}

