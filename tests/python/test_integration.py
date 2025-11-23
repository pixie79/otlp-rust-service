"""Integration test for end-to-end usage"""

import tempfile
import os
import pytest
import time


def test_end_to_end_workflow():
    """Test complete end-to-end workflow"""
    import otlp_arrow_library
    
    with tempfile.TemporaryDirectory() as tmpdir:
        # Initialize library with custom configuration
        library = otlp_arrow_library.PyOtlpLibrary(
            output_dir=tmpdir,
            write_interval_secs=1,
            trace_cleanup_interval_secs=600,
            metric_cleanup_interval_secs=3600
        )
        
        # Export multiple traces
        spans = []
        for i in range(5):
            trace_id = bytes([i] * 16)
            span_id = bytes([i] * 8)
            span_dict = {
                "trace_id": trace_id,
                "span_id": span_id,
                "name": f"integration-test-span-{i}",
                "kind": "server",
                "attributes": {
                    "service.name": "integration-test-service",
                    "test.id": str(i)
                }
            }
            spans.append(span_dict)
        
        library.export_traces(spans)
        
        # Export metrics
        metrics_dict = {
            "resource": {},
            "scope_metrics": []
        }
        library.export_metrics(metrics_dict)
        
        # Flush to ensure everything is written
        library.flush()
        
        # Verify trace files were created
        traces_dir = os.path.join(tmpdir, "otlp", "traces")
        trace_files = os.listdir(traces_dir)
        assert len(trace_files) > 0, "Expected trace files to be created"
        
        # Verify metrics directory exists
        metrics_dir = os.path.join(tmpdir, "otlp", "metrics")
        assert os.path.exists(metrics_dir), "Expected metrics directory to exist"
        
        # Cleanup
        library.shutdown()


if __name__ == "__main__":
    pytest.main([__file__])

