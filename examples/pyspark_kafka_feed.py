#!/usr/bin/env python3
"""
PySpark Kafka Feed Example with OTLP Metrics

This example demonstrates how to:
- Read data from Kafka using PySpark Structured Streaming
- Monitor memory usage, throughput, and back pressure
- Export metrics and logs to OTLP Arrow Library
- Handle back pressure gracefully

## Prerequisites

Install required packages:
```bash
# Install Python bindings
maturin develop --features python-extension

# Install dependencies
pip install pyspark opentelemetry-api opentelemetry-sdk psutil

# Or use requirements file
pip install -r examples/requirements-pyspark.txt
```

## Kafka Setup

You'll need a running Kafka cluster. For local testing:
```bash
# Using Docker
docker run -d --name kafka -p 9092:9092 apache/kafka:latest

# Or using Kafka installed locally
# Start Zookeeper and Kafka servers
```

## Usage

```bash
python examples/pyspark_kafka_feed.py \
    --kafka-bootstrap-servers localhost:9092 \
    --kafka-topic my-topic \
    --output-dir ./output_dir \
    --checkpoint-location ./checkpoint
```

## Metrics Exported

- **spark.kafka.memory.used**: Memory used by Spark process (MB)
- **spark.kafka.memory.available**: Available memory (MB)
- **spark.kafka.throughput**: Messages processed per second
- **spark.kafka.backpressure.lag**: Kafka consumer lag (messages)
- **spark.kafka.batch.duration**: Batch processing duration (ms)
- **spark.kafka.records.processed**: Total records processed
- **spark.kafka.errors**: Number of processing errors
"""

import argparse
import os
import sys
import time
import threading
import signal
import logging
from typing import Optional
from datetime import datetime

try:
    from pyspark.sql import SparkSession
    from pyspark.sql.functions import col, from_json, window, count, current_timestamp
    from pyspark.sql.types import StructType, StructField, StringType, IntegerType, TimestampType
except ImportError:
    print("❌ Error: PySpark not found. Please install it:")
    print("   pip install pyspark")
    sys.exit(1)

try:
    import otlp_arrow_library
except ImportError:
    print("❌ Error: otlp_arrow_library not found. Please install it first:")
    print("   maturin develop --features python-extension")
    sys.exit(1)

try:
    from opentelemetry.sdk.metrics import MeterProvider
    from opentelemetry.sdk.metrics.export import PeriodicExportingMetricReader
    from opentelemetry import metrics
    from opentelemetry.sdk.trace.export import BatchSpanProcessor
    from opentelemetry.sdk.trace import TracerProvider
    from opentelemetry import trace
except ImportError:
    print("❌ Error: OpenTelemetry SDK not found. Please install it:")
    print("   pip install opentelemetry-api opentelemetry-sdk")
    sys.exit(1)

# Configure logging
logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s - %(name)s - %(levelname)s - %(message)s',
    handlers=[
        logging.StreamHandler(sys.stdout),
        logging.FileHandler('pyspark_kafka_feed.log')
    ]
)
logger = logging.getLogger(__name__)

# Global flag for graceful shutdown
shutdown_flag = threading.Event()


def signal_handler(sig, frame):
    """Handle Ctrl+C gracefully"""
    logger.info("Shutdown signal received, shutting down gracefully...")
    shutdown_flag.set()


class SparkKafkaMetrics:
    """Monitor and export Spark Kafka processing metrics"""
    
    def __init__(self, library: otlp_arrow_library.PyOtlpLibrary, spark: SparkSession):
        self.library = library
        self.spark = spark
        self.meter = None
        self.tracer = None
        self._setup_telemetry()
        
        # Metrics
        self.memory_used_gauge = None
        self.memory_available_gauge = None
        self.throughput_counter = None
        self.backpressure_lag_gauge = None
        self.batch_duration_histogram = None
        self.records_processed_counter = None
        self.errors_counter = None
        
        # Statistics
        self.last_record_count = 0
        self.last_check_time = time.time()
        self.total_records = 0
        self.total_errors = 0
        self._estimated_lag = 0  # Estimated Kafka consumer lag
        
    def _setup_telemetry(self):
        """Setup OpenTelemetry metrics and traces"""
        # Create metric exporter adapter
        metric_exporter = self.library.metric_exporter_adapter()
        reader = PeriodicExportingMetricReader(
            metric_exporter,
            export_interval_millis=1000  # Export every second for real-time monitoring
        )
        meter_provider = MeterProvider(metric_readers=[reader])
        metrics.set_meter_provider(meter_provider)
        self.meter = metrics.get_meter(__name__)
        
        # Create span exporter adapter
        span_exporter = self.library.span_exporter_adapter()
        span_processor = BatchSpanProcessor(span_exporter)
        tracer_provider = TracerProvider()
        tracer_provider.add_span_processor(span_processor)
        trace.set_tracer_provider(tracer_provider)
        self.tracer = trace.get_tracer(__name__)
        
        # Create metrics
        self.memory_used_gauge = self.meter.create_observable_gauge(
            "spark.kafka.memory.used",
            callbacks=[self._memory_used_callback],
            description="Memory used by Spark process in MB",
            unit="MB"
        )
        
        self.memory_available_gauge = self.meter.create_observable_gauge(
            "spark.kafka.memory.available",
            callbacks=[self._memory_available_callback],
            description="Available memory in MB",
            unit="MB"
        )
        
        self.throughput_counter = self.meter.create_counter(
            "spark.kafka.throughput",
            description="Messages processed per second",
            unit="1/s"
        )
        
        self.backpressure_lag_gauge = self.meter.create_observable_gauge(
            "spark.kafka.backpressure.lag",
            callbacks=[self._backpressure_lag_callback],
            description="Kafka consumer lag (messages)",
            unit="1"
        )
        
        self.batch_duration_histogram = self.meter.create_histogram(
            "spark.kafka.batch.duration",
            description="Batch processing duration in milliseconds",
            unit="ms"
        )
        
        self.records_processed_counter = self.meter.create_counter(
            "spark.kafka.records.processed",
            description="Total records processed",
            unit="1"
        )
        
        self.errors_counter = self.meter.create_counter(
            "spark.kafka.errors",
            description="Number of processing errors",
            unit="1"
        )
        
        logger.info("Telemetry setup complete")
    
    def _memory_used_callback(self, callback_options):
        """Callback to get memory used by Spark process"""
        try:
            import psutil
            import os
            process = psutil.Process(os.getpid())
            memory_mb = process.memory_info().rss / (1024 * 1024)
            return [metrics.Observation(memory_mb, {"process": "spark"})]
        except Exception as e:
            logger.warning(f"Failed to get memory usage: {e}")
            return [metrics.Observation(0.0, {"process": "spark"})]
    
    def _memory_available_callback(self, callback_options):
        """Callback to get available system memory"""
        try:
            import psutil
            memory = psutil.virtual_memory()
            available_mb = memory.available / (1024 * 1024)
            return [metrics.Observation(available_mb, {"system": "host"})]
        except Exception as e:
            logger.warning(f"Failed to get available memory: {e}")
            return [metrics.Observation(0.0, {"system": "host"})]
    
    def _backpressure_lag_callback(self, callback_options):
        """Callback to get Kafka consumer lag"""
        try:
            # Get lag from Spark streaming query progress
            # Spark tracks offset ranges which can be used to estimate lag
            lag = getattr(self, '_estimated_lag', 0)
            
            # In production, you could also query Kafka Admin API for actual consumer group lag:
            # from kafka import KafkaAdminClient
            # admin = KafkaAdminClient(bootstrap_servers='localhost:9092')
            # # Query consumer group lag metrics
            # # This requires Kafka 0.11.0+ and proper consumer group configuration
            
            return [metrics.Observation(float(lag), {"topic": "kafka"})]
        except Exception as e:
            logger.warning(f"Failed to get backpressure lag: {e}")
            return [metrics.Observation(0.0, {"topic": "kafka"})]
    
    def update_lag_estimate(self, query_progress):
        """Update lag estimate from Spark streaming query progress"""
        try:
            # Extract offset information from query progress
            # Spark provides offset ranges in the progress object
            if hasattr(query_progress, 'sources') and query_progress.sources:
                for source in query_progress.sources:
                    if hasattr(source, 'startOffset') and hasattr(source, 'endOffset'):
                        # Calculate lag as difference between latest offset and processed offset
                        # This is a simplified calculation - actual lag requires Kafka metadata
                        start = source.startOffset
                        end = source.endOffset
                        if isinstance(start, dict) and isinstance(end, dict):
                            # Estimate lag based on offset ranges
                            # In a real scenario, you'd compare with Kafka's latest offset
                            self._estimated_lag = max(0, end.get('latest', {}).get('offset', 0) - 
                                                         start.get('earliest', {}).get('offset', 0))
        except Exception as e:
            logger.debug(f"Could not update lag estimate: {e}")
            self._estimated_lag = 0
    
    def record_throughput(self, records_per_second: float):
        """Record throughput metric"""
        self.throughput_counter.add(
            int(records_per_second),
            {"source": "kafka"}
        )
    
    def record_batch_duration(self, duration_ms: float):
        """Record batch processing duration"""
        self.batch_duration_histogram.record(
            duration_ms,
            {"operation": "kafka_batch"}
        )
    
    def record_records_processed(self, count: int):
        """Record number of records processed"""
        self.records_processed_counter.add(
            count,
            {"source": "kafka"}
        )
        self.total_records += count
    
    def record_error(self, error_type: str = "processing"):
        """Record an error"""
        self.errors_counter.add(
            1,
            {"error_type": error_type}
        )
        self.total_errors += 1
    
    def update_statistics(self, current_count: int):
        """Update statistics and calculate throughput"""
        current_time = time.time()
        time_delta = current_time - self.last_check_time
        
        if time_delta >= 1.0:  # Update every second
            records_delta = current_count - self.last_record_count
            throughput = records_delta / time_delta if time_delta > 0 else 0
            
            self.record_throughput(throughput)
            self.last_record_count = current_count
            self.last_check_time = current_time
            
            logger.info(
                f"Stats: throughput={throughput:.2f} msg/s, "
                f"total={self.total_records}, errors={self.total_errors}"
            )


def create_spark_session(app_name: str, checkpoint_location: str) -> SparkSession:
    """Create and configure Spark session for Kafka streaming"""
    spark = SparkSession.builder \
        .appName(app_name) \
        .config("spark.sql.streaming.checkpointLocation", checkpoint_location) \
        .config("spark.sql.streaming.schemaInference", "true") \
        .config("spark.sql.adaptive.enabled", "true") \
        .config("spark.sql.adaptive.coalescePartitions.enabled", "true") \
        .getOrCreate()
    
    spark.sparkContext.setLogLevel("WARN")  # Reduce Spark logging noise
    logger.info(f"Spark session created: {spark.sparkContext.applicationId}")
    return spark


def read_from_kafka(
    spark: SparkSession,
    bootstrap_servers: str,
    topic: str,
    starting_offsets: str = "latest"
) -> 'DataFrame':
    """Read stream from Kafka"""
    logger.info(f"Reading from Kafka: {bootstrap_servers}, topic: {topic}")
    
    df = spark \
        .readStream \
        .format("kafka") \
        .option("kafka.bootstrap.servers", bootstrap_servers) \
        .option("subscribe", topic) \
        .option("startingOffsets", starting_offsets) \
        .option("failOnDataLoss", "false") \
        .option("maxOffsetsPerTrigger", 10000) \
        .load()
    
    return df


def process_kafka_stream(
    df,
    metrics_monitor: SparkKafkaMetrics,
    output_mode: str = "update"
):
    """Process Kafka stream with metrics monitoring"""
    from pyspark.sql.functions import col, from_json, current_timestamp, lit
    
    # Define schema for JSON messages (adjust based on your Kafka message format)
    schema = StructType([
        StructField("id", StringType(), True),
        StructField("value", StringType(), True),
        StructField("timestamp", TimestampType(), True),
    ])
    
    # Parse JSON from Kafka value
    parsed_df = df.select(
        col("key").cast("string").alias("kafka_key"),
        from_json(col("value").cast("string"), schema).alias("data"),
        col("timestamp").alias("kafka_timestamp"),
        col("partition"),
        col("offset")
    ).select(
        col("kafka_key"),
        col("data.*"),
        col("kafka_timestamp"),
        col("partition"),
        col("offset")
    )
    
    # Add processing timestamp
    processed_df = parsed_df.withColumn(
        "processing_timestamp",
        current_timestamp()
    )
    
    # Query with metrics tracking
    query = processed_df.writeStream \
        .outputMode(output_mode) \
        .foreachBatch(lambda batch_df, batch_id: process_batch(
            batch_df, batch_id, metrics_monitor
        )) \
        .trigger(processingTime='2 seconds') \
        .start()
    
    return query


def process_batch(batch_df, batch_id, metrics_monitor: SparkKafkaMetrics):
    """Process a batch of records with metrics"""
    start_time = time.time()
    record_count = 0
    
    try:
        # Count records in batch
        record_count = batch_df.count()
        
        # Estimate backpressure: if batch size is consistently large, there may be lag
        # This is a heuristic - actual lag requires querying Kafka
        if record_count > 0:
            # Simple heuristic: if we're processing many records per batch,
            # there might be backpressure (more data waiting)
            # In production, query Kafka Admin API for actual lag
            estimated_lag = max(0, record_count * 2)  # Placeholder heuristic
            metrics_monitor._estimated_lag = estimated_lag
        
        # Process records (example: just log them)
        # In production, you'd do actual processing here
        batch_df.show(truncate=False)
        
        # Record metrics
        metrics_monitor.record_records_processed(record_count)
        metrics_monitor.update_statistics(metrics_monitor.total_records)
        
        # Calculate batch duration
        duration_ms = (time.time() - start_time) * 1000
        metrics_monitor.record_batch_duration(duration_ms)
        
        logger.info(
            f"Batch {batch_id}: processed {record_count} records in {duration_ms:.2f}ms "
            f"(estimated lag: {metrics_monitor._estimated_lag})"
        )
        
    except Exception as e:
        logger.error(f"Error processing batch {batch_id}: {e}", exc_info=True)
        metrics_monitor.record_error("batch_processing")
        raise


def monitor_spark_metrics(
    spark: SparkSession,
    metrics_monitor: SparkKafkaMetrics,
    query,  # Add query parameter to monitor progress
    interval_seconds: float = 5.0
):
    """Monitor Spark metrics in a separate thread"""
    while not shutdown_flag.is_set():
        try:
            # Get Spark status
            status = spark.streams.active
            active_streams = len(status)
            
            if active_streams > 0:
                for stream in status:
                    # Get query progress for backpressure monitoring
                    if hasattr(stream, 'lastProgress') and stream.lastProgress:
                        progress = stream.lastProgress
                        metrics_monitor.update_lag_estimate(progress)
                        
                        # Log progress information
                        logger.info(
                            f"Stream: {stream.name}, "
                            f"id={stream.id}, "
                            f"isActive={stream.isActive}, "
                            f"estimated_lag={metrics_monitor._estimated_lag}"
                        )
                    else:
                        logger.info(
                            f"Stream: {stream.name}, "
                            f"id={stream.id}, "
                            f"isActive={stream.isActive}"
                        )
            
            # Log memory usage
            try:
                import psutil
                import os
                process = psutil.Process(os.getpid())
                memory_mb = process.memory_info().rss / (1024 * 1024)
                logger.info(f"Memory usage: {memory_mb:.2f} MB")
            except Exception as e:
                logger.warning(f"Failed to get memory: {e}")
            
        except Exception as e:
            logger.error(f"Error monitoring metrics: {e}", exc_info=True)
        
        shutdown_flag.wait(timeout=interval_seconds)


def main():
    """Main entry point"""
    parser = argparse.ArgumentParser(
        description="PySpark Kafka Feed with OTLP Metrics"
    )
    parser.add_argument(
        "--kafka-bootstrap-servers",
        default="localhost:9092",
        help="Kafka bootstrap servers (default: localhost:9092)"
    )
    parser.add_argument(
        "--kafka-topic",
        default="test-topic",
        help="Kafka topic to read from (default: test-topic)"
    )
    parser.add_argument(
        "--output-dir",
        default="./output_dir",
        help="Output directory for OTLP files (default: ./output_dir)"
    )
    parser.add_argument(
        "--checkpoint-location",
        default="./checkpoint",
        help="Spark checkpoint location (default: ./checkpoint)"
    )
    parser.add_argument(
        "--starting-offsets",
        default="latest",
        choices=["earliest", "latest"],
        help="Kafka starting offsets (default: latest)"
    )
    parser.add_argument(
        "--app-name",
        default="KafkaFeed",
        help="Spark application name (default: KafkaFeed)"
    )
    
    args = parser.parse_args()
    
    # Setup signal handlers
    signal.signal(signal.SIGINT, signal_handler)
    signal.signal(signal.SIGTERM, signal_handler)
    
    logger.info("=" * 60)
    logger.info("PySpark Kafka Feed with OTLP Metrics")
    logger.info("=" * 60)
    logger.info(f"Kafka servers: {args.kafka_bootstrap_servers}")
    logger.info(f"Kafka topic: {args.kafka_topic}")
    logger.info(f"Output directory: {args.output_dir}")
    logger.info(f"Checkpoint location: {args.checkpoint_location}")
    logger.info("=" * 60)
    
    # Create OTLP library
    try:
        library = otlp_arrow_library.PyOtlpLibrary(
            output_dir=args.output_dir,
            write_interval_secs=5,
            dashboard_enabled=True,
            dashboard_port=8080,
            dashboard_static_dir="./dashboard/dist",
            dashboard_bind_address="127.0.0.1"
        )
        logger.info("OTLP library initialized")
    except Exception as e:
        logger.error(f"Failed to initialize OTLP library: {e}", exc_info=True)
        sys.exit(1)
    
    # Create Spark session
    try:
        spark = create_spark_session(args.app_name, args.checkpoint_location)
    except Exception as e:
        logger.error(f"Failed to create Spark session: {e}", exc_info=True)
        sys.exit(1)
    
    # Setup metrics monitoring
    try:
        metrics_monitor = SparkKafkaMetrics(library, spark)
    except Exception as e:
        logger.error(f"Failed to setup metrics: {e}", exc_info=True)
        spark.stop()
        sys.exit(1)
    
    # Read from Kafka
    try:
        df = read_from_kafka(
            spark,
            args.kafka_bootstrap_servers,
            args.kafka_topic,
            args.starting_offsets
        )
        
        # Process stream
        query = process_kafka_stream(df, metrics_monitor)
        
        # Start metrics monitoring thread (after query is created)
        monitor_thread = threading.Thread(
            target=monitor_spark_metrics,
            args=(spark, metrics_monitor, query),
            daemon=True
        )
        monitor_thread.start()
        
        logger.info("Stream processing started. Press Ctrl+C to stop.")
        logger.info(f"Dashboard available at http://127.0.0.1:8080")
        
        # Wait for query to complete or shutdown signal
        query.awaitTermination()
        
    except KeyboardInterrupt:
        logger.info("Interrupted by user")
    except Exception as e:
        logger.error(f"Stream processing error: {e}", exc_info=True)
        metrics_monitor.record_error("stream_processing")
    finally:
        # Shutdown
        logger.info("Shutting down...")
        
        try:
            # Stop Spark streams
            for stream in spark.streams.active:
                stream.stop()
            
            # Flush metrics
            from opentelemetry.sdk.metrics import MeterProvider
            meter_provider = metrics.get_meter_provider()
            if isinstance(meter_provider, MeterProvider):
                meter_provider.force_flush(timeout_millis=2000)
                meter_provider.shutdown()
            
            # Flush library
            library.flush()
            library.shutdown()
            
            # Stop Spark
            spark.stop()
            
            logger.info("Shutdown complete")
            logger.info(f"Total records processed: {metrics_monitor.total_records}")
            logger.info(f"Total errors: {metrics_monitor.total_errors}")
            logger.info(f"Check output directory: {args.output_dir}/otlp/")
            
        except Exception as e:
            logger.error(f"Error during shutdown: {e}", exc_info=True)


if __name__ == "__main__":
    main()

