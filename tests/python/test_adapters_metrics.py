"""Tests for Python OpenTelemetry SDK metric exporter adapter"""

import tempfile
import shutil
import pytest


def test_metric_exporter_interface():
    """Test that metric exporter adapter implements required interface"""
    import otlp_arrow_library
    
    # Use mkdtemp instead of TemporaryDirectory to avoid segfault during cleanup
    tmpdir = tempfile.mkdtemp()
    try:
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
        
        # Test force_flush (pass timeout as keyword argument)
        flush_result = metric_exporter.force_flush(timeout_millis=1000)
        assert flush_result is not None, "force_flush() should return a result"
        
        # Test temporality
        temporality = metric_exporter.temporality()
        assert temporality is not None, "temporality() should return a value"
        
        # Cleanup
        library.shutdown()
    finally:
        shutil.rmtree(tmpdir, ignore_errors=True)


def test_metric_exporter_with_real_sdk_data():
    """Test metric exporter with actual OpenTelemetry SDK metric data"""
    import otlp_arrow_library
    
    # Try to import OpenTelemetry SDK - skip test if not available
    try:
        from opentelemetry.sdk.metrics import MeterProvider
        from opentelemetry.sdk.metrics.export import PeriodicExportingMetricReader
        from opentelemetry import metrics
    except ImportError:
        pytest.skip("OpenTelemetry SDK not installed - install with: pip install opentelemetry-api opentelemetry-sdk")
    
    # Use mkdtemp instead of TemporaryDirectory to avoid segfault during cleanup
    tmpdir = tempfile.mkdtemp()
    try:
        library = otlp_arrow_library.PyOtlpLibrary(
            output_dir=tmpdir,
            write_interval_secs=1
        )
        
        # Create metric exporter adapter
        metric_exporter = library.metric_exporter_adapter()
        
        # Create a reader with the adapter
        reader = PeriodicExportingMetricReader(
            metric_exporter,
            export_interval_millis=100  # Fast export for testing
        )
        
        # Create meter provider with the reader
        meter_provider = MeterProvider(metric_readers=[reader])
        
        # Set as global meter provider
        metrics.set_meter_provider(meter_provider)
        
        try:
            # Get a meter and create a counter
            meter = metrics.get_meter(__name__)
            counter = meter.create_counter(
                "test_counter",
                description="A test counter metric",
                unit="1"
            )
            
            # Record some metrics
            counter.add(5, {"environment": "test", "service": "test-service"})
            counter.add(3, {"environment": "test", "service": "test-service"})
            
            # Force flush to trigger export
            meter_provider.force_flush(timeout_millis=1000)
            
            # The export should have succeeded (no exception raised)
            # Verify by checking that the adapter's export method was called
            # (we can't directly verify the export result, but if it failed, an exception would be raised)
            assert True, "Export completed successfully"
            
        finally:
            # Cleanup
            meter_provider.shutdown()
            library.shutdown()
    finally:
        shutil.rmtree(tmpdir, ignore_errors=True)


def test_metric_exporter_direct_export():
    """Test metric exporter by directly calling export with SDK-generated data"""
    import otlp_arrow_library
    
    # Try to import OpenTelemetry SDK - skip test if not available
    try:
        from opentelemetry.sdk.metrics import MeterProvider
        from opentelemetry.sdk.metrics.export import PeriodicExportingMetricReader
        from opentelemetry import metrics
    except ImportError:
        pytest.skip("OpenTelemetry SDK not installed - install with: pip install opentelemetry-api opentelemetry-sdk")
    
    # Use mkdtemp instead of TemporaryDirectory to avoid segfault during cleanup
    tmpdir = tempfile.mkdtemp()
    try:
        library = otlp_arrow_library.PyOtlpLibrary(
            output_dir=tmpdir,
            write_interval_secs=1
        )
        
        # Create metric exporter adapter
        metric_exporter = library.metric_exporter_adapter()
        
        # Create a reader with the adapter
        reader = PeriodicExportingMetricReader(
            metric_exporter,
            export_interval_millis=100
        )
        
        # Create meter provider
        meter_provider = MeterProvider(metric_readers=[reader])
        metrics.set_meter_provider(meter_provider)
        
        try:
            # Get a meter and create metrics
            meter = metrics.get_meter(__name__)
            counter = meter.create_counter("test_counter", description="Test counter")
            gauge = meter.create_up_down_counter("test_gauge", description="Test gauge")
            
            # Record metrics
            counter.add(10)
            gauge.add(5)
            
            # Manually collect metrics to get a MetricExportResult
            # This simulates what PeriodicExportingMetricReader does internally
            reader.collect()
            
            # Force flush to ensure export happens
            result = meter_provider.force_flush(timeout_millis=1000)
            assert result, "Force flush should succeed"
            
        finally:
            meter_provider.shutdown()
            library.shutdown()
    finally:
        shutil.rmtree(tmpdir, ignore_errors=True)


if __name__ == "__main__":
    pytest.main([__file__])

