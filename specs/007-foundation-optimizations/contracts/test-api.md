# Test API Contract: Foundation Optimizations

**Feature**: 007-foundation-optimizations  
**Date**: 2025-01-27  
**Status**: Complete

## Overview

This contract defines the structure and patterns for test implementations covering concurrent access, circuit breaker state transitions, and edge cases.

## Test Structure

### Concurrent Access Tests

**Location**: `tests/unit/otlp/test_batch_buffer_concurrent.rs`, `tests/integration/test_concurrent_access.rs`

**Pattern**:
```rust
#[tokio::test]
async fn test_concurrent_batch_buffer_access() {
    // Setup
    let buffer = BatchBuffer::new(/* ... */);
    let concurrency_level = 100;
    
    // Spawn concurrent tasks
    let mut handles = Vec::new();
    for i in 0..concurrency_level {
        let buffer_clone = buffer.clone();
        let handle = tokio::spawn(async move {
            buffer_clone.add_trace(/* ... */).await
        });
        handles.push(handle);
    }
    
    // Wait for all tasks
    let results: Vec<_> = futures::future::join_all(handles).await
        .into_iter()
        .map(|r| r.unwrap())
        .collect();
    
    // Validate
    assert!(results.iter().all(|r| r.is_ok()));
    assert_eq!(buffer.trace_count().await, concurrency_level);
}
```

**Requirements**:
- Use `tokio::spawn` for concurrent task execution
- Use `futures::future::join_all` or `JoinSet` for waiting on all tasks
- Validate final state consistency
- Test with multiple concurrency levels (10, 100, 1000)

---

### Circuit Breaker State Transition Tests

**Location**: `tests/unit/otlp/test_circuit_breaker.rs`

**Pattern**:
```rust
#[tokio::test]
async fn test_circuit_breaker_state_transitions() {
    // Setup
    let breaker = CircuitBreaker::new(5, Duration::from_secs(60));
    
    // Test Closed → Open transition
    for _ in 0..5 {
        breaker.call(|| async { Err(/* ... */) }).await;
    }
    assert_eq!(breaker.state().await, CircuitState::Open);
    
    // Test Open → HalfOpen transition (with mock time)
    tokio::time::advance(Duration::from_secs(31)).await;
    // Trigger transition check
    // ...
    assert_eq!(breaker.state().await, CircuitState::HalfOpen);
    
    // Test HalfOpen → Closed (success)
    breaker.call(|| async { Ok(()) }).await;
    assert_eq!(breaker.state().await, CircuitState::Closed);
}
```

**Requirements**:
- Test all state transitions explicitly
- Use mock time for timeout-based transitions
- Test concurrent access during state transitions
- Verify state consistency after operations

---

### Edge Case Tests

**Location**: `tests/contract/test_edge_cases.rs`

**Pattern**:
```rust
#[tokio::test]
async fn test_buffer_capacity_limit() {
    let buffer = BatchBuffer::new(/* max_size = 100 */);
    
    // Fill buffer to capacity
    for i in 0..100 {
        buffer.add_trace(/* ... */).await.unwrap();
    }
    
    // Attempt to exceed capacity
    let result = buffer.add_trace(/* ... */).await;
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), OtlpError::Export(OtlpExportError::BufferFull)));
}
```

**Requirements**:
- Test buffer capacity limits
- Test file rotation race conditions
- Test error recovery scenarios
- Test invalid input handling

---

## Test Coverage Requirements

### Coverage Targets

- **Concurrent Access**: 100% of concurrent scenarios covered
- **Circuit Breaker**: All state transitions covered (100%)
- **Edge Cases**: All identified edge cases covered
- **Overall**: Maintain 85% code coverage per file

### Test Types

1. **Unit Tests**: Fast, isolated tests for individual components
2. **Integration Tests**: Tests for component interactions
3. **Contract Tests**: Tests for protocol/API contracts
4. **Performance Tests**: Benchmarks for performance validation

---

## Test Execution

### Running Tests

```bash
# Run all tests
cargo test

# Run specific test file
cargo test --test test_batch_buffer_concurrent

# Run with output
cargo test -- --nocapture

# Run benchmarks
cargo bench
```

### Test Organization

```
tests/
├── unit/otlp/
│   ├── test_circuit_breaker.rs
│   ├── test_batch_buffer_concurrent.rs
│   └── test_exporter_temporality.rs
├── integration/
│   ├── test_concurrent_access.rs
│   └── test_circuit_breaker_recovery.rs
├── contract/
│   └── test_edge_cases.rs
└── bench/
    ├── bench_circuit_breaker.rs
    ├── bench_batch_buffer.rs
    └── bench_exporter.rs
```

---

## Validation

### Test Validation Rules

1. All tests must pass with 100% success rate
2. Tests must be deterministic (no flaky tests)
3. Tests must be independent (can run in parallel)
4. Tests must be fast (<100ms each for unit tests)
5. Tests must validate both success and failure cases

### Coverage Validation

- Code coverage must not decrease below 85% per file
- New code must have corresponding tests
- Edge cases must be explicitly tested
- Concurrent scenarios must be validated
