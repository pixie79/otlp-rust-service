//! Integration tests for Python bindings of OtlpMetricExporter and OtlpSpanExporter

use otlp_arrow_library::{Config, OtlpLibrary};
use std::path::PathBuf;
use std::process::Command;
use tempfile::TempDir;

/// Helper function to create a Python test script
fn create_python_test_script(script_content: &str) -> String {
    format!(
        r#"
import sys
import os
import tempfile
import shutil

# Add the library to the path (assuming it's built)
sys.path.insert(0, os.path.join(os.path.dirname(__file__), "..", "..", "target", "debug"))

try:
    import otlp_arrow_library
    
    # Create temporary directory
    temp_dir = tempfile.mkdtemp()
    
    # Create library instance
    library = otlp_arrow_library.PyOtlpLibrary(
        output_dir=temp_dir,
        write_interval_secs=1
    )
    
    {}
    
    # Cleanup
    library.shutdown()
    shutil.rmtree(temp_dir)
    
    print("SUCCESS")
    sys.exit(0)
except Exception as e:
    print(f"ERROR: {{e}}")
    import traceback
    traceback.print_exc()
    sys.exit(1)
"#,
        script_content
    )
}

#[test]
fn test_python_metric_exporter_creation() {
    let temp_dir = TempDir::new().unwrap();
    let script_path = temp_dir.path().join("test_metric_exporter.py");
    
    let script = create_python_test_script(
        r#"
    # Test metric_exporter() method exists and can be called
    metric_exporter = library.metric_exporter()
    assert metric_exporter is not None, "metric_exporter() should return an object"
    "#,
    );
    
    std::fs::write(&script_path, script).unwrap();
    
    // Run Python script
    let output = Command::new("python3")
        .arg(script_path.to_str().unwrap())
        .output()
        .expect("Failed to execute Python script");
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    
    if !output.status.success() {
        eprintln!("Python script failed:");
        eprintln!("STDOUT: {}", stdout);
        eprintln!("STDERR: {}", stderr);
        panic!("Python test failed");
    }
    
    assert!(stdout.contains("SUCCESS"), "Python script should succeed");
}

#[test]
fn test_python_span_exporter_creation() {
    let temp_dir = TempDir::new().unwrap();
    let script_path = temp_dir.path().join("test_span_exporter.py");
    
    let script = create_python_test_script(
        r#"
    # Test span_exporter() method exists and can be called
    span_exporter = library.span_exporter()
    assert span_exporter is not None, "span_exporter() should return an object"
    "#,
    );
    
    std::fs::write(&script_path, script).unwrap();
    
    // Run Python script
    let output = Command::new("python3")
        .arg(script_path.to_str().unwrap())
        .output()
        .expect("Failed to execute Python script");
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    
    if !output.status.success() {
        eprintln!("Python script failed:");
        eprintln!("STDOUT: {}", stdout);
        eprintln!("STDERR: {}", stderr);
        panic!("Python test failed");
    }
    
    assert!(stdout.contains("SUCCESS"), "Python script should succeed");
}

#[test]
fn test_python_exporters_basic_usage() {
    let temp_dir = TempDir::new().unwrap();
    let script_path = temp_dir.path().join("test_exporters_usage.py");
    
    let script = create_python_test_script(
        r#"
    # Test that both exporters can be created from the same library
    metric_exporter = library.metric_exporter()
    span_exporter = library.span_exporter()
    
    assert metric_exporter is not None, "metric_exporter should be created"
    assert span_exporter is not None, "span_exporter should be created"
    
    # Test that exporters are different objects
    assert metric_exporter is not span_exporter, "Exporters should be different objects"
    
    # Test that multiple calls return new objects
    metric_exporter2 = library.metric_exporter()
    assert metric_exporter is not metric_exporter2, "Multiple calls should return different objects"
    "#,
    );
    
    std::fs::write(&script_path, script).unwrap();
    
    // Run Python script
    let output = Command::new("python3")
        .arg(script_path.to_str().unwrap())
        .output()
        .expect("Failed to execute Python script");
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    
    if !output.status.success() {
        eprintln!("Python script failed:");
        eprintln!("STDOUT: {}", stdout);
        eprintln!("STDERR: {}", stderr);
        panic!("Python test failed");
    }
    
    assert!(stdout.contains("SUCCESS"), "Python script should succeed");
}

