# Rust Service Configuration Contract: Dashboard Integration

**Date**: 2024-12-19  
**Feature**: 002-realtime-dashboard

## Overview

This document defines the configuration contract for integrating the web dashboard with the Rust service. The Rust service can optionally serve the dashboard static files via HTTP.

## Configuration Schema

### YAML Configuration

```yaml
dashboard:
  enabled: false  # Default: false (disabled)
  port: 8080      # Default: 8080 (when enabled)
  static_dir: "./dashboard/dist"  # Default: "./dashboard/dist"
```

### Configuration Structure

```rust
pub struct DashboardConfig {
    pub enabled: bool,
    pub port: u16,
    pub static_dir: PathBuf,
}
```

**Default Values**:
- `enabled`: `false`
- `port`: `8080`
- `static_dir`: `"./dashboard/dist"`

---

## Environment Variable Overrides

Environment variables override YAML configuration:

- `OTLP_DASHBOARD_ENABLED`: `true` or `false` (string)
- `OTLP_DASHBOARD_PORT`: Port number (u16)
- `OTLP_DASHBOARD_STATIC_DIR`: Path to static files directory

**Priority**: Environment variables > YAML config > defaults

---

## Configuration Validation

### Validation Rules

1. **enabled**: Must be boolean
2. **port**: Must be valid port (1-65535), must not conflict with gRPC ports (4317, 4318)
3. **static_dir**: Must be valid path, directory must exist when enabled is true

### Validation Errors

- Invalid port: `OtlpConfigError::ValidationFailed("Dashboard port must be between 1 and 65535")`
- Port conflict: `OtlpConfigError::ValidationFailed("Dashboard port conflicts with gRPC port")`
- Missing directory: `OtlpConfigError::InvalidOutputDir("Dashboard static directory does not exist")`

---

## HTTP Server Contract

### When Enabled

When `dashboard.enabled: true`:

1. **HTTP Server**: Rust service starts HTTP server on configured port
2. **Static File Serving**: Serves files from `static_dir` directory
3. **Root Path**: Dashboard files served at `/` (root)
4. **File Access**: Dashboard still uses direct file system access (File System Access API/FileReader API) - Rust service does NOT serve Arrow IPC files

### HTTP Server Implementation

```rust
// Pseudo-code structure
if config.dashboard.enabled {
    let static_dir = config.dashboard.static_dir.clone();
    let port = config.dashboard.port;
    
    // Start HTTP server serving static files
    tokio::spawn(async move {
        serve_static_files(static_dir, port).await;
    });
}
```

### Static File Serving

- **Method**: GET requests only
- **Content-Type**: Determined by file extension (text/html, application/javascript, text/css, etc.)
- **Index File**: Serve `index.html` for root path `/`
- **Error Handling**: 404 for missing files, 500 for server errors

---

## Integration with Existing Service

### Service Startup

1. Load configuration (YAML + environment variables)
2. Validate configuration
3. If `dashboard.enabled: true`:
   - Verify `static_dir` exists
   - Start HTTP server on separate task
   - Log dashboard URL: `"Dashboard available at http://localhost:{port}"`
4. Start gRPC servers (Protobuf and Arrow Flight) as before
5. Start file exporter as before

### Service Shutdown

1. Stop HTTP server (if running)
2. Stop gRPC servers
3. Flush file exporter
4. Graceful shutdown

---

## Configuration Example

### YAML Example

```yaml
output_dir: ./output_dir
write_interval_secs: 5
protocols:
  protobuf_enabled: true
  protobuf_port: 4317
  arrow_flight_enabled: true
  arrow_flight_port: 4318
dashboard:
  enabled: true
  port: 8080
  static_dir: ./dashboard/dist
```

### Environment Variable Example

```bash
export OTLP_DASHBOARD_ENABLED=true
export OTLP_DASHBOARD_PORT=8080
export OTLP_DASHBOARD_STATIC_DIR=./dashboard/dist
```

### Programmatic Example

```rust
use otlp_arrow_library::{ConfigBuilder, DashboardConfig};

let config = ConfigBuilder::new()
    .output_dir("./output_dir")
    .dashboard(DashboardConfig {
        enabled: true,
        port: 8080,
        static_dir: PathBuf::from("./dashboard/dist"),
    })
    .build()?;
```

---

## Error Handling

### Configuration Errors

- Invalid port: Service fails to start with validation error
- Missing directory: Service fails to start with validation error
- Port conflict: Service fails to start with validation error

### Runtime Errors

- HTTP server startup failure: Log error, continue with gRPC servers (dashboard optional)
- Static file read error: Return 404 or 500 as appropriate
- Server crash: Log error, attempt graceful shutdown

---

## Logging

### Startup Logs

```
INFO: Dashboard disabled (default)
# OR
INFO: Dashboard enabled, serving static files from ./dashboard/dist on port 8080
INFO: Dashboard available at http://localhost:8080
```

### Runtime Logs

```
INFO: Dashboard HTTP server started on port 8080
ERROR: Failed to serve dashboard file: index.html (file not found)
```

---

## Testing Contract

### Unit Tests

- Configuration loading from YAML
- Configuration loading from environment variables
- Configuration validation (valid/invalid ports, directories)
- Default values when not specified

### Integration Tests

- HTTP server starts when enabled
- HTTP server does not start when disabled
- Static files served correctly
- Port conflicts detected
- Missing directory detected

---

## Future Enhancements (Out of Scope)

- Authentication/authorization for dashboard
- Dashboard API endpoints (dashboard reads files directly)
- WebSocket support for real-time updates
- Dashboard configuration via HTTP API

