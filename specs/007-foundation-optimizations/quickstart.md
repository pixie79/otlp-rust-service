# Quickstart: Foundation Optimizations and Quality Improvements

**Feature**: 007-foundation-optimizations  
**Date**: 2025-01-27  
**Status**: Complete

## Overview

This quickstart guide helps developers understand and work with the foundational improvements and performance optimizations implemented in this feature.

## Running Tests

### Concurrent Access Tests

Test concurrent access to BatchBuffer:

```bash
# Run concurrent access tests
cargo test --test test_batch_buffer_concurrent

# Run with output
cargo test --test test_batch_buffer_concurrent -- --nocapture
```

**What it tests**:
- Multiple concurrent writers accessing BatchBuffer
- Data integrity under concurrent access
- Lock contention behavior

---

### Circuit Breaker Tests

Test circuit breaker state transitions:

```bash
# Run circuit breaker tests
cargo test --test test_circuit_breaker

# Run with output
cargo test --test test_circuit_breaker -- --nocapture
```

**What it tests**:
- All state transitions (Closed → Open → HalfOpen → Closed)
- Concurrent access during state transitions
- Timeout-based transitions

---

### Edge Case Tests

Test edge cases:

```bash
# Run edge case tests
cargo test --test test_edge_cases

# Run with output
cargo test --test test_edge_cases -- --nocapture
```

**What it tests**:
- Buffer capacity limits
- File rotation race conditions
- Error recovery scenarios

---

## Running Benchmarks

### Circuit Breaker Benchmarks

Benchmark circuit breaker lock performance:

```bash
# Run circuit breaker benchmarks
cargo bench --bench bench_circuit_breaker

# Generate flamegraph
cargo flamegraph --bench bench_circuit_breaker
```

**What it measures**:
- Lock acquisition frequency
- State transition time
- Concurrent access throughput

---

### BatchBuffer Benchmarks

Benchmark BatchBuffer throughput:

```bash
# Run BatchBuffer benchmarks
cargo bench --bench bench_batch_buffer

# Generate flamegraph
cargo flamegraph --bench bench_batch_buffer
```

**What it measures**:
- Throughput (operations per second)
- Lock contention time
- Concurrent access latency

---

### Exporter Benchmarks

Benchmark exporter performance:

```bash
# Run exporter benchmarks
cargo bench --bench bench_exporter

# Generate flamegraph
cargo flamegraph --bench bench_exporter
```

**What it measures**:
- Throughput (exports per second)
- Memory allocations per export
- CPU time per export

---

## Using Temporality Configuration

### Rust API

Configure temporality for metric exporters:

```rust
use opentelemetry_sdk::metrics::data::Temporality;

// Create exporter with Cumulative temporality (default)
let exporter_cumulative = OtlpMetricExporterBuilder::new()
    .with_temporality(Temporality::Cumulative)
    .build();

// Create exporter with Delta temporality
let exporter_delta = OtlpMetricExporterBuilder::new()
    .with_temporality(Temporality::Delta)
    .build();

// Use default (Cumulative) - no change needed
let exporter_default = OtlpMetricExporterBuilder::new()
    .build();
```

---

### Python API

Configure temporality for metric exporters:

```python
from opentelemetry.sdk.metrics.export import Temporality

# Create exporter with Cumulative temporality (default)
exporter_cumulative = library.metric_exporter()
exporter_cumulative.set_temporality(Temporality.CUMULATIVE)

# Create exporter with Delta temporality
exporter_delta = library.metric_exporter()
exporter_delta.set_temporality(Temporality.DELTA)

# Use default (Cumulative) - no change needed
exporter_default = library.metric_exporter()
```

---

## Viewing Architecture Documentation

Read the architecture documentation:

```bash
# View ARCHITECTURE.md
cat docs/ARCHITECTURE.md

# Or open in editor
code docs/ARCHITECTURE.md
```

**What it contains**:
- System overview and high-level architecture
- Data flow diagrams and descriptions
- Component interaction patterns
- Key design decisions and rationale
- Technology stack information

---

## Verifying Optimizations

### Check Lock Optimization

Verify circuit breaker lock optimization:

```bash
# Profile lock acquisition
cargo flamegraph --bench bench_circuit_breaker

# Compare before/after
# (Run benchmarks before and after optimization)
cargo bench --bench bench_circuit_breaker > before.txt
# ... implement optimization ...
cargo bench --bench bench_circuit_breaker > after.txt
# Compare results
```

**Expected improvement**: Lock acquisition frequency reduced by at least 50%

---

### Check BatchBuffer Optimization

Verify BatchBuffer optimization:

```bash
# Profile BatchBuffer throughput
cargo bench --bench bench_batch_buffer

# Compare before/after
cargo bench --bench bench_batch_buffer > before.txt
# ... implement optimization ...
cargo bench --bench bench_batch_buffer > after.txt
# Compare results
```

**Expected improvement**: Throughput improved by at least 20% under high concurrency

---

### Check Exporter Optimization

Verify exporter optimization:

```bash
# Profile exporter performance
cargo bench --bench bench_exporter

# Compare before/after
cargo bench --bench bench_exporter > before.txt
# ... implement optimization ...
cargo bench --bench bench_exporter > after.txt
# Compare results
```

**Expected improvement**: Throughput improved by at least 15% without increasing resource usage

---

## Running All Tests

Run the complete test suite:

```bash
# Run all tests
cargo test --all-features --workspace

# Run with coverage
cargo tarpaulin --workspace --all-features

# Run benchmarks
cargo bench
```

---

## Next Steps

1. **Review Architecture Documentation**: Read `docs/ARCHITECTURE.md` to understand system design
2. **Run Tests**: Execute test suite to verify functionality
3. **Run Benchmarks**: Validate performance improvements
4. **Configure Temporality**: Use temporality configuration if needed
5. **Profile Optimizations**: Use profiling tools to verify improvements

---

## Troubleshooting

### Tests Failing

- Check that all dependencies are installed
- Verify Rust version is up-to-date
- Run `cargo clean` and rebuild
- Check test output for specific error messages

### Benchmarks Not Running

- Ensure `criterion` is added to `Cargo.toml` dev-dependencies
- Check that benchmark files are in `tests/bench/` directory
- Verify benchmark function signatures are correct

### Performance Not Improving

- Profile with `cargo-flamegraph` to identify bottlenecks
- Verify optimizations are actually applied
- Check that benchmarks are measuring the right metrics
- Compare before/after results carefully

---

## Additional Resources

- **Test API Contract**: `contracts/test-api.md`
- **Configuration API Contract**: `contracts/configuration-api.md`
- **Performance API Contract**: `contracts/performance-api.md`
- **Data Model**: `data-model.md`
- **Research**: `research.md`
