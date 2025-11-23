# Implementation Plan: Web JS Realtime Dashboard

**Branch**: `002-realtime-dashboard` | **Date**: 2024-12-19 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/002-realtime-dashboard/spec.md`

**Note**: This template is filled in by the `/speckit.plan` command. See `.specify/templates/commands/plan.md` for the execution workflow.

## Summary

Build a web-based real-time dashboard that streams Arrow IPC Flight files to provide a live tail of OTLP traces and real-time graphing of OTLP metrics. The dashboard runs entirely in the browser, uses DuckDB-wasm for efficient Arrow IPC file querying, and Plotly.js for interactive metric visualization. The dashboard reads Arrow IPC files directly from the output directory using File System Access API or FileReader API, with polling-based file watching for real-time updates.

## Technical Context

**Language/Version**: JavaScript/TypeScript (ES2020+), HTML5, CSS3  
**Primary Dependencies**: 
- `@duckdb/duckdb-wasm` - DuckDB WebAssembly for Arrow IPC querying
- `apache-arrow` - Arrow IPC file reading and parsing
- `plotly.js-dist-min` - Interactive time-series visualization
- `vite` or `webpack` - Modern JavaScript bundling
- Optional: `react` / `vue` - Frontend framework (TBD based on complexity)

**Storage**: Client-side only - reads Arrow IPC files from file system  
**Testing**: Jest/Vitest for unit tests, Playwright for E2E tests  
**Target Platform**: Modern browsers with WebAssembly support (Chrome, Firefox, Safari, Edge - latest 2 versions)  
**Project Type**: Standalone web application (can be served statically or via simple HTTP server)  
**Performance Goals**: 
- Display new traces/metrics within 1 second (p95 latency)
- Handle 100+ Arrow IPC files simultaneously
- Display 10,000+ traces with smooth scrolling
- DuckDB queries complete within 500ms (p95)
- UI remains responsive (60fps) during streaming

**Constraints**: 
- Must run entirely in browser (no server-side processing)
- Must work with Arrow IPC Streaming format files from Rust service
- Must support File System Access API (Chrome/Edge) and FileReader API (fallback)
- Must handle large files (up to 1GB) without browser memory issues
- Must be responsive and accessible (WCAG 2.1 AA minimum)

**Scale/Scope**: 
- Single-page web application
- Separate from Rust service (reads files from output directory)
- Modular architecture: file reading, DuckDB querying, UI components, visualization
- Can be served as static files or via simple HTTP server

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

Verify compliance with OTLP Rust Service Constitution principles:

- **Code Quality (I)**: ✅ Design follows JavaScript/TypeScript best practices with modular organization, comprehensive documentation, and separation of concerns. Complexity managed through component separation (file reading, querying, UI, visualization).
- **Testing Standards (II)**: ✅ Testing strategy defined with unit tests (Jest/Vitest) and E2E tests (Playwright). Coverage targets: 80% minimum. Test types: unit (fast, isolated), integration (component interaction), E2E (user workflows).
- **User Experience Consistency (III)**: ✅ UI/UX patterns consistent with modern web dashboard standards. Error handling and loading states standardized. Responsive design for desktop and tablet.
- **Performance Requirements (IV)**: ✅ Performance targets defined: 1s latency for data display, 500ms for queries, 60fps UI. Virtual scrolling for large lists. Web Workers for non-blocking I/O.
- **Observability & Reliability (V)**: ✅ Error handling and logging for file read errors, query failures. Loading states and error messages for users. Graceful degradation for unsupported browsers.

Any violations or exceptions MUST be documented in the Complexity Tracking section below.

## Project Structure

### Documentation (this feature)

```text
specs/002-realtime-dashboard/
├── plan.md              # This file (/speckit.plan command output)
├── research.md          # Phase 0 output (/speckit.plan command)
├── data-model.md        # Phase 1 output (/speckit.plan command)
├── quickstart.md        # Phase 1 output (/speckit.plan command)
├── contracts/           # Phase 1 output (/speckit.plan command)
│   ├── ui-components.md # UI component contracts
│   └── rust-service-config.md # Rust service dashboard configuration contract
└── tasks.md             # Phase 2 output (/speckit.tasks command - NOT created by /speckit.plan)
```

### Source Code (new dashboard directory)

```text
dashboard/
├── package.json         # npm dependencies and scripts
├── vite.config.js       # Vite configuration (or webpack.config.js)
├── index.html           # Main HTML entry point
├── src/
│   ├── main.js          # Application entry point
│   ├── app.js           # Main application logic
│   ├── config.js        # Configuration (polling interval, etc.)
│   ├── file/
│   │   ├── file-reader.js      # File reading via FileReader API
│   │   ├── file-system-api.js  # File System Access API wrapper
│   │   └── file-watcher.js     # File watching/polling logic
│   ├── duckdb/
│   │   ├── duckdb-client.js    # DuckDB-wasm initialization and management
│   │   └── query-executor.js   # SQL query execution
│   ├── arrow/
│   │   └── arrow-reader.js     # Arrow IPC file reading and parsing
│   ├── traces/
│   │   ├── trace-list.js       # Trace list component
│   │   ├── trace-detail.js     # Trace detail view component
│   │   └── trace-filter.js     # Trace filtering logic
│   ├── metrics/
│   │   ├── metric-graph.js     # Plotly.js graph component
│   │   ├── metric-selector.js  # Metric selection component
│   │   └── metric-aggregator.js # Metric aggregation logic
│   ├── ui/
│   │   ├── layout.js            # Main layout component
│   │   ├── navigation.js        # Navigation between views
│   │   └── loading.js           # Loading state component
│   └── workers/
│       └── data-worker.js      # Web Worker for file I/O and queries
├── styles/
│   ├── main.css         # Main stylesheet
│   └── components.css   # Component-specific styles
└── tests/
    ├── unit/            # Unit tests
    ├── integration/    # Integration tests
    └── e2e/            # E2E tests (Playwright)
```

**Structure Decision**: Standalone web application in `dashboard/` directory. Modular architecture with separation of concerns: file reading, DuckDB querying, UI components, visualization. Web Workers for non-blocking operations. Can be served statically or via simple HTTP server.

## Complexity Tracking

> **Fill ONLY if Constitution Check has violations that must be justified**

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|-------------------------------------|
| DuckDB-wasm integration | Efficient Arrow IPC querying requires database-like capabilities | Manual Arrow parsing insufficient - need SQL queries for filtering and aggregation |
| Web Workers | File I/O and queries must not block UI thread | Main thread processing insufficient - causes UI freezing with large files |
| Virtual scrolling | Large trace lists require efficient rendering | Full list rendering insufficient - performance issues with 10,000+ traces |
| Dual file access APIs | Browser compatibility requires both File System Access API and FileReader API | Single API insufficient - File System Access API not available in all browsers |

## Architecture Overview

### Data Flow

1. **File Discovery**: Poll output directory every 1 second (configurable) for new Arrow IPC files
2. **File Reading**: Read Arrow IPC files via File System Access API or FileReader API
3. **Arrow Parsing**: Parse Arrow IPC Streaming format using `apache-arrow`
4. **DuckDB Registration**: Register Arrow data in DuckDB-wasm as tables
5. **Query Execution**: Execute SQL queries in DuckDB-wasm (filtering, aggregation)
6. **Data Streaming**: Stream query results to UI components
7. **UI Updates**: Update trace list and metric graphs via requestAnimationFrame batching

### Component Interactions

- **File Watcher** → **File Reader** → **Arrow Reader** → **DuckDB Client** → **Query Executor**
- **Query Executor** → **Trace List** / **Metric Graph** (via message passing)
- **Trace List** → **Trace Detail** (on click)
- **Metric Graph** → **Plotly.js** (for rendering)

### Web Worker Architecture

- **Main Thread**: UI rendering, user interactions, Plotly.js graphs
- **Web Worker**: File reading, Arrow parsing, DuckDB queries, data processing
- **Communication**: postMessage for data transfer between worker and main thread

## Technology Decisions

### Frontend Framework

**Decision**: Start with vanilla JavaScript, evaluate React/Vue if complexity increases.

**Rationale**: 
- Dashboard may be simple enough for vanilla JS
- Reduces bundle size and complexity
- Can migrate to framework later if needed
- Faster initial development

### Build Tool

**Decision**: Use Vite for modern JavaScript bundling.

**Rationale**:
- Fast development server
- Modern ES module support
- Good TypeScript support (if we add types)
- Smaller bundle size than webpack

### File Access Strategy

**Decision**: Support both File System Access API (Chrome/Edge) and FileReader API (fallback).

**Rationale**:
- File System Access API provides best UX (directory access)
- FileReader API provides fallback for all browsers
- Polling works reliably across all browsers
- User can select directory or individual files

### Data Querying Strategy

**Decision**: Use DuckDB-wasm for all Arrow IPC querying.

**Rationale**:
- Native Arrow IPC support
- Efficient SQL queries for filtering and aggregation
- Runs entirely in browser
- Better performance than manual Arrow parsing

## Performance Considerations

### Large File Handling

- Stream Arrow IPC files incrementally (don't load entire file into memory)
- Use DuckDB-wasm's efficient columnar processing
- Virtual scrolling for trace lists (render only visible items)
- Limit graph data points (e.g., max 10,000 points per graph)

### Real-time Updates

- Poll for new files every 1 second (configurable)
- Batch UI updates using requestAnimationFrame
- Use Plotly.js `extendTraces` for efficient graph updates (no full re-render)
- Web Workers prevent blocking main thread

### Memory Management

- Unregister old DuckDB tables when files are no longer needed
- Limit in-memory trace/metric data (e.g., keep last 10,000 traces)
- Clear old graph data points (sliding window)
- Use WeakMap for component references

## Security Considerations

- File System Access API requires user permission (secure by default)
- FileReader API only reads user-selected files (no arbitrary file access)
- No server-side component (no network security concerns)
- DuckDB-wasm runs in sandboxed WebAssembly environment
- No sensitive data stored in browser (reads files on-demand)

## Browser Compatibility

### Required Features

- WebAssembly support (DuckDB-wasm requirement)
- FileReader API or File System Access API
- ES2020+ JavaScript features
- CSS Grid/Flexbox for layout

### Supported Browsers

- Chrome 90+ (File System Access API)
- Edge 90+ (File System Access API)
- Firefox 88+ (FileReader API fallback)
- Safari 14+ (FileReader API fallback)

### Fallbacks

- FileReader API for browsers without File System Access API
- Manual file selection if directory picker not available
- Graceful degradation for unsupported features

## Testing Strategy

### Unit Tests

- File reading logic (mocked file system)
- Arrow IPC parsing
- DuckDB query execution (mocked DuckDB)
- Filtering and aggregation logic
- UI component rendering

### Integration Tests

- File reading → Arrow parsing → DuckDB registration flow
- Query execution → data transformation → UI update flow
- Web Worker communication
- Plotly.js graph updates

### E2E Tests (Playwright)

- User selects directory and sees traces/metrics
- User filters traces and sees filtered results
- User clicks trace and sees detail view
- User changes time range and sees updated graphs
- User interacts with graphs (zoom, pan, hover)

## Deployment

### Static File Serving

- Build dashboard with `npm run build`
- Serve `dist/` directory via any static file server
- No server-side processing required
- Can be served from CDN

### Development Server

- Use Vite dev server for development
- Hot module replacement for fast iteration
- Proxy for CORS if needed (unlikely for file access)

## Future Enhancements (Out of Scope)

- Authentication/authorization
- Export trace/metric data
- Server-side API for remote file access
- Real-time collaboration (multiple users)
- Custom dashboard layouts
- Alerting based on metrics
- Trace comparison and diffing

