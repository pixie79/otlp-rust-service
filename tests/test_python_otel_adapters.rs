//! Integration/contract tests for Python OpenTelemetry SDK adapters
//!
//! Tests that adapters correctly implement the contract with Python OpenTelemetry SDK

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
# Try multiple possible locations for the built module
import pathlib
script_dir = pathlib.Path(__file__).parent.absolute()
project_root = script_dir.parent.parent

possible_paths = [
    project_root / "target" / "debug",
    project_root / "target" / "release",
    project_root,
]

for path in possible_paths:
    path_str = str(path)
    if os.path.exists(path_str):
        sys.path.insert(0, path_str)
        # Also check for .so/.dylib files in the directory
        for file in os.listdir(path_str):
            if file.startswith("otlp_arrow_library") and (file.endswith(".so") or file.endswith(".dylib")):
                # Module is in this directory
                break

try:
    import otlp_arrow_library
except ImportError:
    # Module not available (e.g., not built in CI before cargo test)
    # This is expected in some CI environments
    print("SKIP: otlp_arrow_library module not available")
    sys.exit(0)

# Create temporary directory
temp_dir = tempfile.mkdtemp()

try:
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
fn test_metric_exporter_contract() {
    let temp_dir = TempDir::new().unwrap();
    let script_path = temp_dir.path().join("test_metric_exporter_contract.py");

    let script = create_python_test_script(
        r#"
    # Test metric_exporter_adapter() method exists and can be called
    metric_exporter = library.metric_exporter_adapter()
    assert metric_exporter is not None, "metric_exporter_adapter() should return an object"
    
    # Test that adapter has required methods
    assert hasattr(metric_exporter, "export"), "Adapter should have export() method"
    assert hasattr(metric_exporter, "shutdown"), "Adapter should have shutdown() method"
    assert hasattr(metric_exporter, "force_flush"), "Adapter should have force_flush() method"
    assert hasattr(metric_exporter, "temporality"), "Adapter should have temporality() method"
    
    # Test shutdown (should be no-op)
    result = metric_exporter.shutdown()
    assert result is None, "shutdown() should return None"
    
    # Test force_flush
    flush_result = metric_exporter.force_flush(timeout_millis=1000)
    assert flush_result is not None, "force_flush() should return a result"
    
    # Test temporality
    temporality = metric_exporter.temporality()
    assert temporality is not None, "temporality() should return a value"
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

    // Check if test was skipped (module not available)
    if stdout.contains("SKIP:") {
        eprintln!("Test skipped: {}", stdout);
        return; // Skip this test
    }

    if !output.status.success() {
        eprintln!("Python script failed:");
        eprintln!("STDOUT: {}", stdout);
        eprintln!("STDERR: {}", stderr);
        panic!("Python test failed");
    }

    assert!(stdout.contains("SUCCESS"), "Python script should succeed");
}

#[test]
fn test_span_exporter_contract() {
    let temp_dir = TempDir::new().unwrap();
    let script_path = temp_dir.path().join("test_span_exporter_contract.py");

    let script = create_python_test_script(
        r#"
    # Test span_exporter_adapter() method exists and can be called
    span_exporter = library.span_exporter_adapter()
    assert span_exporter is not None, "span_exporter_adapter() should return an object"
    
    # Test that adapter has required methods
    assert hasattr(span_exporter, "export"), "Adapter should have export() method"
    assert hasattr(span_exporter, "shutdown"), "Adapter should have shutdown() method"
    assert hasattr(span_exporter, "force_flush"), "Adapter should have force_flush() method"
    
    # Test shutdown (should be no-op)
    result = span_exporter.shutdown()
    assert result is None, "shutdown() should return None"
    
    # Test force_flush
    flush_result = span_exporter.force_flush(timeout_millis=1000)
    assert flush_result is not None, "force_flush() should return a result"
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

    // Check if test was skipped (module not available)
    if stdout.contains("SKIP:") {
        eprintln!("Test skipped: {}", stdout);
        return; // Skip this test
    }

    if !output.status.success() {
        eprintln!("Python script failed:");
        eprintln!("STDOUT: {}", stdout);
        eprintln!("STDERR: {}", stderr);
        panic!("Python test failed");
    }

    assert!(stdout.contains("SUCCESS"), "Python script should succeed");
}
