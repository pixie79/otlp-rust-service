# Data Model: Security, Bug, and Memory Fixes

**Feature**: Fix Security, Bug, and Memory Issues  
**Date**: 2025-01-27  
**Phase**: 1 - Design & Contracts

## Overview

This feature fixes security vulnerabilities, bugs, and memory management issues. The data model changes are minimal - primarily updating existing types to use secure storage and adding configuration fields for buffer limits. No new major entities are introduced.

## Entity Changes

### AuthConfig (Modified)

**Location**: `src/config/types.rs`

**Current Structure**:
```rust
pub struct AuthConfig {
    pub auth_type: String,
    pub credentials: HashMap<String, String>,  // INSECURE
}
```

**Updated Structure**:
```rust
use secrecy::SecretString;

pub struct AuthConfig {
    pub auth_type: String,
    pub credentials: HashMap<String, SecretString>,  // SECURE
}
```

**Changes**:
- `credentials` values changed from `String` to `SecretString`
- Ensures credentials are zeroed on drop
- Prevents credentials from appearing in `Debug` or `Display` implementations
- Requires updates to all credential access points

**Validation Rules**:
- `auth_type` must not be empty
- For `api_key`: credentials must contain `"key"` (not `"api_key"` or `"token"`)
- For `bearer_token`: credentials must contain `"token"`
- For `basic`: credentials must contain both `"username"` and `"password"`

**State Transitions**: None (static configuration)

---

### Config (Modified)

**Location**: `src/config/types.rs`

**Current Structure**:
```rust
pub struct Config {
    pub output_dir: PathBuf,
    pub write_interval_secs: u64,
    pub trace_cleanup_interval_secs: u64,
    pub metric_cleanup_interval_secs: u64,
    pub protocols: ProtocolConfig,
    pub forwarding: Option<ForwardingConfig>,
    pub dashboard: DashboardConfig,
    // NO BUFFER LIMITS
}
```

**Updated Structure**:
```rust
pub struct Config {
    pub output_dir: PathBuf,
    pub write_interval_secs: u64,
    pub trace_cleanup_interval_secs: u64,
    pub metric_cleanup_interval_secs: u64,
    pub max_trace_buffer_size: usize,      // NEW: default 10000
    pub max_metric_buffer_size: usize,     // NEW: default 10000
    pub protocols: ProtocolConfig,
    pub forwarding: Option<ForwardingConfig>,
    pub dashboard: DashboardConfig,
}
```

**Changes**:
- Added `max_trace_buffer_size: usize` (default: 10000)
- Added `max_metric_buffer_size: usize` (default: 10000)
- Used by `BatchBuffer` to enforce memory limits

**Validation Rules**:
- `max_trace_buffer_size` must be > 0 and <= 1,000,000 (reasonable upper bound)
- `max_metric_buffer_size` must be > 0 and <= 1,000,000 (reasonable upper bound)
- Values should be configured based on available memory and workload

**State Transitions**: None (static configuration)

---

### DashboardConfig (Modified)

**Location**: `src/config/types.rs`

**Current Structure**:
```rust
pub struct DashboardConfig {
    pub enabled: bool,
    pub port: u16,
    pub static_dir: PathBuf,
    pub bind_address: String,
    // NO SECURITY HEADER CONFIGURATION
}
```

**Updated Structure**:
```rust
pub struct DashboardConfig {
    pub enabled: bool,
    pub port: u16,
    pub static_dir: PathBuf,
    pub bind_address: String,
    pub x_frame_options: Option<String>,  // NEW: Optional, default "DENY"
}
```

**Changes**:
- Added `x_frame_options: Option<String>` for configurable X-Frame-Options header
- Default: `"DENY"` if not specified
- Allows `"SAMEORIGIN"` if iframe embedding is needed

**Validation Rules**:
- If `x_frame_options` is `Some`, value must be `"DENY"` or `"SAMEORIGIN"`
- If `None`, defaults to `"DENY"`

**State Transitions**: None (static configuration)

---

### BatchBuffer (Modified)

**Location**: `src/otlp/batch_writer.rs`

**Current Structure**:
```rust
pub struct BatchBuffer {
    traces: Arc<Mutex<Vec<SpanData>>>,
    metrics: Arc<Mutex<Vec<ExportMetricsServiceRequest>>>,
    write_interval: Duration,
    last_write: Arc<Mutex<SystemTime>>,
    // NO SIZE LIMITS
}
```

**Updated Structure**:
```rust
pub struct BatchBuffer {
    traces: Arc<Mutex<Vec<SpanData>>>,
    metrics: Arc<Mutex<Vec<ExportMetricsServiceRequest>>>,
    write_interval: Duration,
    last_write: Arc<Mutex<SystemTime>>,
    max_trace_size: usize,      // NEW: from Config
    max_metric_size: usize,     // NEW: from Config
}
```

**Changes**:
- Added `max_trace_size: usize` field
- Added `max_metric_size: usize` field
- Methods check size before adding items
- Returns `BufferFull` error when limit reached

**Validation Rules**:
- `add_trace()` checks `traces.len() < max_trace_size` before adding
- `add_metrics_protobuf()` checks `metrics.len() < max_metric_size` before adding
- Returns `OtlpError::Export(OtlpExportError::BufferFull)` when limit reached

**State Transitions**: None (stateless buffer)

---

### CircuitBreaker (Modified)

**Location**: `src/otlp/forwarder.rs`

**Current Structure**:
```rust
enum CircuitState {
    Closed,
    Open,
    HalfOpen,  // INCOMPLETE IMPLEMENTATION
}

struct CircuitBreaker {
    state: Arc<Mutex<CircuitState>>,
    failure_count: Arc<Mutex<u32>>,
    last_failure_time: Arc<Mutex<Option<Instant>>>,
    failure_threshold: u32,
    timeout: Duration,
    half_open_timeout: Duration,
}
```

**Updated Structure**:
```rust
enum CircuitState {
    Closed,   // Normal operation
    Open,     // Failing, reject requests
    HalfOpen, // Testing if service recovered - NOW FULLY IMPLEMENTED
}

struct CircuitBreaker {
    state: Arc<Mutex<CircuitState>>,
    failure_count: Arc<Mutex<u32>>,
    last_failure_time: Arc<Mutex<Option<Instant>>>,
    failure_threshold: u32,
    timeout: Duration,
    half_open_timeout: Duration,
    half_open_test_in_progress: Arc<Mutex<bool>>,  // NEW: prevent concurrent tests
}
```

**Changes**:
- Added `half_open_test_in_progress: Arc<Mutex<bool>>` to prevent concurrent test requests
- Complete implementation of `HalfOpen` state handling

**State Transitions**:
- `Closed` → `Open`: When `failure_count >= failure_threshold`
- `Open` → `HalfOpen`: When `timeout` elapses
- `HalfOpen` → `Closed`: When test request succeeds
- `HalfOpen` → `Open`: When test request fails

**Validation Rules**:
- Only one test request allowed in `HalfOpen` state
- State transitions are atomic (protected by mutex)

---

## No New Entities

This feature does not introduce new major entities. Changes are limited to:
- Updating existing types for security (credentials)
- Adding configuration fields (buffer limits, security headers)
- Completing incomplete implementations (circuit breaker)

## Relationships

- `Config` contains `AuthConfig` (via `ForwardingConfig`)
- `Config` contains `DashboardConfig`
- `Config` provides buffer limits to `BatchBuffer`
- `CircuitBreaker` is used by `OtlpForwarder`
- `AuthConfig` credentials are used by `OtlpForwarder` for authentication

## Data Flow

1. **Credential Storage**: User provides credentials → `AuthConfig` stores as `SecretString` → Used by `OtlpForwarder` → Never exposed in logs/errors
2. **Buffer Management**: Traces/metrics → `BatchBuffer` → Check size limits → Add if under limit, error if at limit → Write to disk periodically
3. **Path Validation**: HTTP request → Parse path → Validate (absolute, traversal, symlinks) → Reject if invalid → Serve if valid
4. **Circuit Breaker**: Request → Check state → Allow if Closed/HalfOpen → Update state based on result → Transition states accordingly

## Validation Summary

- Credentials: Stored securely, never exposed
- Buffer sizes: Configurable limits, enforced at add time
- Paths: Validated for traversal attacks, platform-specific checks
- URLs: Parsed and validated using `url` crate
- Circuit breaker: Complete state machine with proper transitions

