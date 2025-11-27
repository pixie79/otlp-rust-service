# Data Model: Demo Rust Application

**Feature**: 005-demo-app  
**Date**: 2025-11-26

## Overview

The demo application is a standalone Rust executable that demonstrates OTLP SDK usage. It does not define new data structures but uses existing OTLP SDK types. This document describes the data entities used in the demo and their relationships.

## Entities

### Demo Application State

The demo application maintains minimal state:

- **Library Instance** (`OtlpLibrary`): Main library instance that handles telemetry export
- **Configuration** (`Config`): Configuration object specifying dashboard settings, output directory, write intervals
- **Generation Counter**: Internal counter for generating unique trace/span IDs and metric values

### Mock Metrics

**Type**: `opentelemetry_sdk::metrics::data::ResourceMetrics`

**Purpose**: Synthetic metric data for demonstration

**Characteristics**:
- Created using `ResourceMetrics::default()` (due to private fields in SDK)
- Exported via `library.export_metrics()` or `library.export_metrics_ref()`
- Written to Arrow IPC files in `{output_dir}/otlp/metrics/`
- Displayed in dashboard for visualization

**Note**: Full metric construction is limited by SDK's private fields. Demo focuses on demonstrating export API rather than complex metric construction.

### Mock Spans

**Type**: `opentelemetry_sdk::trace::SpanData`

**Purpose**: Synthetic trace span data for demonstration

**Key Fields**:
- `span_context`: Contains `trace_id` and `span_id`
- `parent_span_id`: Links to parent span (or `SpanId::INVALID` for root)
- `span_kind`: Type of span (Server, Client, Internal, Producer, Consumer)
- `name`: Operation name (e.g., "http-request", "database-query")
- `attributes`: Key-value pairs (service.name, http.method, http.status_code, etc.)
- `start_time` / `end_time`: Timestamp range
- `status`: Span status (Ok, Error)

**Relationships**:
- Multiple spans can share the same `trace_id` (belong to same trace)
- Child spans reference parent via `parent_span_id`
- Root spans have `parent_span_id = SpanId::INVALID`

**Example Structure**:
```
Trace (trace_id: [1,2,3,...])
├── Root Span (span_id: [1,1,1,...], parent: INVALID, kind: Server)
│   ├── Child Span (span_id: [2,2,2,...], parent: [1,1,1,...], kind: Internal)
│   └── Child Span (span_id: [3,3,3,...], parent: [1,1,1,...], kind: Client)
```

### Dashboard Configuration

**Type**: `otlp_arrow_library::config::DashboardConfig`

**Purpose**: Configuration for dashboard HTTP server

**Fields**:
- `enabled: bool` - Whether dashboard is enabled (default: false)
- `port: u16` - HTTP server port (default: 8080)
- `static_dir: PathBuf` - Directory containing dashboard static files (default: `./dashboard/dist`)
- `bind_address: String` - Bind address (default: "127.0.0.1")

**Validation Rules**:
- Port must be 1-65535
- Port cannot conflict with gRPC ports (4317, 4318)
- Static directory must exist when enabled
- Bind address must be valid IP address

## Data Flow

1. **Initialization**: Demo creates `Config` with dashboard enabled, then creates `OtlpLibrary` instance
2. **Generation**: Demo generates mock `SpanData` and `ResourceMetrics` objects
3. **Export**: Demo calls `library.export_trace()` / `library.export_traces()` / `library.export_metrics()`
4. **Buffering**: Library buffers exported data internally
5. **Batch Write**: Library writes batches to Arrow IPC files at configured intervals
6. **Dashboard**: Dashboard reads Arrow IPC files and displays data in web UI

## Validation Rules

- Trace IDs must be 16 bytes
- Span IDs must be 8 bytes
- Parent span ID must be valid (8 bytes) or `SpanId::INVALID`
- Span start_time must be <= end_time
- All spans in a trace must share the same trace_id
- Dashboard static directory must exist when dashboard is enabled

## State Transitions

**Application Lifecycle**:
1. **Initialized**: Library created, dashboard started (if enabled)
2. **Generating**: Demo generating and exporting data
3. **Flushing**: Demo calls `flush()` to write pending data
4. **Shutdown**: Demo calls `shutdown()`, library cleans up resources

**Data States**:
- **Pending**: Data exported but not yet written to disk (buffered)
- **Written**: Data written to Arrow IPC files
- **Visible**: Data visible in dashboard (after file read)

