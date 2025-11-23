# Research: Web JS Realtime Dashboard

**Date**: 2024-12-19  
**Feature**: 002-realtime-dashboard

## Research Questions

### 1. DuckDB-wasm Integration with Arrow IPC Files

**Question**: How to efficiently query Arrow IPC Streaming format files using DuckDB-wasm in the browser?

**Context**: Dashboard needs to read and query Arrow IPC files written by the Rust service. DuckDB-wasm provides native Arrow support and efficient querying capabilities.

**Research Findings**:

**Decision**: Use DuckDB-wasm with Arrow IPC file reading via FileReader API or File System Access API. DuckDB-wasm supports Arrow format natively and can query Arrow IPC files directly.

**Rationale**:
- DuckDB-wasm has native Arrow IPC support via `duckdb-wasm` package
- Can read Arrow IPC files directly without conversion
- Efficient columnar querying suitable for time-series and trace data
- Runs entirely in browser (no server required)
- Supports SQL queries for filtering and aggregation

**Alternatives Considered**:
- Apache Arrow JS: Lower-level, requires more manual query logic
- Parquet-wasm: Different format, not what Rust service writes
- Server-side API: Adds complexity, defeats purpose of client-side dashboard

**Implementation Approach**:
1. Use `@duckdb/duckdb-wasm` npm package
2. Initialize DuckDB-wasm instance in browser
3. Register Arrow IPC files as tables using `registerFileBuffer` or `registerFileURL`
4. Query using SQL: `SELECT * FROM arrow_file WHERE ...`
5. Stream query results for real-time updates

**References**:
- DuckDB-wasm documentation: https://duckdb.org/docs/api/wasm
- Arrow IPC format: https://arrow.apache.org/docs/format/Columnar.html#ipc-streaming-format
- DuckDB Arrow integration: https://duckdb.org/docs/data/arrow

---

### 2. Plotly.js for Realtime Metrics Visualization

**Question**: How to use Plotly.js for real-time time-series metric visualization with efficient updates?

**Context**: Dashboard needs to display OTLP metrics as interactive time-series graphs that update in real-time as new data arrives.

**Research Findings**:

**Decision**: Use Plotly.js with `plotly.js-dist-min` for time-series graphs with real-time data updates via `Plotly.extendTraces` and `Plotly.addTraces`.

**Rationale**:
- Plotly.js provides excellent time-series visualization capabilities
- Supports real-time updates without full re-rendering
- Interactive features (zoom, pan, hover) built-in
- Handles large datasets efficiently with WebGL rendering
- Well-documented and widely used

**Alternatives Considered**:
- Chart.js: Less suitable for time-series, fewer features
- D3.js: More flexible but requires more code, steeper learning curve
- Apache ECharts: Good alternative but Plotly.js has better real-time update APIs

**Implementation Approach**:
1. Use `plotly.js-dist-min` npm package (minified for smaller bundle)
2. Create time-series graphs with `Plotly.newPlot` for initial render
3. Update graphs in real-time using `Plotly.extendTraces` for appending new data points
4. Use `Plotly.relayout` for time range changes
5. Configure graphs with appropriate time-series layout options

**References**:
- Plotly.js documentation: https://plotly.com/javascript/
- Real-time updates: https://plotly.com/javascript/plotlyjs-function-reference/#plotly-extendtraces
- Time-series examples: https://plotly.com/javascript/time-series/

---

### 3. Browser File System Access for Arrow IPC Files

**Question**: How to access Arrow IPC files from the browser - direct file system access or via API?

**Context**: Dashboard needs to read Arrow IPC files from the output directory. Options include direct file system access (File System Access API) or serving files via HTTP.

**Research Findings**:

**Decision**: Support both File System Access API (for direct file system access) and FileReader API (for file selection), with polling-based file watching for detecting new files.

**Rationale**:
- File System Access API provides direct access to directories (Chrome, Edge)
- FileReader API provides fallback for browsers without File System Access API
- Polling is reliable across all browsers (File System Access API watch() has limited support)
- Allows users to select output directory or individual files

**Alternatives Considered**:
- HTTP API: Requires server-side component, adds complexity
- WebSocket streaming: Requires server-side component, defeats client-side purpose
- IndexedDB caching: Useful for caching but not for initial file access

**Implementation Approach**:
1. Use File System Access API `window.showDirectoryPicker()` for directory selection (Chrome, Edge)
2. Fallback to `<input type="file" webkitdirectory>` for directory selection (Firefox, Safari)
3. Use FileReader API to read selected files
4. Poll directory for new files every 1 second (configurable)
5. Track loaded files to avoid re-reading
6. Use File System Access API `watch()` if available for event-based updates (future enhancement)

**References**:
- File System Access API: https://web.dev/file-system-access/
- FileReader API: https://developer.mozilla.org/en-US/docs/Web/API/FileReader
- File watching: https://web.dev/file-system-access/#watch-for-changes-to-a-file-or-directory

---

### 4. Arrow IPC Streaming Format Reading in Browser

**Question**: How to read Arrow IPC Streaming format files in the browser efficiently?

**Context**: Rust service writes Arrow IPC Streaming format files. Dashboard needs to read and parse these files in the browser.

**Research Findings**:

**Decision**: Use Apache Arrow JS (`apache-arrow`) for reading Arrow IPC Streaming format files, with DuckDB-wasm for querying.

**Rationale**:
- Apache Arrow JS provides native Arrow IPC Streaming format support
- Can read Arrow IPC files directly from ArrayBuffer or Blob
- Efficient columnar data access
- DuckDB-wasm can consume Arrow data directly
- Well-maintained and compatible with Rust Arrow format

**Alternatives Considered**:
- Manual parsing: Too complex, error-prone
- Server-side conversion: Defeats client-side purpose
- Parquet format: Different format, not what Rust service writes

**Implementation Approach**:
1. Use `apache-arrow` npm package for Arrow IPC reading
2. Read file as ArrayBuffer via FileReader API
3. Use `arrow.Table.from()` or `arrow.RecordBatchStreamReader` for parsing
4. Convert to DuckDB-wasm compatible format or use directly
5. Handle streaming format (multiple RecordBatches per file)

**References**:
- Apache Arrow JS: https://arrow.apache.org/docs/js/
- Arrow IPC format: https://arrow.apache.org/docs/format/Columnar.html#ipc-streaming-format
- Arrow JS IPC reading: https://arrow.apache.org/docs/js/ipc.html

---

### 5. Real-time Data Streaming Architecture

**Question**: How to efficiently stream data from Arrow IPC files in real-time without blocking the UI?

**Context**: Dashboard needs to continuously read new data from Arrow IPC files and update the UI without performance degradation.

**Research Findings**:

**Decision**: Use Web Workers for file reading and DuckDB querying, with message passing to main thread for UI updates. Use requestAnimationFrame for batched UI updates.

**Rationale**:
- Web Workers prevent blocking main thread during file I/O and queries
- Message passing enables efficient data transfer
- requestAnimationFrame batches UI updates for smooth rendering
- Polling interval (1s) balances responsiveness with performance

**Alternatives Considered**:
- Main thread processing: Blocks UI, poor user experience
- Service Workers: Overkill for file reading, adds complexity
- WebAssembly threads: More complex, DuckDB-wasm handles this internally

**Implementation Approach**:
1. Create Web Worker for file reading and DuckDB queries
2. Poll for new files every 1 second in Web Worker
3. Read new/changed files in Web Worker
4. Query DuckDB-wasm in Web Worker
5. Send query results to main thread via postMessage
6. Batch UI updates using requestAnimationFrame
7. Use virtual scrolling for large trace lists

**References**:
- Web Workers: https://developer.mozilla.org/en-US/docs/Web/API/Web_Workers_API
- requestAnimationFrame: https://developer.mozilla.org/en-US/docs/Web/API/window/requestAnimationFrame
- Virtual scrolling: https://web.dev/virtualize-long-lists-react-window/

---

### 6. Trace Display and Filtering

**Question**: How to efficiently display and filter large numbers of traces in a live tail view?

**Context**: Dashboard needs to display traces in a scrollable list with filtering capabilities, handling potentially thousands of traces.

**Research Findings**:

**Decision**: Use virtual scrolling (react-window or similar) for trace list, with client-side filtering via DuckDB-wasm queries. Display traces in table format with expandable detail view.

**Rationale**:
- Virtual scrolling renders only visible items, enabling large lists
- DuckDB-wasm provides efficient SQL-based filtering
- Table format is familiar and scannable
- Expandable details prevent information overload

**Alternatives Considered**:
- Full list rendering: Performance issues with large datasets
- Server-side filtering: Requires server, defeats client-side purpose
- Infinite scroll: Less suitable for live tail (new items at top)

**Implementation Approach**:
1. Use virtual scrolling library (e.g., `react-window` or vanilla JS implementation)
2. Query DuckDB-wasm with SQL WHERE clauses for filtering
3. Display traces in table: trace ID, span name, service, duration, status, timestamp
4. Click to expand detail view: full attributes, events, span hierarchy
5. Auto-scroll to newest traces (with option to pause)
6. Highlight error traces with visual indicators

**References**:
- react-window: https://github.com/bvaughn/react-window
- Virtual scrolling: https://web.dev/virtualize-long-lists-react-window/

---

### 7. Metrics Time-Series Graph Design

**Question**: How to design time-series graphs for OTLP metrics with multiple metric types and labels?

**Context**: Dashboard needs to display various OTLP metric types (gauge, counter, histogram) as time-series graphs with proper aggregation and labeling.

**Research Findings**:

**Decision**: Use Plotly.js with separate graph per metric type, with support for multiple series per graph (different labels). Use appropriate aggregation for histogram metrics.

**Rationale**:
- Plotly.js handles multiple series per graph well
- Separate graphs per metric type reduces complexity
- Proper aggregation (sum, avg, min, max) for histogram metrics
- Labels can be shown in legend and tooltips

**Alternatives Considered**:
- Single graph with all metrics: Too cluttered, hard to read
- Separate graphs per label: Too many graphs, overwhelming
- Tabbed interface: Less efficient for comparing metrics

**Implementation Approach**:
1. Group metrics by metric name
2. Create one Plotly graph per metric name
3. Multiple series per graph for different label combinations
4. Use appropriate aggregation for histogram metrics (avg, sum, p50, p95, p99)
5. Support metric selection (show/hide specific metrics)
6. Time range selector affects all graphs
7. Hover tooltips show full label information

**References**:
- Plotly.js time-series: https://plotly.com/javascript/time-series/
- OTLP metric types: https://opentelemetry.io/docs/specs/otel/metrics/data-model/

---

## Technology Stack Summary

- **Frontend Framework**: Vanilla JavaScript or lightweight framework (React/Vue) - TBD based on complexity
- **Arrow IPC Reading**: `apache-arrow` npm package
- **Database/Querying**: `@duckdb/duckdb-wasm` for Arrow IPC querying
- **Visualization**: `plotly.js-dist-min` for metric graphs
- **File Access**: File System Access API (Chrome/Edge) + FileReader API (fallback)
- **Web Workers**: For file I/O and DuckDB queries
- **Virtual Scrolling**: For large trace lists
- **Build Tool**: Vite or similar for modern JS bundling

## Open Questions

1. Should we use a frontend framework (React/Vue) or vanilla JavaScript?
   - **Decision Needed**: Evaluate complexity - if simple, vanilla JS; if complex UI, consider React
2. How should we handle authentication/authorization?
   - **Decision**: Not required for initial version, but architecture should allow future addition
3. Should we support exporting trace/metric data?
   - **Decision**: Not required for initial version, but architecture should allow future addition
4. How should we handle schema evolution in Arrow IPC files?
   - **Decision**: Dashboard should handle missing columns gracefully, show warnings for schema mismatches

