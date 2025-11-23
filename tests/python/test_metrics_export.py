"""Unit test for metrics export"""

import tempfile
import os
import pytest


def test_export_metrics():
    """Test exporting metrics"""
    import otlp_arrow_library
    
    with tempfile.TemporaryDirectory() as tmpdir:
        library = otlp_arrow_library.PyOtlpLibrary(
            output_dir=tmpdir,
            write_interval_secs=1
        )
        
        # Create a minimal metrics dict
        # Note: Full metrics conversion is complex, this is a placeholder
        metrics_dict = {
            "resource": {},
            "scope_metrics": []
        }
        
        # Export the metrics
        library.export_metrics(metrics_dict)
        
        # Flush to ensure it's written
        library.flush()
        
        # Note: Full metrics verification would require proper metrics structure
        # For now, we just verify the call doesn't crash
        
        # Cleanup
        library.shutdown()


if __name__ == "__main__":
    pytest.main([__file__])

