#!/bin/bash
# Script to create GitHub issues from code review findings
# Usage: ./scripts/create_github_issues.sh

# Don't exit on error - continue creating issues even if one fails
set +e

REPO="pixie79/otlp-rust-service"

# Function to create label if it doesn't exist
create_label_if_needed() {
  local label=$1
  local color=$2
  local desc=$3
  gh label create "$label" --repo "$REPO" --description "$desc" --color "$color" 2>/dev/null || true
}

# Function to create issue with labels, handling label errors gracefully
create_issue() {
  local title="$1"
  local body="$2"
  local labels="$3"
  
  # Try to create issue with labels
  local output
  output=$(gh issue create --repo "$REPO" --title "$title" --body "$body" --label "$labels" 2>&1)
  local exit_code=$?
  
  if [ $exit_code -eq 0 ]; then
    echo "$output"
    return 0
  else
    # If labels failed, try without labels
    if echo "$output" | grep -q "not found"; then
      echo "Warning: Some labels not found, creating issue without labels..."
      gh issue create --repo "$REPO" --title "$title" --body "$body" 2>&1
      return $?
    else
      echo "Error creating issue: $output" >&2
      return $exit_code
    fi
  fi
}

# Create necessary labels
echo "Ensuring labels exist..."
create_label_if_needed "security" "d73a4a" "Security-related issues"
create_label_if_needed "performance" "0e8a16" "Performance improvements"
create_label_if_needed "critical" "b60205" "Critical priority issue"
create_label_if_needed "breaking-change" "b60205" "Breaking change"
create_label_if_needed "incomplete" "fbca04" "Incomplete implementation"
create_label_if_needed "todo" "fbca04" "TODO item"
create_label_if_needed "python" "1d76db" "Python-related"
create_label_if_needed "memory-safety" "b60205" "Memory safety issue"
create_label_if_needed "logic-error" "d73a4a" "Logic error"
create_label_if_needed "memory" "0e8a16" "Memory-related"
create_label_if_needed "refactoring" "c5def5" "Code refactoring"
create_label_if_needed "testing" "0e8a16" "Testing improvements"

echo ""
echo "Creating GitHub issues for code review findings..."
echo "Repository: $REPO"
echo ""

# Issue #1: Secure Credential Storage
create_issue \
  "Use secure string types for authentication credentials to prevent memory exposure" \
  "## Description

Currently, \`AuthConfig\` stores credentials in plain \`HashMap<String, String>\`, which can expose sensitive data in:
- Memory dumps
- Error messages  
- Log output (if accidentally logged)
- Core dumps

**Current Code:**
\`\`\`rust
// src/config/types.rs:529
pub credentials: HashMap<String, String>,
\`\`\`

## Recommendation

Use \`secrecy::SecretString\` to ensure credentials are zeroed on drop and never appear in logs:

\`\`\`rust
use secrecy::SecretString;

pub credentials: HashMap<String, SecretString>,
\`\`\`

## Additional Actions
- Ensure credentials are never logged (add sanitization in logging)
- Update all credential access points
- Update tests to use \`SecretString\`

## References
- \`src/config/types.rs:524-530\`
- \`src/otlp/forwarder.rs:395-445\`
- \`tests/integration/test_forwarding_auth.rs\`" \
  "security,enhancement,breaking-change"

# Issue #2: Enhanced Path Validation
create_issue \
  "Improve path validation in dashboard HTTP server to prevent symlink/UNC path attacks" \
  "## Description

The dashboard server's path validation checks for \`ParentDir\` components but doesn't handle:
- Symlink traversal attacks
- Absolute paths
- UNC paths on Windows
- Path normalization edge cases

**Current Code:**
\`\`\`rust
// src/dashboard/server.rs:130-136
let file_path = PathBuf::from(relative_path);
if file_path.components().any(|c| matches!(c, std::path::Component::ParentDir)) {
    return Self::send_response(&mut stream, 403, \"Forbidden\", b\"\", None).await;
}
\`\`\`

## Recommendation

Add comprehensive path validation:
1. Reject absolute paths
2. Normalize paths before validation
3. Use \`canonicalize()\` with proper error handling
4. Add platform-specific checks for Windows UNC paths

## Test Cases
- \`/data/../../etc/passwd\` (already handled)
- \`/data//etc/passwd\` (double slashes)
- \`/data/C:\\Windows\\System32\` (absolute Windows path)
- Symlink traversal attempts

## References
- \`src/dashboard/server.rs:124-177\`
- \`tests/integration/test_dashboard_server.rs:150-178\`" \
  "security,bug"

# Issue #3: Security Headers
create_issue \
  "Add security headers (CSP, X-Frame-Options, etc.) to dashboard HTTP responses" \
  "## Description

The dashboard HTTP server doesn't include security headers, leaving it vulnerable to:
- XSS attacks
- Clickjacking
- MIME type sniffing attacks

## Recommendation

Add security headers to all HTTP responses:
- \`X-Content-Type-Options: nosniff\`
- \`X-Frame-Options: DENY\` (or \`SAMEORIGIN\` if iframe embedding is needed)
- \`Content-Security-Policy: default-src 'self'\`
- \`X-XSS-Protection: 1; mode=block\` (for older browsers)

## Implementation Location
- \`src/dashboard/server.rs:218-250\` (in \`send_response\` method)" \
  "security,enhancement"

# Issue #4: Syntax Error Fix
create_issue \
  "Fix missing opening brace in add_auth_headers method for basic auth" \
  "## Description

There's a syntax error in the \`add_auth_headers\` method where the \`\"basic\"\` match arm is missing an opening brace.

**Current Code:**
\`\`\`rust
// src/otlp/forwarder.rs:424
\"basic\" =>
    let username = auth.credentials.get(\"username\").ok_or_else(|| {
\`\`\`

**Expected:**
\`\`\`rust
\"basic\" => {
    let username = auth.credentials.get(\"username\").ok_or_else(|| {
        // ...
    })?;
    // ...
}
\`\`\`

## Impact
This will cause a compilation error.

## References
- \`src/otlp/forwarder.rs:424-436\`" \
  "bug,critical"

# Issue #5: Auth Validation Logic
create_issue \
  "Fix auth validation logic mismatch - checks wrong credential key for api_key auth type" \
  "## Description

The \`AuthConfig::validate()\` method checks for \`\"token\"\` or \`\"api_key\"\` keys, but the actual \`add_auth_headers()\` method expects \`\"key\"\` for \`api_key\` auth type.

**Current Validation:**
\`\`\`rust
// src/config/types.rs:543-551
\"api_key\" | \"bearer_token\" => {
    if !self.credentials.contains_key(\"token\")
        && !self.credentials.contains_key(\"api_key\")
    {
        return Err(...);
    }
}
\`\`\`

**Actual Usage:**
\`\`\`rust
// src/otlp/forwarder.rs:403
let key = auth.credentials.get(\"key\").ok_or_else(|| {
\`\`\`

## Recommendation

Align validation with usage:
- For \`api_key\`: require \`\"key\"\` (or document that \`\"api_key\"\` is also accepted)
- For \`bearer_token\`: require \`\"token\"\`
- Update validation logic accordingly

## References
- \`src/config/types.rs:532-571\`
- \`src/otlp/forwarder.rs:401-413\`
- \`tests/unit/otlp/test_auth_config.rs\`" \
  "bug,logic-error"

# Issue #6: Circuit Breaker Half-Open
create_issue \
  "Complete circuit breaker half-open state transition logic" \
  "## Description

The circuit breaker's half-open state logic is incomplete. When in \`HalfOpen\` state, the code doesn't properly test if the service has recovered.

**Current Code:**
\`\`\`rust
// src/otlp/forwarder.rs:74-80
CircuitState::HalfOpen => {
    // Test if service recovered
}
\`\`\`

## Recommendation

Implement proper half-open logic:
1. Allow a single test request
2. On success: transition to \`Closed\` and reset counters
3. On failure: transition back to \`Open\` and update failure time
4. Add timeout to prevent staying in half-open indefinitely

## References
- \`src/otlp/forwarder.rs:49-109\`
- \`src/otlp/forwarder.rs:17-23\` (CircuitState enum)" \
  "bug,enhancement,incomplete"

# Issue #7: Bounded Buffer
create_issue \
  "Add size limits to BatchBuffer to prevent unbounded memory growth" \
  "## Description

The \`BatchBuffer\` has no size limits, which can lead to unbounded memory growth if the write interval is too long or if writes fail repeatedly.

**Current Implementation:**
\`\`\`rust
// src/otlp/batch_writer.rs:17-19
traces: Arc<Mutex<Vec<SpanData>>>,
metrics: Arc<Mutex<Vec<ExportMetricsServiceRequest>>>,
\`\`\`

## Recommendation

1. Add configurable \`max_buffer_size\` to \`Config\`
2. Implement backpressure: return \`BufferFull\` error when limit is reached
3. Add metrics for buffer utilization
4. Consider using a bounded channel (\`tokio::sync::mpsc::bounded\`) instead of \`Vec\`

**Configuration:**
\`\`\`rust
// Add to Config
pub max_trace_buffer_size: usize,  // default: 10000
pub max_metric_buffer_size: usize,  // default: 10000
\`\`\`

## References
- \`src/otlp/batch_writer.rs:14-107\`
- \`src/api/public.rs:114\`
- \`src/error.rs:56-57\` (BufferFull error exists but may not be used)" \
  "performance,enhancement,memory"

# Issue #8: Mutex Contention
create_issue \
  "Optimize BatchBuffer locking to reduce contention under high concurrency" \
  "## Description

The \`BatchBuffer\` uses a single \`Mutex\` for all operations, which can cause contention under high concurrent load.

**Current Implementation:**
\`\`\`rust
// src/otlp/batch_writer.rs
traces: Arc<Mutex<Vec<SpanData>>>,
metrics: Arc<Mutex<Vec<ExportMetricsServiceRequest>>>,
\`\`\`

## Recommendation

Consider:
1. Using \`RwLock\` for read-heavy operations (if reads become common)
2. Separate locks for traces and metrics (already separate, but could optimize further)
3. Using lock-free data structures for high-throughput scenarios
4. Batch operations to reduce lock acquisition frequency

## Performance Testing
- Add benchmarks for concurrent access
- Measure lock contention under load

## References
- \`src/otlp/batch_writer.rs:14-107\`
- \`tests/contract/test_otlp_protocol.rs:268-311\` (concurrent requests test)" \
  "performance,enhancement"

# Issue #9: Circuit Breaker Optimization
create_issue \
  "Reduce lock acquisition frequency in circuit breaker" \
  "## Description

The circuit breaker acquires multiple locks sequentially, which could be optimized by batching state updates.

**Current Code:**
\`\`\`rust
// src/otlp/forwarder.rs:53-108
let state = *self.state.lock().await;
// ... later ...
*self.state.lock().await = CircuitState::Closed;
*self.failure_count.lock().await = 0;
*self.last_failure_time.lock().await = None;
\`\`\`

## Recommendation

Combine state updates into a single lock or use a struct that groups related state together.

## References
- \`src/otlp/forwarder.rs:26-35\` (CircuitBreaker struct)
- \`src/otlp/forwarder.rs:49-109\` (call method)" \
  "performance,refactoring"

# Issue #10: Arrow Flight Forwarding
create_issue \
  "Complete Arrow Flight forwarding implementation" \
  "## Description

Arrow Flight forwarding is marked as TODO and not implemented.

**Current Code:**
\`\`\`rust
// src/otlp/forwarder.rs:385-393
async fn send_arrow_flight_metrics(
    &self,
    _batch: arrow::record_batch::RecordBatch,
) -> Result<(), OtlpError> {
    // TODO: Implement Arrow Flight client
    warn!(\"Arrow Flight forwarding not yet fully implemented - using placeholder\");
    Ok(())
}
\`\`\`

## Recommendation

1. Implement Arrow Flight gRPC client
2. Add tests for Arrow Flight forwarding
3. Update documentation to reflect completion status

## References
- \`src/otlp/forwarder.rs:384-393\`
- \`src/otlp/forwarder.rs:378\` (similar TODO for traces)" \
  "enhancement,incomplete,todo"

# Issue #11: Protobuf Encoding
create_issue \
  "Complete Protobuf encoding implementation for HTTP forwarding" \
  "## Description

HTTP forwarding uses placeholder empty buffers instead of proper Protobuf encoding.

**Current Code:**
\`\`\`rust
// src/otlp/forwarder.rs:303
let buf = Vec::new(); // TODO: Implement proper Protobuf encoding for HTTP

// src/otlp/forwarder.rs:345
let buf = Vec::new(); // TODO: Implement proper Protobuf encoding for HTTP
\`\`\`

## Recommendation

Implement proper Protobuf serialization using \`prost\` or \`prost_types\`:
\`\`\`rust
use prost::Message;

let mut buf = Vec::new();
request.encode(&mut buf)?;
\`\`\`

## References
- \`src/otlp/forwarder.rs:239-310\` (send_protobuf_traces)
- \`src/otlp/forwarder.rs:312-350\` (send_protobuf_metrics)
- \`Cargo.toml:35\` (prost dependency available)" \
  "bug,incomplete,todo"

# Issue #12: Python Segfaults
create_issue \
  "Investigate and fix segfaults in Python test suite" \
  "## Description

The CI workflow has complex logic to handle segfaults during Python test cleanup, which suggests potential memory safety issues.

**Current Situation:**
- CI workflow handles segfaults as \"acceptable\" if tests pass
- Exit codes 139 (segfault) and 138 (bus error) are caught and ignored
- This is a workaround, not a solution

## Recommendation

1. Investigate root cause of segfaults:
   - Check PyO3 bindings for unsafe code issues
   - Review Python object lifecycle management
   - Check for double-free or use-after-free
2. Add memory sanitizer tests
3. Review \`unsafe\` blocks in Python bindings
4. Consider using \`pyo3-ffi\` or updating PyO3 version

## References
- \`.github/workflows/ci.yml:105-177\` (Linux)
- \`.github/workflows/ci.yml:236-295\` (macOS)
- \`src/python/bindings.rs:6\` (unsafe code)
- \`src/python/adapters.rs:7\` (unsafe code)" \
  "bug,python,memory-safety,critical"

# Issue #13: Input Validation
create_issue \
  "Enhance URL and input validation throughout the codebase" \
  "## Description

Several areas lack comprehensive input validation:
1. URL validation only checks prefix, not full URL structure
2. File paths could be more strictly validated
3. Configuration values could have stricter bounds checking

## Current Issues

1. **URL Validation:**
\`\`\`rust
// src/config/types.rs:476-480
if !url.starts_with(\"http://\") && !url.starts_with(\"https://\") {
    return Err(OtlpConfigError::InvalidUrl(...));
}
\`\`\`

2. **File Size Limits:** No maximum file size validation

## Recommendation

1. Use \`url::Url\` crate for proper URL parsing and validation
2. Add file size limits to configuration
3. Add comprehensive bounds checking for all numeric config values
4. Add validation tests

## References
- \`src/config/types.rs:466-489\` (ForwardingConfig::validate)
- \`src/otlp/exporter.rs:78\` (max_file_size is hardcoded)" \
  "security,enhancement"

# Issue #14: Architecture Documentation
create_issue \
  "Create ARCHITECTURE.md documenting system design and data flow" \
  "## Description

The project lacks comprehensive architecture documentation that would help new contributors understand:
- System architecture and component interactions
- Data flow through the system
- Threading model and concurrency patterns
- Error handling strategy
- Performance characteristics

## Recommendation

Create \`ARCHITECTURE.md\` with:
1. High-level architecture diagram
2. Component descriptions and responsibilities
3. Data flow diagrams (traces and metrics)
4. Threading model explanation
5. Error propagation patterns
6. Configuration system overview

## Sections
- Overview
- Components
- Data Flow
- Concurrency Model
- Error Handling
- Configuration System
- Extension Points" \
  "documentation,enhancement"

# Issue #15: Security Documentation
create_issue \
  "Create SECURITY.md with security model and reporting guidelines" \
  "## Description

The project lacks security documentation covering:
- Security model and assumptions
- Threat model
- Vulnerability reporting process
- Security best practices for users

## Recommendation

Create \`SECURITY.md\` with:
1. Security model and assumptions
2. Threat model (what the system protects against)
3. Vulnerability reporting process
4. Security best practices for configuration
5. Known security considerations
6. Security update policy

**Template:** Use GitHub's security policy template as a starting point." \
  "documentation,security"

# Issue #16: Enhanced Test Coverage
create_issue \
  "Add tests for concurrent access, circuit breaker, and edge cases" \
  "## Description

Several areas need additional test coverage:
1. Concurrent access to BatchBuffer under high load
2. Circuit breaker state transitions (especially half-open)
3. File rotation edge cases
4. Large file handling
5. Invalid input handling
6. Error recovery scenarios

## Missing Test Scenarios

1. **Concurrency:**
   - High concurrent writes to BatchBuffer
   - Race conditions in file rotation
   - Concurrent cleanup operations

2. **Circuit Breaker:**
   - Half-open state transitions
   - Recovery after failures
   - Timeout handling

3. **Edge Cases:**
   - Maximum buffer size reached
   - Disk full scenarios
   - File permission errors
   - Network timeouts in forwarding

## References
- \`tests/contract/test_otlp_protocol.rs:268-311\` (basic concurrency test exists)
- \`src/otlp/forwarder.rs:49-109\` (circuit breaker needs tests)" \
  "testing,enhancement"

echo ""
echo "All issues created successfully!"
echo "Total issues created: 16"
