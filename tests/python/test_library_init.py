"""Unit test for library initialization"""

import tempfile
import os
import pytest


def test_library_init_default():
    """Test library initialization with default configuration"""
    import otlp_arrow_library
    
    library = otlp_arrow_library.PyOtlpLibrary()
    assert library is not None
    
    # Cleanup
    library.shutdown()


def test_library_init_custom_output_dir():
    """Test library initialization with custom output directory"""
    import otlp_arrow_library
    
    with tempfile.TemporaryDirectory() as tmpdir:
        library = otlp_arrow_library.PyOtlpLibrary(output_dir=tmpdir)
        assert library is not None
        
        # Verify output directory was created
        assert os.path.exists(os.path.join(tmpdir, "otlp", "traces"))
        assert os.path.exists(os.path.join(tmpdir, "otlp", "metrics"))
        
        # Cleanup
        library.shutdown()


def test_library_init_custom_intervals():
    """Test library initialization with custom intervals"""
    import otlp_arrow_library
    
    library = otlp_arrow_library.PyOtlpLibrary(
        write_interval_secs=10,
        trace_cleanup_interval_secs=1200,
        metric_cleanup_interval_secs=7200
    )
    assert library is not None
    
    # Cleanup
    library.shutdown()


def test_library_init_protocol_config():
    """Test library initialization with protocol configuration"""
    import otlp_arrow_library
    
    library = otlp_arrow_library.PyOtlpLibrary(
        protobuf_enabled=True,
        protobuf_port=4317,
        arrow_flight_enabled=True,
        arrow_flight_port=4318
    )
    assert library is not None
    
    # Cleanup
    library.shutdown()


if __name__ == "__main__":
    pytest.main([__file__])

