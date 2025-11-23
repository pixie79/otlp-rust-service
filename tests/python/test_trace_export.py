"""Unit test for trace export"""

import tempfile
import os
import pytest
import time


def test_export_single_trace():
    """Test exporting a single trace span"""
    import otlp_arrow_library
    
    with tempfile.TemporaryDirectory() as tmpdir:
        library = otlp_arrow_library.PyOtlpLibrary(
            output_dir=tmpdir,
            write_interval_secs=1
        )
        
        # Create a test span
        trace_id = bytes([1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16])
        span_id = bytes([1, 2, 3, 4, 5, 6, 7, 8])
        
        span_dict = {
            "trace_id": trace_id,
            "span_id": span_id,
            "name": "test-span",
            "kind": "server",
            "attributes": {
                "service.name": "test-service",
                "http.method": "GET"
            }
        }
        
        # Export the span
        library.export_trace(span_dict)
        
        # Flush to ensure it's written
        library.flush()
        
        # Verify file was created
        traces_dir = os.path.join(tmpdir, "otlp", "traces")
        files = os.listdir(traces_dir)
        assert len(files) > 0, "Expected at least one trace file to be created"
        
        # Cleanup
        library.shutdown()


def test_export_multiple_traces():
    """Test exporting multiple trace spans"""
    import otlp_arrow_library
    
    with tempfile.TemporaryDirectory() as tmpdir:
        library = otlp_arrow_library.PyOtlpLibrary(
            output_dir=tmpdir,
            write_interval_secs=1
        )
        
        # Create multiple test spans
        spans = []
        for i in range(3):
            trace_id = bytes([i] * 16)
            span_id = bytes([i] * 8)
            span_dict = {
                "trace_id": trace_id,
                "span_id": span_id,
                "name": f"test-span-{i}",
                "kind": "internal",
            }
            spans.append(span_dict)
        
        # Export the spans
        library.export_traces(spans)
        
        # Flush to ensure they're written
        library.flush()
        
        # Verify file was created
        traces_dir = os.path.join(tmpdir, "otlp", "traces")
        files = os.listdir(traces_dir)
        assert len(files) > 0, "Expected at least one trace file to be created"
        
        # Cleanup
        library.shutdown()


if __name__ == "__main__":
    pytest.main([__file__])

