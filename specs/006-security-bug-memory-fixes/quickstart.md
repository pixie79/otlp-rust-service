# Quickstart: Security, Bug, and Memory Fixes

**Feature**: Fix Security, Bug, and Memory Issues  
**Date**: 2025-01-27

## Overview

This quickstart guide covers the security, bug, and memory management fixes implemented in this feature. These are critical fixes that address vulnerabilities and stability issues.

## What's Fixed

### Security Fixes

1. **Secure Credential Storage** - Credentials now use `SecretString` to prevent memory exposure
2. **Path Traversal Protection** - Enhanced path validation prevents directory traversal attacks
3. **Security Headers** - HTTP responses include security headers (CSP, X-Frame-Options, etc.)
4. **Input Validation** - Comprehensive URL and path validation

### Bug Fixes

1. **Syntax Error** - Fixed missing brace in basic auth handler
2. **Auth Validation Logic** - Fixed mismatch between validation and usage
3. **Circuit Breaker** - Completed half-open state implementation
4. **Protobuf Encoding** - Implemented proper Protobuf serialization for forwarding

### Memory Management

1. **Buffer Limits** - Added configurable size limits to prevent unbounded memory growth
2. **Python Segfaults** - Fixed memory safety issues in Python bindings

## Configuration Changes

### New Configuration Fields

Add buffer size limits to your configuration:

```rust
use otlp_arrow_library::{ConfigBuilder, OtlpLibrary};

let config = ConfigBuilder::new()
    .output_dir("./output")
    .max_trace_buffer_size(50000)      // NEW: Limit trace buffer
    .max_metric_buffer_size(50000)     // NEW: Limit metric buffer
    .build()?;

let library = OtlpLibrary::new(config).await?;
```

Or via YAML:

```yaml
max_trace_buffer_size: 50000
max_metric_buffer_size: 50000
```

Or via environment variables:

```bash
export OTLP_MAX_TRACE_BUFFER_SIZE=50000
export OTLP_MAX_METRIC_BUFFER_SIZE=50000
```

### Breaking Change: Credential Storage

If you're using `AuthConfig` programmatically, you must update to use `SecretString`:

```rust
// Before (INSECURE - DO NOT USE)
use std::collections::HashMap;
let mut creds = HashMap::new();
creds.insert("token".to_string(), "my-token".to_string());

// After (SECURE)
use secrecy::SecretString;
use std::collections::HashMap;
let mut creds = HashMap::new();
creds.insert("token".to_string(), SecretString::new("my-token".to_string()));
```

YAML configuration remains the same - the library handles conversion internally:

```yaml
authentication:
  auth_type: bearer_token
  credentials:
    token: "my-token"  # Automatically converted to SecretString
```

## Security Headers

Security headers are automatically added to all HTTP responses. To customize:

```rust
use otlp_arrow_library::config::DashboardConfig;

let dashboard = DashboardConfig {
    enabled: true,
    port: 8080,
    static_dir: PathBuf::from("./dashboard/dist"),
    bind_address: "127.0.0.1".to_string(),
    x_frame_options: Some("SAMEORIGIN".to_string()), // Allow iframe embedding if needed
};
```

Default headers added:
- `X-Content-Type-Options: nosniff`
- `X-Frame-Options: DENY` (or configured value)
- `Content-Security-Policy: default-src 'self'`
- `X-XSS-Protection: 1; mode=block`

## Buffer Management

### Understanding Buffer Limits

Buffer limits prevent unbounded memory growth. When limits are reached:

1. New additions return `BufferFull` error
2. System applies backpressure
3. Existing data continues to be written to disk
4. Once buffer drains, new data can be added

### Handling BufferFull Errors

```rust
match library.export_trace(span).await {
    Ok(()) => {
        // Success
    }
    Err(OtlpError::Export(OtlpExportError::BufferFull)) => {
        // Buffer is full - wait or reduce input rate
        tokio::time::sleep(Duration::from_millis(100)).await;
        // Retry or handle backpressure
    }
    Err(e) => {
        // Other error
        eprintln!("Error: {}", e);
    }
}
```

### Recommended Buffer Sizes

- **Small deployments**: 10,000 (default)
- **Medium deployments**: 50,000
- **Large deployments**: 100,000
- **Maximum**: 1,000,000 (configurable limit)

Consider:
- Available memory
- Write interval (longer intervals need larger buffers)
- Expected throughput

## Path Validation

Path validation is now comprehensive and prevents:
- Directory traversal (`../`)
- Absolute paths
- Symlink attacks
- UNC paths (Windows)

All invalid paths return `403 Forbidden` responses.

## URL Validation

URL validation is now strict. Valid URLs must:
- Parse successfully
- Use `http://` or `https://` scheme
- Include a host

Invalid URLs return clear error messages:

```rust
let config = ConfigBuilder::new()
    .enable_forwarding(ForwardingConfig {
        enabled: true,
        endpoint_url: Some("invalid-url".to_string()), // Will fail validation
        protocol: ForwardingProtocol::Protobuf,
        authentication: None,
    })
    .build();

// Returns: Err(OtlpConfigError::InvalidUrl("missing scheme"))
```

## Circuit Breaker

Circuit breaker now properly handles recovery:

1. **Closed** → **Open**: After failure threshold reached
2. **Open** → **HalfOpen**: After timeout period
3. **HalfOpen** → **Closed**: On successful test request
4. **HalfOpen** → **Open**: On failed test request

No code changes needed - works automatically.

## Testing

### Verify Security Fixes

```bash
# Test path traversal protection
curl http://localhost:8080/data/../../etc/passwd
# Should return: 403 Forbidden

# Test security headers
curl -I http://localhost:8080/
# Should include: X-Frame-Options, X-Content-Type-Options, CSP
```

### Verify Buffer Limits

```rust
#[tokio::test]
async fn test_buffer_limit() {
    let config = ConfigBuilder::new()
        .max_trace_buffer_size(10) // Small limit for testing
        .build()?;
    
    let library = OtlpLibrary::new(config).await?;
    
    // Add 10 spans - should succeed
    for i in 0..10 {
        library.export_trace(create_test_span(i)).await?;
    }
    
    // 11th span should fail
    assert!(library.export_trace(create_test_span(11)).await.is_err());
}
```

### Verify Credential Security

```rust
#[test]
fn test_credentials_not_in_debug() {
    use secrecy::SecretString;
    let secret = SecretString::new("my-secret".to_string());
    
    // Debug should not expose secret
    let debug_str = format!("{:?}", secret);
    assert!(!debug_str.contains("my-secret"));
}
```

## Migration Checklist

- [ ] Update code using `AuthConfig` to use `SecretString`
- [ ] Configure buffer size limits appropriate for your workload
- [ ] Review and test path validation (if using dashboard)
- [ ] Verify URL validation (if using forwarding)
- [ ] Test circuit breaker recovery (if using forwarding)
- [ ] Update documentation referencing credential storage
- [ ] Run full test suite to verify compatibility

## Troubleshooting

### BufferFull Errors

**Symptom**: Frequent `BufferFull` errors  
**Solution**: Increase buffer size limits or reduce input rate

### Credential Access Errors

**Symptom**: Compilation errors with `AuthConfig`  
**Solution**: Update to use `SecretString::new()` for credential values

### Path Validation Too Strict

**Symptom**: Valid paths rejected  
**Solution**: Check path format - ensure relative paths, no `../`, no absolute paths

### URL Validation Fails

**Symptom**: Valid URLs rejected  
**Solution**: Ensure URL includes scheme (`http://` or `https://`) and host

## Next Steps

1. Review [Configuration API Contract](./contracts/config-api.md) for detailed API changes
2. Review [Data Model](./data-model.md) for entity changes
3. Run tests to verify fixes
4. Update deployment configurations with buffer limits
5. Review security headers configuration

## References

- [GitHub Issues Fixed](../spec.md#notes)
- [Research Findings](./research.md)
- [Implementation Plan](./plan.md)

