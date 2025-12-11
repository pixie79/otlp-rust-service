//! Common benchmark utilities and helpers
//!
//! Provides shared functionality for benchmark tests including setup, teardown,
//! and measurement utilities.

use std::time::{Duration, Instant};

/// Benchmark configuration for consistent test setup
pub struct BenchmarkConfig {
    pub iterations: usize,
    pub warmup_iterations: usize,
    pub concurrency_level: usize,
}

impl Default for BenchmarkConfig {
    fn default() -> Self {
        Self {
            iterations: 1000,
            warmup_iterations: 100,
            concurrency_level: 10,
        }
    }
}

/// Measure lock acquisition frequency
pub struct LockAcquisitionCounter {
    count: std::sync::atomic::AtomicU64,
}

impl LockAcquisitionCounter {
    pub fn new() -> Self {
        Self {
            count: std::sync::atomic::AtomicU64::new(0),
        }
    }

    pub fn increment(&self) {
        self.count.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    }

    pub fn get(&self) -> u64 {
        self.count.load(std::sync::atomic::Ordering::Relaxed)
    }

    pub fn reset(&self) {
        self.count.store(0, std::sync::atomic::Ordering::Relaxed);
    }
}

impl Default for LockAcquisitionCounter {
    fn default() -> Self {
        Self::new()
    }
}

/// Measure throughput (operations per second)
pub fn measure_throughput<F>(iterations: usize, operation: F) -> f64
where
    F: Fn(),
{
    let start = Instant::now();
    for _ in 0..iterations {
        operation();
    }
    let elapsed = start.elapsed();
    iterations as f64 / elapsed.as_secs_f64()
}

/// Measure average latency
pub fn measure_latency<F>(iterations: usize, operation: F) -> Duration
where
    F: Fn(),
{
    let mut total = Duration::ZERO;
    for _ in 0..iterations {
        let start = Instant::now();
        operation();
        total += start.elapsed();
    }
    total / iterations as u32
}
