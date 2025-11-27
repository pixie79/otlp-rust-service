# Quickstart Guide: Demo Rust Application

**Feature**: 005-demo-app  
**Date**: 2025-11-26

## Overview

This quickstart guide helps you run the demo Rust application to verify the OTLP service is working and see a reference implementation of SDK usage.

## Prerequisites

- Rust 1.75+ (stable channel) installed
- OTLP Rust Service repository cloned
- Dashboard static files built (in `./dashboard/dist`)

## Quick Start

### Step 1: Build Dashboard (if needed)

If the dashboard static files don't exist, build them:

```bash
cd dashboard
npm install
npm run build
cd ..
```

### Step 2: Run the Demo

Run the demo application:

```bash
cargo run --example demo-app
```

### Step 3: View Dashboard

Open your browser and navigate to:

```
http://127.0.0.1:8080
```

You should see:
- Metrics appearing in the metrics view
- Spans appearing in the traces view
- Data updating as the demo generates new telemetry

### Step 4: Stop the Demo

Press `Ctrl+C` to stop the demo. The application will:
- Flush any pending data
- Shutdown gracefully
- Exit cleanly

## What the Demo Does

The demo application:

1. **Enables Dashboard**: Starts the dashboard HTTP server on port 8080
2. **Generates Metrics**: Creates and exports mock metrics using the OTLP SDK
3. **Generates Spans**: Creates and exports mock spans with different kinds and relationships
4. **Writes Data**: Exports data to Arrow IPC files in `./output_dir/otlp/`
5. **Displays Data**: Dashboard reads and displays the data in real-time

## Understanding the Code

The demo application (`examples/demo-app.rs`) demonstrates:

### Dashboard Configuration

```rust
let config = ConfigBuilder::new()
    .dashboard_enabled(true)  // Enable dashboard
    .output_dir("./output_dir")
    .build()?;
```

### Library Initialization

```rust
let library = OtlpLibrary::new(config).await?;
```

### Creating and Exporting Spans

```rust
// Create a span
let span = SpanData {
    span_context: /* ... */,
    name: std::borrow::Cow::Borrowed("example-operation"),
    // ... other fields
};

// Export the span
library.export_trace(span).await?;
```

### Creating and Exporting Metrics

```rust
// Create metrics (using default due to SDK limitations)
let metrics = ResourceMetrics::default();

// Export metrics
library.export_metrics(metrics).await?;
```

### Graceful Shutdown

```rust
// Flush pending data
library.flush().await?;

// Shutdown gracefully
library.shutdown().await?;
```

## Using as a Reference

The demo application serves as a reference implementation. To use it in your own project:

1. **Copy Patterns**: Use the code patterns shown in the demo
2. **Adapt Configuration**: Modify configuration for your needs
3. **Customize Data**: Replace mock data with your application's telemetry
4. **Follow Structure**: Use the same initialization → generation → export → shutdown pattern

## Troubleshooting

### Dashboard Not Accessible

**Problem**: Cannot access dashboard at http://127.0.0.1:8080

**Solutions**:
- Check that dashboard static files exist in `./dashboard/dist`
- Verify port 8080 is not in use by another application
- Check console output for dashboard startup errors

### No Data in Dashboard

**Problem**: Dashboard loads but shows no metrics or spans

**Solutions**:
- Wait 10 seconds for data to appear (write interval)
- Check that `./output_dir/otlp/` contains Arrow IPC files
- Verify demo is generating data (check console output)
- Check browser console for errors

### Demo Fails to Start

**Problem**: Demo exits immediately with error

**Solutions**:
- Verify Rust version: `rustc --version` (should be 1.75+)
- Run `cargo build --example demo-app` to see compilation errors
- Check that all dependencies are installed
- Verify dashboard static directory exists if dashboard is enabled

### Port Already in Use

**Problem**: Error about port 8080 already in use

**Solutions**:
- Stop other applications using port 8080
- Modify demo to use different port: `.dashboard_port(8081)`
- Use environment variable: `OTLP_DASHBOARD_PORT=8081`

## Next Steps

After running the demo:

1. **Read the Code**: Examine `examples/demo-app.rs` to understand SDK usage
2. **Modify the Demo**: Try changing configuration or data generation
3. **Integrate SDK**: Use the patterns in your own application
4. **Explore Dashboard**: Use dashboard features to explore the generated data

## Additional Resources

- [Main README](../../README.md): General library documentation
- [Embedded Example](../../examples/embedded.rs): Simpler embedded usage example
- [Standalone Example](../../examples/standalone.rs): Standalone service example
- [Specification](./spec.md): Full feature specification

