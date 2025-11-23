"""Python example usage of OTLP Arrow Library"""

import otlp_arrow_library
import tempfile
import os


def main():
    """Example demonstrating library usage"""
    
    # Create a temporary directory for output
    with tempfile.TemporaryDirectory() as tmpdir:
        print(f"Using output directory: {tmpdir}")
        
        # Initialize the library with custom configuration
        library = otlp_arrow_library.PyOtlpLibrary(
            output_dir=tmpdir,
            write_interval_secs=2,
            trace_cleanup_interval_secs=600,
            metric_cleanup_interval_secs=3600,
            protobuf_enabled=True,
            arrow_flight_enabled=True
        )
        
        print("Library initialized successfully")
        
        # Export a single trace
        trace_id = bytes([1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16])
        span_id = bytes([1, 2, 3, 4, 5, 6, 7, 8])
        
        span = {
            "trace_id": trace_id,
            "span_id": span_id,
            "name": "example-span",
            "kind": "server",
            "attributes": {
                "service.name": "example-service",
                "http.method": "GET",
                "http.status_code": 200
            }
        }
        
        print("Exporting trace...")
        library.export_trace(span)
        
        # Export multiple traces
        print("Exporting multiple traces...")
        spans = []
        for i in range(3):
            trace_id = bytes([i] * 16)
            span_id = bytes([i] * 8)
            span_dict = {
                "trace_id": trace_id,
                "span_id": span_id,
                "name": f"batch-span-{i}",
                "kind": "internal",
                "attributes": {
                    "batch.id": str(i)
                }
            }
            spans.append(span_dict)
        
        library.export_traces(spans)
        
        # Export metrics
        print("Exporting metrics...")
        metrics_dict = {
            "resource": {},
            "scope_metrics": []
        }
        library.export_metrics(metrics_dict)
        
        # Flush to ensure all data is written
        print("Flushing buffers...")
        library.flush()
        
        # Verify files were created
        traces_dir = os.path.join(tmpdir, "otlp", "traces")
        if os.path.exists(traces_dir):
            files = os.listdir(traces_dir)
            print(f"Created {len(files)} trace file(s)")
        
        metrics_dir = os.path.join(tmpdir, "otlp", "metrics")
        if os.path.exists(metrics_dir):
            print(f"Metrics directory exists: {metrics_dir}")
        
        # Shutdown gracefully
        print("Shutting down library...")
        library.shutdown()
        print("Example completed successfully!")


if __name__ == "__main__":
    main()

