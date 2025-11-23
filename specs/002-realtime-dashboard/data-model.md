# Data Model: Web JS Realtime Dashboard

**Date**: 2024-12-19  
**Feature**: 002-realtime-dashboard

## Entities

### TraceEntry

Represents a single trace span displayed in the dashboard.

**Fields**:
- `traceId: string` - 16-byte trace ID (hex-encoded)
- `spanId: string` - 8-byte span ID (hex-encoded)
- `parentSpanId: string | null` - 8-byte parent span ID (hex-encoded, optional)
- `name: string` - Span name
- `kind: string` - Span kind (internal, server, client, producer, consumer)
- `startTime: number` - Start time in Unix nanoseconds
- `endTime: number` - End time in Unix nanoseconds
- `duration: number` - Duration in milliseconds (calculated)
- `statusCode: string` - Status code (ok, error, unset)
- `statusMessage: string | null` - Status message (optional)
- `attributes: Record<string, any>` - Span attributes (key-value pairs)
- `events: Array<TraceEvent>` - Span events (optional)
- `links: Array<TraceLink>` - Span links (optional)
- `serviceName: string` - Service name (extracted from attributes)
- `error: boolean` - Whether span has error status

**Display Format**:
- Table row with: trace ID (truncated), span name, service, duration, status, timestamp
- Expandable detail view with full attributes, events, links, and timing

**Filtering**:
- By trace ID (exact match or prefix)
- By service name (exact match or contains)
- By span name (exact match or contains)
- By error status (true/false)
- By time range (start/end timestamp)

---

### MetricEntry

Represents a single metric data point displayed in graphs.

**Fields**:
- `metricName: string` - Metric name
- `value: number` - Metric value
- `timestamp: number` - Timestamp in Unix nanoseconds
- `labels: Record<string, string>` - Metric labels (key-value pairs)
- `metricType: string` - Metric type (gauge, counter, histogram, summary)
- `unit: string | null` - Metric unit (optional)

**Display Format**:
- Time-series graph with timestamp on X-axis, value on Y-axis
- Multiple series per graph for different label combinations
- Hover tooltip with full label information

**Aggregation** (for histogram metrics):
- `sum: number` - Sum of values
- `count: number` - Count of values
- `min: number` - Minimum value
- `max: number` - Maximum value
- `avg: number` - Average value
- `p50: number` - 50th percentile
- `p95: number` - 95th percentile
- `p99: number` - 99th percentile

**Filtering**:
- By metric name (exact match or contains)
- By labels (key-value matching)
- By time range (start/end timestamp)

---

### ArrowIpcFile

Represents an Arrow IPC Streaming format file containing trace or metric data.

**Fields**:
- `path: string` - File path
- `name: string` - File name
- `size: number` - File size in bytes
- `lastModified: number` - Last modified timestamp
- `type: 'traces' | 'metrics'` - File type (traces or metrics)
- `schema: ArrowSchema` - Arrow schema (from file header)
- `recordCount: number` - Number of RecordBatches in file
- `loaded: boolean` - Whether file has been loaded into DuckDB
- `lastRead: number | null` - Timestamp of last read (for polling)

**State Transitions**:
- `discovered` → `loading` → `loaded` → `querying` → `queried`
- `discovered` → `error` (if file read fails)

---

### DashboardState

Represents the current state of the dashboard.

**Fields**:
- `selectedDirectory: FileSystemDirectoryHandle | null` - Selected output directory (File System Access API)
- `selectedFiles: File[]` - Selected files (FileReader API fallback)
- `loadedFiles: Map<string, ArrowIpcFile>` - Map of file path to ArrowIpcFile
- `traces: TraceEntry[]` - Current trace entries (filtered)
- `metrics: Map<string, MetricEntry[]>` - Current metric entries by metric name
- `filters: DashboardFilters` - Active filters
- `timeRange: TimeRange` - Selected time range
- `isPaused: boolean` - Whether live stream is paused
- `isLoading: boolean` - Whether data is currently loading
- `error: string | null` - Current error message (if any)
- `lastUpdate: number` - Timestamp of last data update

**State Transitions**:
- `initial` → `selecting` → `loading` → `ready` → `streaming`
- `ready` → `paused` → `resumed` → `streaming`
- Any state → `error` (on error)

---

### DashboardFilters

Represents active filters for traces and metrics.

**Fields**:
- `traceFilters: TraceFilters` - Trace filtering options
- `metricFilters: MetricFilters` - Metric filtering options
- `timeRange: TimeRange` - Time range filter (applies to both)

**TraceFilters**:
- `traceId: string | null` - Trace ID filter (exact or prefix)
- `serviceName: string | null` - Service name filter (contains)
- `spanName: string | null` - Span name filter (contains)
- `errorOnly: boolean` - Show only error traces
- `minDuration: number | null` - Minimum duration in milliseconds
- `maxDuration: number | null` - Maximum duration in milliseconds

**MetricFilters**:
- `metricNames: string[]` - Selected metric names to display
- `labelFilters: Record<string, string>` - Label key-value filters
- `minValue: number | null` - Minimum metric value
- `maxValue: number | null` - Maximum metric value

---

### TimeRange

Represents a time range for filtering data.

**Fields**:
- `start: number` - Start timestamp in Unix nanoseconds
- `end: number` - End timestamp in Unix nanoseconds (null for "now")
- `preset: 'last5m' | 'last15m' | 'last1h' | 'last6h' | 'last24h' | 'custom'` - Preset time range

**Presets**:
- `last5m`: Last 5 minutes
- `last15m`: Last 15 minutes
- `last1h`: Last 1 hour
- `last6h`: Last 6 hours
- `last24h`: Last 24 hours
- `custom`: User-defined time range

---

### DuckDBTable

Represents a table registered in DuckDB-wasm for querying Arrow data.

**Fields**:
- `name: string` - Table name (derived from file name)
- `filePath: string` - Source file path
- `schema: ArrowSchema` - Arrow schema
- `rowCount: number` - Number of rows in table
- `registeredAt: number` - Timestamp when table was registered

**State Transitions**:
- `unregistered` → `registering` → `registered` → `querying` → `queried`
- `registered` → `unregistered` (when file is no longer needed)

---

### PlotlyGraphConfig

Represents configuration for a Plotly.js graph.

**Fields**:
- `metricName: string` - Metric name for this graph
- `traces: PlotlyTrace[]` - Plotly trace configurations (one per label combination)
- `layout: PlotlyLayout` - Plotly layout configuration
- `config: PlotlyConfig` - Plotly display configuration

**PlotlyTrace**:
- `name: string` - Trace name (includes label values)
- `x: number[]` - Timestamp values
- `y: number[]` - Metric values
- `type: 'scatter' | 'bar'` - Trace type (scatter for time-series)
- `mode: 'lines' | 'markers' | 'lines+markers'` - Display mode
- `line: PlotlyLine` - Line styling
- `marker: PlotlyMarker` - Marker styling

**PlotlyLayout**:
- `title: string` - Graph title
- `xaxis: PlotlyAxis` - X-axis configuration (time)
- `yaxis: PlotlyAxis` - Y-axis configuration (value)
- `hovermode: 'closest' | 'x' | 'y'` - Hover mode
- `showlegend: boolean` - Whether to show legend

---

## Data Flow

### Trace Data Flow

```
Arrow IPC File → FileReader → Arrow Parser → RecordBatch → DuckDB Registration
  → SQL Query (with filters) → Query Results → TraceEntry[] → UI Update
```

### Metric Data Flow

```
Arrow IPC File → FileReader → Arrow Parser → RecordBatch → DuckDB Registration
  → SQL Query (with filters/aggregation) → Query Results → MetricEntry[]
  → PlotlyGraphConfig → Plotly.js Render → UI Update
```

### File Discovery Flow

```
Poll Directory (every 1s) → Discover New Files → Check if Loaded
  → If New: Read File → Parse Arrow → Register in DuckDB → Query → Update UI
  → If Changed: Re-read File → Update DuckDB → Re-query → Update UI
```

---

## Query Patterns

### Trace Queries

```sql
-- Get all traces with filters
SELECT * FROM traces_table
WHERE trace_id LIKE 'abc123%'
  AND service_name LIKE '%api%'
  AND status_code = 'error'
  AND start_time >= 1234567890000000000
  AND start_time <= 1234567895000000000
ORDER BY start_time DESC
LIMIT 10000;
```

### Metric Queries

```sql
-- Get metric values with aggregation
SELECT 
  metric_name,
  labels,
  timestamp,
  value,
  AVG(value) OVER (PARTITION BY metric_name, labels ORDER BY timestamp ROWS BETWEEN 9 PRECEDING AND CURRENT ROW) as moving_avg
FROM metrics_table
WHERE metric_name IN ('cpu_usage', 'memory_usage')
  AND timestamp >= 1234567890000000000
  AND timestamp <= 1234567895000000000
ORDER BY timestamp ASC;
```

### Histogram Aggregation

```sql
-- Aggregate histogram metrics
SELECT 
  metric_name,
  labels,
  timestamp,
  COUNT(*) as count,
  SUM(value) as sum,
  AVG(value) as avg,
  MIN(value) as min,
  MAX(value) as max,
  PERCENTILE_CONT(0.5) WITHIN GROUP (ORDER BY value) as p50,
  PERCENTILE_CONT(0.95) WITHIN GROUP (ORDER BY value) as p95,
  PERCENTILE_CONT(0.99) WITHIN GROUP (ORDER BY value) as p99
FROM metrics_table
WHERE metric_type = 'histogram'
GROUP BY metric_name, labels, timestamp;
```

---

## Memory Management

### Trace Data Limits

- Maximum traces in memory: 10,000 (configurable)
- When limit reached: Remove oldest traces (FIFO)
- Virtual scrolling: Only render visible traces (e.g., 50 at a time)

### Metric Data Limits

- Maximum data points per graph: 10,000 (configurable)
- When limit reached: Use sliding window (remove oldest points)
- Graph downsampling: Aggregate points if too many (e.g., 1 point per second)

### File Management

- Maximum loaded files: 100 (configurable)
- When limit reached: Unregister oldest files from DuckDB
- File cleanup: Remove files older than 24 hours from loaded set

---

## Error Handling

### File Read Errors

- **Error**: File not found or unreadable
- **Handling**: Log error, skip file, continue with other files
- **User Feedback**: Show error message in UI, allow retry

### Arrow Parsing Errors

- **Error**: Invalid Arrow IPC format
- **Handling**: Log error, skip file, continue with other files
- **User Feedback**: Show error message in UI, mark file as error

### DuckDB Query Errors

- **Error**: Query timeout or syntax error
- **Handling**: Log error, retry with simpler query, fallback to manual filtering
- **User Feedback**: Show error message in UI, allow query edit

### Memory Errors

- **Error**: Browser memory limit exceeded
- **Handling**: Reduce data limits, unregister old files, clear caches
- **User Feedback**: Show warning message, suggest reducing data limits

