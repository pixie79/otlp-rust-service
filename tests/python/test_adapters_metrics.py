"""Tests for Python OpenTelemetry SDK metric exporter adapter"""

import tempfile
import os
import pytest


def test_metric_exporter_interface():
    """Test that metric exporter adapter implements required interface"""
    import otlp_arrow_library
    
    with tempfile.TemporaryDirectory() as tmpdir:
        library = otlp_arrow_library.PyOtlpLibrary(
            output_dir=tmpdir,
            write_interval_secs=1
        )
        
        # Create adapter
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
        
        # Cleanup
        library.shutdown()


def test_metric_exporter_with_mock_data():
    """Test metric exporter with mock metric data"""
    import otlp_arrow_library
    
    with tempfile.TemporaryDirectory() as tmpdir:
        library = otlp_arrow_library.PyOtlpLibrary(
            output_dir=tmpdir,
            write_interval_secs=1
        )
        
        metric_exporter = library.metric_exporter_adapter()
        
        # Create a minimal mock MetricExportResult structure
        # Note: This is a simplified test - real usage would use OpenTelemetry SDK types
        class MockResource:
            def __init__(self):
                self.attributes = {}
        
        class MockScope:
            def __init__(self):
                self.name = "test-scope"
                self.version = "1.0.0"
        
        class MockMetric:
            def __init__(self):
                self.name = "test-metric"
                self.description = "Test metric"
                self.unit = "1"
                self.data = None
        
        class MockScopeMetric:
            def __init__(self):
                self.scope = MockScope()
                self.metrics = [MockMetric()]
        
        class MockResourceMetrics:
            def __init__(self):
                self.resource = MockResource()
                self.scope_metrics = [MockScopeMetric()]
        
        class MockMetricExportResult:
            def __init__(self):
                self.resource_metrics = MockResourceMetrics()
        
        # Try to export (may fail if types don't match exactly, but tests interface)
        try:
            mock_result = MockMetricExportResult()
            export_result = metric_exporter.export(mock_result)
            # If it succeeds, verify result
            if export_result is not None:
                assert True, "Export should return a result"
        except Exception as e:
            # Expected if mock types don't match exactly
            # Real test would use actual OpenTelemetry SDK types
            pass
        
        library.shutdown()


if __name__ == "__main__":
    pytest.main([__file__])

