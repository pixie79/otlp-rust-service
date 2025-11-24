# OTLP Realtime Dashboard

A web-based real-time dashboard for monitoring OTLP traces and metrics from Arrow IPC files directly in your browser.

## Features

- **Live Trace Tail Viewer**: Real-time display of trace data with filtering and detail pane
- **Real-time Metrics Graphing**: Dynamic time-series visualization using Plotly.js
- **Arrow IPC File Streaming**: Direct client-side reading of Arrow IPC files
- **DuckDB-wasm Integration**: Efficient in-browser querying using DuckDB WebAssembly
- **Interactive UI**: Responsive design with keyboard navigation and accessibility features
- **Configurable Settings**: Adjust polling interval, data limits, and memory management

## Quick Start

### Standalone Usage

1. Open `index.html` in a modern browser (Chrome, Firefox, Safari, or Edge - latest 2 versions)
2. Click "Choose Directory" and select the OTLP output directory
3. The dashboard will automatically start monitoring for new Arrow IPC files

### Served by Rust Service

1. Configure the Rust service with `dashboard.enabled: true` in your YAML config:

```yaml
dashboard:
  enabled: true
  port: 8080
  static_dir: ./dashboard/dist
```

2. Start the Rust service
3. Access the dashboard at `http://localhost:8080`

## Architecture

The dashboard is a client-side web application that:

- Reads Arrow IPC files directly from the file system using File System Access API or FileReader API
- Uses DuckDB-wasm to query Arrow IPC data efficiently in the browser
- Renders traces and metrics using vanilla JavaScript with Plotly.js for graphs
- Runs heavy operations (file I/O, DuckDB queries) in Web Workers to keep the UI responsive

## Components

### Core Components

- **App**: Main application orchestrator
- **FileReader**: Handles reading Arrow IPC files
- **FileWatcher**: Monitors directory for new files
- **DuckDBClient**: Manages DuckDB-wasm instance and connections
- **QueryExecutor**: Executes SQL queries against DuckDB

### UI Components

- **Layout**: Main layout structure with responsive design
- **Navigation**: View switching between traces and metrics
- **TraceList**: Virtual scrolling list of traces
- **TraceDetail**: Detail pane for selected trace
- **TraceFilter**: Filtering controls for traces
- **MetricGraph**: Plotly.js-based time-series graphs
- **MetricSelector**: UI for selecting metrics to display
- **Search**: Search bar for trace IDs and metric names
- **TimeRangeSelector**: Time range selection for metrics
- **Settings**: Configuration UI for polling interval and data limits
- **Loading**: Loading indicators and error messages

## Keyboard Shortcuts

- `Ctrl+1` / `Cmd+1`: Switch to Traces view
- `Ctrl+2` / `Cmd+2`: Switch to Metrics view
- `/`: Focus search bar
- `Escape`: Clear search
- `Ctrl+P` / `Cmd+P`: Pause/Resume live stream

## Configuration

Access settings via the Settings button (⚙️) in the header to configure:

- **Polling Interval**: How often to check for new files (default: 1000ms)
- **Max Traces**: Maximum number of traces to keep in memory (default: 10000)
- **Max Graph Points**: Maximum data points per metric graph (default: 10000)
- **Max Loaded Tables**: Maximum DuckDB tables in memory (default: 50)

## Browser Requirements

- WebAssembly support
- Web Workers
- LocalStorage
- File System Access API or FileReader API

Supported browsers: Chrome, Firefox, Safari, Edge (latest 2 versions)

## Development

### Building

The dashboard uses vanilla JavaScript with ES modules. No build step is required for development.

For production, you may want to:

1. Bundle JavaScript files
2. Minify CSS and JavaScript
3. Optimize assets

### Testing

```bash
# Run unit tests
npm test

# Run integration tests
npm run test:integration

# Run E2E tests
npm run test:e2e
```

## Examples

See `examples/basic-usage.html` for usage examples.

## Performance

The dashboard is optimized for performance:

- Virtual scrolling for large trace lists
- Plotly `extendTraces` for efficient graph updates
- LRU cache for DuckDB tables
- Web Workers for heavy operations
- Incremental file reading for large files

## Accessibility

The dashboard follows WCAG 2.1 AA guidelines:

- Keyboard navigation support
- ARIA labels and roles
- Screen reader support
- High contrast mode support

## License

MIT OR Apache-2.0
