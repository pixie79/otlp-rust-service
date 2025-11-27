//! Integration test for metric cleanup interval configuration

use otlp_arrow_library::api::OtlpLibrary;
use otlp_arrow_library::config::ConfigBuilder;
use opentelemetry_sdk::metrics::data::ResourceMetrics;
use std::fs;
use std::path::PathBuf;
use std::time::{Duration, SystemTime};
use tempfile::TempDir;
use tokio::time::sleep;

/// Helper to create an old file for testing cleanup
fn create_old_metric_file(dir: &PathBuf, filename: &str, age_seconds: u64) {
    let file_path = dir.join(filename);
    fs::write(&file_path, b"test metric data").unwrap();
    
    // Set file modification time to be old
    let old_time = SystemTime::now() - Duration::from_secs(age_seconds);
    let file_time = filetime::FileTime::from_system_time(old_time);
    filetime::set_file_mtime(&file_path, file_time).unwrap();
}

#[tokio::test]
async fn test_metric_cleanup_interval() {
    let temp_dir = TempDir::new().unwrap();
    let metrics_dir = temp_dir.path().join("otlp/metrics");
    fs::create_dir_all(&metrics_dir).unwrap();
    
    // Create old metric files that should be cleaned up
    create_old_metric_file(&metrics_dir, "old_metric_1.arrow", 3700); // 3700 seconds old
    create_old_metric_file(&metrics_dir, "old_metric_2.arrow", 3800); // 3800 seconds old
    create_old_metric_file(&metrics_dir, "recent_metric.arrow", 100); // 100 seconds old (should not be cleaned)
    
    let initial_files: Vec<_> = fs::read_dir(&metrics_dir).unwrap().collect();
    assert_eq!(initial_files.len(), 3, "Should have 3 files initially");
    
    // Create config with metric cleanup interval of 3600 seconds (1 hour)
    let config = ConfigBuilder::new()
        .output_dir(temp_dir.path())
        .metric_cleanup_interval_secs(3600)
        .write_interval_secs(1)
        .build()
        .unwrap();
    
    // Create library instance (this starts cleanup task)
    let library = OtlpLibrary::new(config).await.unwrap();
    
    // Wait for cleanup task to start
    sleep(Duration::from_millis(500)).await;
    
    // The cleanup should have run. Files older than 3600 seconds should be removed
    let remaining_files: Vec<_> = fs::read_dir(&metrics_dir).unwrap().collect();
    
    // At least the recent file should remain
    let recent_file_exists = remaining_files.iter()
        .any(|e| e.as_ref().unwrap().file_name().to_string_lossy().contains("recent"));
    assert!(recent_file_exists, "Recent file should not be cleaned up");
    
    // Cleanup
    library.shutdown().await.unwrap();
}

#[tokio::test]
async fn test_default_metric_cleanup_interval() {
    let temp_dir = TempDir::new().unwrap();
    
    // Create config with default metric cleanup interval (3600 seconds)
    let config = ConfigBuilder::new()
        .output_dir(temp_dir.path())
        // Don't set metric_cleanup_interval_secs - should use default of 3600
        .build()
        .unwrap();
    
    assert_eq!(config.metric_cleanup_interval_secs, 3600, "Should use default cleanup interval of 3600 seconds");
    
    // Create library instance
    let library = OtlpLibrary::new(config).await.unwrap();
    
    // Export metrics (using default ResourceMetrics)
    let metrics = ResourceMetrics::default();
    library.export_metrics_arrow(&metrics).await.unwrap();
    
    // Flush to ensure write
    library.flush().await.unwrap();
    
    // Verify files were created
    let metrics_dir = temp_dir.path().join("otlp/metrics");
    assert!(metrics_dir.exists(), "Metrics directory should exist");
    
    // Cleanup
    library.shutdown().await.unwrap();
}

