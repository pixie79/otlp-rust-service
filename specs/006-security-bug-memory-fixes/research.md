# Research: Security, Bug, and Memory Fixes

**Feature**: Fix Security, Bug, and Memory Issues  
**Date**: 2025-01-27  
**Phase**: 0 - Research & Technical Decisions

## Research Questions

### 1. Secure Credential Storage

**Question**: What is the best approach for storing credentials securely in Rust to prevent memory exposure?

**Research Findings**:

- **Decision**: Use `secrecy` crate with `SecretString` type
- **Rationale**: 
  - `secrecy` is the de-facto standard Rust crate for secure string handling
  - Provides `SecretString` type that implements `Zeroize` trait to zero memory on drop
  - Prevents credentials from appearing in `Debug` and `Display` implementations
  - Well-maintained and widely used in Rust security-sensitive projects
  - Compatible with `serde` for configuration serialization
- **Alternatives Considered**:
  - Manual `Zeroize` implementation: REJECTED - More error-prone, doesn't prevent accidental Display/Debug
  - Custom wrapper type: REJECTED - Reinventing the wheel, less battle-tested
  - Plain `String` with manual sanitization: REJECTED - Too easy to forget sanitization, doesn't prevent memory dumps

**Implementation Approach**:
- Add `secrecy = "0.8"` dependency to `Cargo.toml`
- Update `AuthConfig::credentials` from `HashMap<String, String>` to `HashMap<String, SecretString>`
- Update all credential access points to use `SecretString::new()` for creation
- Ensure `Debug` and `Display` implementations never expose credentials
- Add credential sanitization in all logging and error reporting points

**References**:
- `secrecy` crate: https://crates.io/crates/secrecy
- Rust security best practices: https://rustsec.org/

---

### 2. Path Validation for Security

**Question**: How to comprehensively validate file paths to prevent directory traversal attacks across all platforms?

**Research Findings**:

- **Decision**: Use `PathBuf::components()` to check for `ParentDir`, reject absolute paths, use `canonicalize()` with error handling, add platform-specific checks
- **Rationale**:
  - `PathBuf::components()` provides cross-platform path component iteration
  - Checking for `Component::ParentDir` catches `../` patterns
  - Rejecting absolute paths prevents access outside intended directory
  - `canonicalize()` resolves symlinks but must be used carefully (can fail, may expose errors)
  - Platform-specific checks needed for Windows UNC paths (`\\server\share`)
- **Alternatives Considered**:
  - `dunce` crate for path canonicalization: CONSIDERED - Good for Windows, but standard library sufficient
  - Whitelist approach: REJECTED - Too restrictive, hard to maintain
  - Path sanitization library: REJECTED - Standard library provides sufficient tools

**Implementation Approach**:
1. Check if path is absolute - reject immediately
2. Check for `ParentDir` components - reject if found
3. Normalize path (remove `//`, `.`, etc.)
4. For symlink handling: Use `canonicalize()` but catch errors gracefully
5. Platform-specific: Check for Windows UNC paths (`\\` prefix)
6. Ensure resolved path stays within allowed directory

**References**:
- Rust `std::path` documentation
- OWASP Path Traversal: https://owasp.org/www-community/attacks/Path_Traversal

---

### 3. Security Headers Implementation

**Question**: What security headers should be included and how to implement them in the custom HTTP server?

**Research Findings**:

- **Decision**: Add X-Content-Type-Options, X-Frame-Options, Content-Security-Policy, X-XSS-Protection headers to all HTTP responses
- **Rationale**:
  - X-Content-Type-Options: nosniff - Prevents MIME type sniffing attacks
  - X-Frame-Options: DENY (or SAMEORIGIN if iframe embedding needed) - Prevents clickjacking
  - Content-Security-Policy: default-src 'self' - Prevents XSS attacks
  - X-XSS-Protection: 1; mode=block - Legacy browser support
- **Alternatives Considered**:
  - Use a web framework (warp, axum): REJECTED - Current custom HTTP server is lightweight and sufficient
  - Configurable headers per route: CONSIDERED - May add later, but all routes need security headers
  - Only add headers to dashboard routes: REJECTED - All HTTP responses should have security headers

**Implementation Approach**:
- Add security headers in `DashboardServer::send_response()` method
- Make headers configurable via `DashboardConfig` if needed (e.g., X-Frame-Options: SAMEORIGIN)
- Ensure headers are added to all response types (HTML, JSON, Arrow files)

**References**:
- OWASP Secure Headers: https://owasp.org/www-project-secure-headers/
- MDN Security Headers: https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers#security

---

### 4. Buffer Size Limits and Backpressure

**Question**: How to implement bounded buffers with backpressure in Rust async context?

**Research Findings**:

- **Decision**: Add size checks to existing `Vec`-based buffers, return `BufferFull` error when limit reached, add configurable limits to `Config`
- **Rationale**:
  - Existing `BatchBuffer` uses `Vec<SpanData>` and `Vec<ExportMetricsServiceRequest>`
  - Simple size check before `push()` is sufficient for backpressure
  - `BufferFull` error already exists in error types
  - Configurable limits allow users to tune based on workload
  - Default limits (10,000 traces, 10,000 metrics) are reasonable starting points
- **Alternatives Considered**:
  - Use `tokio::sync::mpsc::bounded` channel: CONSIDERED - Would require significant refactoring, current approach simpler
  - Use `crossbeam` bounded queue: REJECTED - Overkill, adds dependency
  - Dynamic resizing with soft limits: REJECTED - Defeats purpose of preventing unbounded growth

**Implementation Approach**:
- Add `max_trace_buffer_size: usize` and `max_metric_buffer_size: usize` to `Config` (default: 10000 each)
- Check buffer size before adding items in `BatchBuffer::add_trace()` and `add_metrics_protobuf()`
- Return `OtlpError::Export(OtlpExportError::BufferFull)` when limit reached
- Add metrics for buffer utilization (current size / max size)

**References**:
- Rust async patterns: https://tokio.rs/tokio/tutorial
- Backpressure patterns: https://tokio.rs/tokio/tutorial/channels

---

### 5. Circuit Breaker Half-Open State

**Question**: How to properly implement half-open state in circuit breaker pattern?

**Research Findings**:

- **Decision**: Allow single test request in half-open state, transition to Closed on success, transition back to Open on failure, add timeout to prevent indefinite half-open
- **Rationale**:
  - Half-open state is standard circuit breaker pattern (see Netflix Hystrix, Resilience4j)
  - Single test request prevents overwhelming recovering service
  - Success transitions to Closed and resets failure counters
  - Failure transitions back to Open and updates failure time
  - Timeout prevents staying in half-open indefinitely if no requests arrive
- **Alternatives Considered**:
  - Multiple test requests: REJECTED - Could overwhelm recovering service
  - Gradual reopening: CONSIDERED - More complex, single request sufficient for this use case
  - No half-open state: REJECTED - Would require manual intervention to recover

**Implementation Approach**:
- In `CircuitBreaker::call()`, handle `HalfOpen` state:
  - Allow the request to proceed (single test)
  - On success: transition to `Closed`, reset `failure_count` and `last_failure_time`
  - On failure: transition back to `Open`, update `last_failure_time`
- Add timeout check: if half-open state persists too long without requests, transition to Open

**References**:
- Circuit Breaker pattern: https://martinfowler.com/bliki/CircuitBreaker.html
- Resilience4j documentation: https://resilience4j.readme.io/docs/circuitbreaker

---

### 6. URL Validation

**Question**: How to properly validate URLs instead of simple prefix checks?

**Research Findings**:

- **Decision**: Use `url` crate for comprehensive URL parsing and validation
- **Rationale**:
  - `url` crate is the standard Rust URL parsing library
  - Provides proper URL structure validation (scheme, host, port, path, query, fragment)
  - Handles edge cases (IDN, special characters, encoding)
  - Validates URL components (e.g., port ranges, scheme requirements)
  - Well-maintained and widely used
- **Alternatives Considered**:
  - Regex-based validation: REJECTED - Error-prone, doesn't handle all edge cases
  - Manual parsing: REJECTED - Complex, error-prone, reinventing the wheel
  - `reqwest::Url`: CONSIDERED - But `url` crate is more lightweight and focused

**Implementation Approach**:
- Add `url = "2.5"` dependency to `Cargo.toml`
- In `ForwardingConfig::validate()`, use `url::Url::parse()` instead of prefix check
- Validate scheme is `http` or `https`
- Validate host is present
- Provide clear error messages for invalid URLs

**References**:
- `url` crate: https://crates.io/crates/url
- URL specification: https://url.spec.whatwg.org/

---

### 7. Python Memory Safety

**Question**: How to investigate and fix segfaults in Python bindings?

**Research Findings**:

- **Decision**: Review `unsafe` blocks, check object lifecycle management, use memory sanitizers, consider PyO3 version update
- **Rationale**:
  - Segfaults indicate memory safety violations (use-after-free, double-free, null pointer dereference)
  - PyO3 bindings use `unsafe` code for FFI - must be carefully reviewed
  - Object lifecycle (Python object reference counting) must be correct
  - Memory sanitizers (AddressSanitizer, Valgrind) can detect violations
  - PyO3 version updates may fix known issues
- **Alternatives Considered**:
  - Ignore segfaults (current workaround): REJECTED - Not acceptable for production
  - Rewrite Python bindings: REJECTED - Too drastic, likely fixable with proper lifecycle management
  - Use different FFI approach: REJECTED - PyO3 is standard, issue is likely in usage

**Implementation Approach**:
1. Review all `unsafe` blocks in `src/python/bindings.rs` and `src/python/adapters.rs`
2. Verify Python object reference counting is correct (use `PyRef`, `Py` types appropriately)
3. Check for double-free or use-after-free patterns
4. Run tests with AddressSanitizer enabled
5. Consider updating PyO3 version if issues persist
6. Add explicit lifetime management if needed

**References**:
- PyO3 documentation: https://pyo3.rs/
- PyO3 memory management: https://pyo3.rs/latest/memory.html
- AddressSanitizer: https://github.com/google/sanitizers/wiki/AddressSanitizer

---

### 8. Protobuf Encoding

**Question**: How to properly encode Protobuf messages for HTTP forwarding?

**Research Findings**:

- **Decision**: Use `prost::Message::encode()` method which is already available via `prost` dependency
- **Rationale**:
  - `prost` (0.14) is already a dependency
  - `Message` trait provides `encode()` method for serialization
  - `ExportTraceServiceRequest` and `ExportMetricsServiceRequest` implement `Message`
  - Standard approach for Protobuf serialization in Rust
- **Alternatives Considered**:
  - Use `prost_types`: REJECTED - Not needed, `prost` provides `Message` trait
  - Manual serialization: REJECTED - Error-prone, `prost` handles it correctly
  - Use `tonic` encoding: CONSIDERED - But `prost::Message::encode()` is more direct

**Implementation Approach**:
- In `send_protobuf_traces()` and `send_protobuf_metrics()`:
  - Use `request.encode(&mut buf)?` instead of `Vec::new()`
  - Ensure `buf` is properly sized or use `encode_length_delimited()` if needed
  - Handle encoding errors appropriately

**References**:
- `prost` documentation: https://docs.rs/prost/
- Protobuf encoding: https://protobuf.dev/programming-guides/encoding/

---

## Summary

All research questions resolved. Implementation will:

1. Use `secrecy::SecretString` for credential storage
2. Implement comprehensive path validation using standard library + platform checks
3. Add standard security headers to all HTTP responses
4. Add configurable buffer size limits with backpressure
5. Complete circuit breaker half-open state implementation
6. Use `url` crate for URL validation
7. Investigate Python segfaults using memory sanitizers and lifecycle review
8. Use `prost::Message::encode()` for Protobuf serialization

No blocking technical issues identified. All dependencies are either already present or standard Rust crates.

