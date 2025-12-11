# Performance API Contract: Benchmarking and Optimization Validation

**Feature**: 007-foundation-optimizations  
**Date**: 2025-01-27  
**Status**: Complete

## Overview

This contract defines the benchmarking API and performance validation requirements for optimizations.

## Benchmark Structure

### Benchmark Files

**Location**: `tests/bench/`

**Files**:
- `bench_circuit_breaker.rs`: Circuit breaker lock contention benchmarks
- `bench_batch_buffer.rs`: BatchBuffer throughput benchmarks
- `bench_exporter.rs`: Exporter performance benchmarks

---

## Benchmark API

### Criterion Benchmark Pattern

**Pattern**:
```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_circuit_breaker_lock_acquisition(c: &mut Criterion) {
    let breaker = CircuitBreaker::new(5, Duration::from_secs(60));
    
    c.bench_function("circuit_breaker_state_update", |b| {
        b.to_async(tokio::runtime::Runtime::new().unwrap())
            .iter(|| async {
                breaker.update_state(/* ... */).await
            });
    });
}

criterion_group!(benches, bench_circuit_breaker_lock_acquisition);
criterion_main!(benches);
```

---

## Performance Metrics

### Circuit Breaker Metrics

**Metrics**:
- Lock acquisition frequency (operations per second)
- State transition time (nanoseconds)
- Concurrent access throughput (operations per second)

**Targets**:
- Lock acquisition frequency reduced by at least 50%
- No increase in state transition time
- Throughput maintained or improved

---

### BatchBuffer Metrics

**Metrics**:
- Throughput (operations per second)
- Lock contention time (nanoseconds)
- Concurrent access latency (nanoseconds)

**Targets**:
- Throughput improved by at least 20% under high concurrency
- Lock contention reduced
- Latency maintained or improved

---

### Exporter Metrics

**Metrics**:
- Throughput (exports per second)
- Memory allocations per export
- CPU time per export

**Targets**:
- Throughput improved by at least 15%
- Memory allocations reduced
- CPU time reduced or maintained

---

## Benchmark Execution

### Running Benchmarks

```bash
# Run all benchmarks
cargo bench

# Run specific benchmark
cargo bench --bench bench_circuit_breaker

# Run with output
cargo bench -- --nocapture
```

### CI/CD Integration

**Requirements**:
- Benchmarks must run in CI/CD pipeline
- Performance regressions must be detected
- Regression threshold: 5% maximum allowed

**CI Configuration**:
```yaml
- name: Run benchmarks
  run: cargo bench -- --output-format json > bench_results.json

- name: Compare benchmarks
  run: scripts/compare_benchmarks.sh bench_results.json baseline.json
```

---

## Performance Validation

### Before/After Comparison

**Process**:
1. Run benchmarks before optimization (baseline)
2. Implement optimization
3. Run benchmarks after optimization
4. Compare results
5. Validate improvement targets met

**Validation Rules**:
- Improvement targets must be met (50% reduction, 20% improvement, etc.)
- No regressions allowed (threshold: 5% maximum)
- Correctness must be maintained (all tests pass)

---

## Profiling Tools

### Lock Contention Analysis

**Tools**:
- `cargo-flamegraph`: Generate flamegraphs for lock analysis
- `perf` (Linux): Profile lock contention
- `instruments` (macOS): Profile lock contention

**Usage**:
```bash
# Generate flamegraph
cargo flamegraph --bench bench_circuit_breaker

# Profile with perf (Linux)
perf record --call-graph dwarf cargo bench --bench bench_circuit_breaker
perf report
```

---

## Performance Targets Summary

| Component | Metric | Target | Validation |
|-----------|--------|--------|------------|
| Circuit Breaker | Lock acquisition frequency | 50% reduction | Profiling |
| BatchBuffer | Throughput | 20% improvement | Benchmarking |
| Exporter | Throughput | 15% improvement | Benchmarking |
| All | Correctness | 100% maintained | Test suite |

---

## Notes

- Benchmarks must be deterministic (no random delays)
- Performance improvements must be measurable
- All optimizations must maintain correctness
- CI/CD must catch performance regressions
