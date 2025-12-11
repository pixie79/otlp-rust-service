//! Test helper utilities for concurrent test patterns
//!
//! Provides utilities for testing concurrent access scenarios, including
//! helpers for spawning concurrent tasks, synchronization barriers, and
//! validation utilities.

use std::sync::Arc;
use tokio::sync::Barrier;
use tokio::task::JoinSet;

/// Spawn multiple concurrent tasks and wait for all to complete
pub async fn spawn_concurrent_tasks<F, Fut>(
    count: usize,
    task_fn: F,
) -> Vec<Result<tokio::task::JoinHandle<()>, tokio::task::JoinError>>
where
    F: Fn(usize) -> Fut + Send + Sync + 'static,
    Fut: std::future::Future<Output = ()> + Send + 'static,
{
    let mut handles = Vec::new();
    for i in 0..count {
        let handle = tokio::spawn(async move {
            task_fn(i).await;
        });
        handles.push(handle);
    }
    handles
}

/// Spawn concurrent tasks with a barrier for synchronization
pub async fn spawn_concurrent_tasks_with_barrier<F, Fut>(
    count: usize,
    task_fn: F,
) -> Vec<Result<(), tokio::task::JoinError>>
where
    F: Fn(usize, Arc<Barrier>) -> Fut + Send + Sync + 'static,
    Fut: std::future::Future<Output = ()> + Send + 'static,
{
    let barrier = Arc::new(Barrier::new(count));
    let mut handles = Vec::new();
    
    for i in 0..count {
        let barrier_clone = barrier.clone();
        let handle = tokio::spawn(async move {
            task_fn(i, barrier_clone).await;
        });
        handles.push(handle);
    }
    
    let mut results = Vec::new();
    for handle in handles {
        results.push(handle.await);
    }
    results
}

/// Spawn concurrent tasks using JoinSet for better control
pub async fn spawn_concurrent_tasks_joinset<F, Fut>(
    count: usize,
    task_fn: F,
) -> Result<(), tokio::task::JoinError>
where
    F: Fn(usize) -> Fut + Send + Sync + 'static,
    Fut: std::future::Future<Output = ()> + Send + 'static,
{
    let mut join_set = JoinSet::new();
    
    for i in 0..count {
        join_set.spawn(async move {
            task_fn(i).await;
        });
    }
    
    while let Some(result) = join_set.join_next().await {
        result?;
    }
    
    Ok(())
}

/// Validate that all results are successful
pub fn validate_all_success<T, E>(results: Vec<Result<T, E>>) -> bool {
    results.iter().all(|r| r.is_ok())
}

/// Validate that final state is consistent
pub async fn validate_state_consistency<F, Fut>(validation_fn: F) -> bool
where
    F: Fn() -> Fut,
    Fut: std::future::Future<Output = bool>,
{
    validation_fn().await
}
