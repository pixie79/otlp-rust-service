# Configuration API Contract

**Feature**: Fix Security, Bug, and Memory Issues  
**Date**: 2025-01-27  
**Type**: Configuration API Changes

## Overview

This contract defines the changes to the configuration API for security, bug fixes, and memory management improvements.

## Breaking Changes

### AuthConfig::credentials Type Change

**Before**:
```rust
pub struct AuthConfig {
    pub auth_type: String,
    pub credentials: HashMap<String, String>,
}
```

**After**:
```rust
use secrecy::SecretString;

pub struct AuthConfig {
    pub auth_type: String,
    pub credentials: HashMap<String, SecretString>,
}
```

**Impact**: Breaking change - code using `AuthConfig` must be updated to use `SecretString` instead of `String` for credential values.

**Migration**:
```rust
// Before
let mut creds = HashMap::new();
creds.insert("token".to_string(), "my-token".to_string());

// After
use secrecy::SecretString;
let mut creds = HashMap::new();
creds.insert("token".to_string(), SecretString::new("my-token".to_string()));
```

---

## New Configuration Fields

### Config::max_trace_buffer_size

**Type**: `usize`  
**Default**: `10000`  
**Description**: Maximum number of trace spans that can be buffered in memory before backpressure is applied.

**Validation**:
- Must be > 0
- Must be <= 1,000,000 (reasonable upper bound)
- Recommended: 10,000 - 100,000 based on available memory

**Example**:
```rust
let config = ConfigBuilder::new()
    .max_trace_buffer_size(50000)
    .build()?;
```

---

### Config::max_metric_buffer_size

**Type**: `usize`  
**Default**: `10000`  
**Description**: Maximum number of metric requests that can be buffered in memory before backpressure is applied.

**Validation**:
- Must be > 0
- Must be <= 1,000,000 (reasonable upper bound)
- Recommended: 10,000 - 100,000 based on available memory

**Example**:
```rust
let config = ConfigBuilder::new()
    .max_metric_buffer_size(50000)
    .build()?;
```

---

### DashboardConfig::x_frame_options

**Type**: `Option<String>`  
**Default**: `None` (which results in `"DENY"` header)  
**Description**: Configurable X-Frame-Options HTTP header value. Allows customization for deployments that need iframe embedding.

**Validation**:
- If `Some`, value must be `"DENY"` or `"SAMEORIGIN"`
- If `None`, defaults to `"DENY"`

**Example**:
```rust
let dashboard = DashboardConfig {
    enabled: true,
    port: 8080,
    static_dir: PathBuf::from("./dashboard/dist"),
    bind_address: "127.0.0.1".to_string(),
    x_frame_options: Some("SAMEORIGIN".to_string()), // Allow iframe embedding
};
```

---

## Updated Validation Rules

### AuthConfig::validate()

**Before**: Checked for `"token"` or `"api_key"` keys for `api_key` auth type.

**After**: 
- For `api_key`: Requires `"key"` in credentials (matches actual usage)
- For `bearer_token`: Requires `"token"` in credentials
- For `basic`: Requires both `"username"` and `"password"` in credentials

**Rationale**: Validation now matches actual credential key names used in `add_auth_headers()` method.

---

### ForwardingConfig::validate()

**Before**: Simple prefix check (`starts_with("http://")` or `starts_with("https://")`).

**After**: Uses `url::Url::parse()` for comprehensive URL validation.

**Validation**:
- URL must parse successfully
- Scheme must be `http` or `https`
- Host must be present
- Clear error messages for invalid URLs

**Example Error Messages**:
- `"Invalid URL format: missing scheme"`
- `"Invalid URL format: unsupported scheme (must be http or https)"`
- `"Invalid URL format: missing host"`

---

## YAML Configuration

### Example Configuration

```yaml
output_dir: ./output_dir
write_interval_secs: 5
trace_cleanup_interval_secs: 600
metric_cleanup_interval_secs: 3600
max_trace_buffer_size: 50000      # NEW
max_metric_buffer_size: 50000     # NEW

protocols:
  protobuf_enabled: true
  protobuf_port: 4317
  arrow_flight_enabled: true
  arrow_flight_port: 4318

forwarding:
  enabled: true
  endpoint_url: "https://collector.example.com:4317"  # Now properly validated
  protocol: protobuf
  authentication:
    auth_type: bearer_token
    credentials:
      token: "my-secure-token"  # Stored as SecretString internally

dashboard:
  enabled: true
  port: 8080
  static_dir: ./dashboard/dist
  bind_address: "127.0.0.1"
  x_frame_options: "DENY"  # NEW: Optional, defaults to DENY
```

---

## Environment Variables

New environment variables:

- `OTLP_MAX_TRACE_BUFFER_SIZE`: Maximum trace buffer size (default: 10000)
- `OTLP_MAX_METRIC_BUFFER_SIZE`: Maximum metric buffer size (default: 10000)
- `OTLP_DASHBOARD_X_FRAME_OPTIONS`: X-Frame-Options header value (default: DENY)

---

## Backward Compatibility

- **Breaking**: `AuthConfig::credentials` type change requires code updates
- **Non-breaking**: New buffer size fields have defaults, existing configs work
- **Non-breaking**: New `x_frame_options` field is optional, defaults to secure value
- **Non-breaking**: URL validation is stricter but rejects invalid URLs that would have failed anyway

## Error Types

### BufferFull Error

**When**: Buffer size limit reached  
**Error**: `OtlpError::Export(OtlpExportError::BufferFull)`  
**Recovery**: Wait for buffer to drain, reduce input rate, or increase buffer size

### InvalidUrl Error

**When**: URL validation fails  
**Error**: `OtlpConfigError::InvalidUrl(String)`  
**Recovery**: Fix URL format, ensure scheme is http/https, ensure host is present

