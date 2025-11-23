# UI Component Contracts: Web JS Realtime Dashboard

**Date**: 2024-12-19  
**Feature**: 002-realtime-dashboard

## Overview

This document defines the contracts for UI components in the dashboard. Since the dashboard is entirely client-side with no server API, these contracts define component interfaces, data flow, and interaction patterns.

## Component Interfaces

### FileReader Component

**Purpose**: Handles file system access and reading Arrow IPC files.

**Interface**:
```javascript
class FileReader {
  // Select directory using File System Access API or FileReader API
  async selectDirectory(): Promise<FileSystemDirectoryHandle | File[]>
  
  // Read Arrow IPC file as ArrayBuffer
  async readFile(file: File | FileSystemFileHandle): Promise<ArrayBuffer>
  
  // List files in directory
  async listFiles(directory: FileSystemDirectoryHandle | File[]): Promise<File[]>
  
  // Check if file has changed (for polling)
  async getFileMetadata(file: File | FileSystemFileHandle): Promise<{ size: number, lastModified: number }>
}
```

**Error Handling**: Throws `FileReadError` on file access failures.

---

### DuckDBClient Component

**Purpose**: Manages DuckDB-wasm instance and table registration.

**Interface**:
```javascript
class DuckDBClient {
  // Initialize DuckDB-wasm instance
  async initialize(): Promise<void>
  
  // Register Arrow IPC file as table
  async registerArrowFile(fileName: string, arrowData: ArrayBuffer): Promise<string>
  
  // Execute SQL query
  async query(sql: string, params?: any[]): Promise<any[]>
  
  // Unregister table
  async unregisterTable(tableName: string): Promise<void>
  
  // Close DuckDB connection
  async close(): Promise<void>
}
```

**Error Handling**: Throws `DuckDBError` on query failures or initialization errors.

---

### TraceList Component

**Purpose**: Displays trace entries in a scrollable list with filtering.

**Interface**:
```javascript
class TraceList {
  // Set trace data
  setTraces(traces: TraceEntry[]): void
  
  // Apply filters
  applyFilters(filters: TraceFilters): void
  
  // Get selected trace
  getSelectedTrace(): TraceEntry | null
  
  // Event: trace selected
  onTraceSelected: (trace: TraceEntry) => void
  
  // Event: filter changed
  onFilterChanged: (filters: TraceFilters) => void
}
```

**Data Contract**: Accepts `TraceEntry[]` as defined in data-model.md.

---

### TraceDetail Component

**Purpose**: Displays detailed information for a selected trace.

**Interface**:
```javascript
class TraceDetail {
  // Display trace details
  showTrace(trace: TraceEntry): void
  
  // Clear display
  clear(): void
}
```

**Data Contract**: Accepts `TraceEntry` as defined in data-model.md.

---

### MetricGraph Component

**Purpose**: Displays metric data as time-series graphs using Plotly.js.

**Interface**:
```javascript
class MetricGraph {
  // Add or update metric data
  updateMetric(metricName: string, data: MetricEntry[]): void
  
  // Remove metric from display
  removeMetric(metricName: string): void
  
  // Set time range
  setTimeRange(range: TimeRange): void
  
  // Get current time range
  getTimeRange(): TimeRange
  
  // Event: time range changed
  onTimeRangeChanged: (range: TimeRange) => void
}
```

**Data Contract**: Accepts `MetricEntry[]` and `TimeRange` as defined in data-model.md.

---

### MetricSelector Component

**Purpose**: Allows users to select which metrics to display.

**Interface**:
```javascript
class MetricSelector {
  // Set available metrics
  setAvailableMetrics(metrics: string[]): void
  
  // Get selected metrics
  getSelectedMetrics(): string[]
  
  // Set selected metrics
  setSelectedMetrics(metrics: string[]): void
  
  // Event: selection changed
  onSelectionChanged: (selected: string[]) => void
}
```

---

### FileWatcher Component

**Purpose**: Polls directory for new/changed Arrow IPC files.

**Interface**:
```javascript
class FileWatcher {
  // Start watching directory
  startWatching(directory: FileSystemDirectoryHandle | File[], intervalMs: number): void
  
  // Stop watching
  stopWatching(): void
  
  // Check for new/changed files
  async checkForChanges(): Promise<File[]>
  
  // Event: new file detected
  onNewFile: (file: File) => void
  
  // Event: file changed
  onFileChanged: (file: File) => void
}
```

---

## Data Flow Contracts

### File Reading Flow

```
FileWatcher → FileReader.readFile() → ArrowReader.parse() → DuckDBClient.registerArrowFile()
```

**Contract**: 
- FileWatcher emits file events
- FileReader returns ArrayBuffer
- ArrowReader returns Arrow Table
- DuckDBClient registers table and returns table name

---

### Query Flow

```
UI Component → DuckDBClient.query() → Query Results → Component Update
```

**Contract**:
- UI Component constructs SQL query with filters
- DuckDBClient executes query and returns array of results
- Results match expected schema (TraceEntry or MetricEntry)
- Component updates UI with results

---

### Real-time Update Flow

```
FileWatcher (polling) → FileReader → ArrowReader → DuckDBClient → Query → UI Update
```

**Contract**:
- Polling interval: 1 second (configurable)
- Updates batched via requestAnimationFrame
- UI updates are non-blocking (Web Workers)

---

## Web Worker Contracts

### DataWorker Interface

**Purpose**: Web Worker for non-blocking file I/O and DuckDB queries.

**Message Types**:

```typescript
// From main thread to worker
interface WorkerRequest {
  type: 'READ_FILE' | 'QUERY' | 'REGISTER_FILE' | 'UNREGISTER_TABLE'
  payload: any
  id: string // Request ID for response matching
}

// From worker to main thread
interface WorkerResponse {
  type: 'FILE_READ' | 'QUERY_RESULT' | 'FILE_REGISTERED' | 'ERROR'
  payload: any
  id: string // Matches request ID
  error?: string
}
```

**Contract**:
- All file I/O and DuckDB operations happen in worker
- Main thread sends requests via postMessage
- Worker sends responses via postMessage
- Request/response matching via ID

---

## Arrow IPC Schema Contracts

### Trace Schema

The dashboard expects Arrow IPC files with the following schema for traces:

```javascript
{
  trace_id: Binary (16 bytes),
  span_id: Binary (8 bytes),
  parent_span_id: Binary (8 bytes, nullable),
  name: Utf8,
  kind: Int32,
  start_time_unix_nano: UInt64,
  end_time_unix_nano: UInt64,
  status_code: Int32,
  status_message: Utf8 (nullable),
  attributes: Utf8 (JSON-encoded, nullable)
}
```

**Contract**: Arrow IPC files must match this schema. Missing columns are handled gracefully.

---

### Metric Schema

The dashboard expects Arrow IPC files with the following schema for metrics:

```javascript
{
  metric_name: Utf8,
  value: Float64,
  timestamp_unix_nano: UInt64,
  metric_type: Utf8,
  attributes: Utf8 (JSON-encoded, nullable)
}
```

**Contract**: Arrow IPC files must match this schema. Missing columns are handled gracefully.

---

## Error Contracts

### Error Types

```javascript
class FileReadError extends Error {
  constructor(file: File, cause: Error)
}

class DuckDBError extends Error {
  constructor(query: string, cause: Error)
}

class ArrowParseError extends Error {
  constructor(file: File, cause: Error)
}
```

**Contract**: All errors are caught and displayed to user via UI error component. Errors do not crash the dashboard.

---

## Configuration Contracts

### Dashboard Configuration

```javascript
interface DashboardConfig {
  pollingIntervalMs: number // Default: 1000
  maxTraces: number // Default: 10000
  maxGraphPoints: number // Default: 10000
  maxLoadedFiles: number // Default: 100
}
```

**Contract**: Configuration is loaded at startup and can be changed via UI settings.

---

## Rust Service Integration Contract

### Dashboard Serving

When `dashboard.enabled: true` in Rust service config:

- Rust service serves static dashboard files from `dashboard/dist/` directory
- HTTP server runs on configurable port (default: 8080)
- Dashboard files served at root path `/`
- No API endpoints required (dashboard reads files directly)

**Contract**: Rust service only serves static files. Dashboard functionality unchanged (still uses direct file access).

