# Tasks: Web JS Realtime Dashboard

**Input**: Design documents from `/specs/002-realtime-dashboard/`
**Prerequisites**: plan.md (required), spec.md (required for user stories), research.md, data-model.md, contracts/

**Tests**: Tests are included for critical components. Unit tests use Jest/Vitest, E2E tests use Playwright.

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3, US4)
- Include exact file paths in descriptions

## Path Conventions

- **Dashboard project**: `dashboard/` directory at repository root
- Paths follow plan.md structure: `dashboard/src/`, `dashboard/tests/`, `dashboard/styles/`
- **Rust service integration**: `src/config/`, `src/bin/` for dashboard serving

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Dashboard project initialization and basic structure

- [x] T001 Create dashboard directory structure per implementation plan: dashboard/src/, dashboard/tests/, dashboard/styles/
- [x] T002 Initialize npm project with package.json in dashboard/ directory
- [x] T003 [P] Add dependencies to dashboard/package.json: @duckdb/duckdb-wasm, apache-arrow, plotly.js-dist-min, vite
- [x] T004 [P] Add dev dependencies to dashboard/package.json: vitest, @vitest/ui, playwright, @playwright/test
- [x] T005 [P] Configure vite.config.js in dashboard/ with ES module support and build settings
- [x] T006 [P] Create dashboard/index.html with main entry point and basic structure
- [x] T007 [P] Create dashboard/.gitignore with Node.js patterns (node_modules/, dist/, etc.)
- [x] T008 [P] Create dashboard/README.md with project overview and setup instructions
- [x] T009 [P] Setup test directory structure: dashboard/tests/unit/, dashboard/tests/integration/, dashboard/tests/e2e/
- [x] T010 [P] Configure vitest.config.js in dashboard/ for unit and integration tests
- [x] T011 [P] Configure playwright.config.js in dashboard/ for E2E tests

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core infrastructure that MUST be complete before ANY user story can be implemented

**⚠️ CRITICAL**: No user story work can begin until this phase is complete

- [x] T012 Create configuration module in dashboard/src/config.js with polling interval, max traces, max graph points settings
- [x] T013 [P] Create error types in dashboard/src/error.js with FileReadError, DuckDBError, ArrowParseError classes
- [x] T014 [P] Create FileReader component in dashboard/src/file/file-reader.js with readFile, listFiles, getFileMetadata methods per contracts/ui-components.md
- [x] T015 [P] Create FileSystemAPI wrapper in dashboard/src/file/file-system-api.js with selectDirectory method using File System Access API
- [x] T016 [P] Create FileWatcher component in dashboard/src/file/file-watcher.js with startWatching, stopWatching, checkForChanges methods per contracts/ui-components.md
- [x] T017 [P] Create ArrowReader component in dashboard/src/arrow/arrow-reader.js with parse method for Arrow IPC Streaming format using apache-arrow
- [x] T018 [P] Create DuckDBClient component in dashboard/src/duckdb/duckdb-client.js with initialize, registerArrowFile, query, unregisterTable, close methods per contracts/ui-components.md
- [x] T019 [P] Create QueryExecutor component in dashboard/src/duckdb/query-executor.js with SQL query execution and result transformation
- [x] T020 [P] Create DataWorker Web Worker in dashboard/src/workers/data-worker.js for file I/O and DuckDB queries per contracts/ui-components.md
- [x] T021 [P] Create main application entry point in dashboard/src/main.js with initialization logic
- [x] T022 [P] Create app.js in dashboard/src/app.js with main application state management
- [x] T023 [P] Create base CSS styles in dashboard/styles/main.css with layout, typography, and color scheme
- [x] T024 [P] Create component CSS in dashboard/styles/components.css with component-specific styles

**Checkpoint**: Foundation ready - user story implementation can now begin

---

## Phase 3: User Story 1 - Live Trace Tail Viewer (Priority: P1) 🎯 MVP

**Goal**: Display OTLP traces in a live tail view that updates automatically as new traces arrive from Arrow IPC files

**Independent Test**: Start the dashboard, generate trace data, and verify that traces appear in the dashboard in real-time with correct formatting and filtering capabilities

### Tests for User Story 1 ⚠️

> **NOTE: Write these tests FIRST, ensure they FAIL before implementation**

- [x] T025 [P] [US1] Unit test for TraceList component in dashboard/tests/unit/traces/test-trace-list.js
- [x] T026 [P] [US1] Unit test for TraceDetail component in dashboard/tests/unit/traces/test-trace-detail.js
- [x] T027 [P] [US1] Unit test for TraceFilter component in dashboard/tests/unit/traces/test-trace-filter.js
- [x] T028 [P] [US1] Integration test for trace data flow (file reading → Arrow parsing → DuckDB → UI) in dashboard/tests/integration/test-trace-data-flow.js
- [x] T029 [P] [US1] E2E test for live trace tail viewer in dashboard/tests/e2e/test-trace-tail-viewer.spec.js

### Implementation for User Story 1

- [x] T030 [P] [US1] Create TraceEntry data model in dashboard/src/traces/trace-entry.js per data-model.md
- [x] T031 [P] [US1] Create TraceList component in dashboard/src/traces/trace-list.js with setTraces, applyFilters, getSelectedTrace methods per contracts/ui-components.md
- [x] T032 [US1] Implement virtual scrolling in dashboard/src/traces/trace-list.js for efficient rendering of large trace lists (10,000+ traces)
- [x] T033 [US1] Create TraceDetail component in dashboard/src/traces/trace-detail.js with showTrace, clear methods per contracts/ui-components.md
- [x] T034 [P] [US1] Create TraceFilter component in dashboard/src/traces/trace-filter.js with filtering by trace ID, service name, span name, error status, time range per FR-007
- [x] T035 [US1] Implement trace data querying in dashboard/src/traces/trace-query.js with DuckDB SQL queries for filtering traces
- [x] T036 [US1] Integrate TraceList with DataWorker for real-time trace updates in dashboard/src/traces/trace-list.js
- [x] T037 [US1] Implement trace detail view with span hierarchy, attributes, events, and timing in dashboard/src/traces/trace-detail.js per FR-009
- [x] T038 [US1] Add trace list UI to main layout in dashboard/src/ui/layout.js
- [x] T039 [US1] Implement chronological merging of traces from multiple Arrow IPC files in dashboard/src/traces/trace-query.js per FR-012

**Checkpoint**: At this point, User Story 1 should be fully functional. Traces can be displayed in live tail view with filtering and detail view.

---

## Phase 4: User Story 2 - Realtime Metrics Graphing (Priority: P1)

**Goal**: Display OTLP metrics as time-series graphs that update automatically as new metric data arrives from Arrow IPC files

**Independent Test**: Start the dashboard, generate metric data, and verify that metrics appear in graphs with correct values, labels, and time-series behavior

### Tests for User Story 2 ⚠️

- [x] T040 [P] [US2] Unit test for MetricGraph component in dashboard/tests/unit/metrics/test-metric-graph.js
- [x] T041 [P] [US2] Unit test for MetricSelector component in dashboard/tests/unit/metrics/test-metric-selector.js
- [x] T042 [P] [US2] Unit test for MetricAggregator component in dashboard/tests/unit/metrics/test-metric-aggregator.js
- [x] T043 [P] [US2] Integration test for metric data flow (file reading → Arrow parsing → DuckDB → Plotly.js) in dashboard/tests/integration/test-metric-data-flow.js
- [x] T044 [P] [US2] E2E test for realtime metrics graphing in dashboard/tests/e2e/test-metrics-graphing.spec.js

### Implementation for User Story 2

- [x] T045 [P] [US2] Create MetricEntry data model in dashboard/src/metrics/metric-entry.js per data-model.md
- [x] T046 [P] [US2] Create MetricGraph component in dashboard/src/metrics/metric-graph.js with updateMetric, removeMetric, setTimeRange methods per contracts/ui-components.md
- [x] T047 [US2] Integrate Plotly.js in dashboard/src/metrics/metric-graph.js with time-series graph rendering per FR-006
- [x] T048 [US2] Implement real-time graph updates using Plotly.js extendTraces in dashboard/src/metrics/metric-graph.js for efficient updates
- [x] T049 [P] [US2] Create MetricSelector component in dashboard/src/metrics/metric-selector.js with setAvailableMetrics, getSelectedMetrics, setSelectedMetrics methods per contracts/ui-components.md
- [x] T050 [P] [US2] Create MetricAggregator component in dashboard/src/metrics/metric-aggregator.js with aggregation logic for histogram metrics (sum, avg, min, max, p50, p95, p99) per data-model.md
- [x] T051 [US2] Implement metric data querying in dashboard/src/metrics/metric-query.js with DuckDB SQL queries for filtering and aggregating metrics per FR-008
- [x] T052 [US2] Integrate MetricGraph with DataWorker for real-time metric updates in dashboard/src/metrics/metric-graph.js
- [x] T053 [US2] Implement interactive graph features (zoom, pan, hover tooltips) in dashboard/src/metrics/metric-graph.js per FR-010
- [x] T054 [US2] Add metric graphs UI to main layout in dashboard/src/ui/layout.js
- [x] T055 [US2] Implement time range selection for metrics in dashboard/src/metrics/metric-graph.js per FR-014
- [x] T056 [US2] Implement multiple metric types display (gauge, counter, histogram) in dashboard/src/metrics/metric-graph.js per acceptance scenario 5

**Checkpoint**: At this point, User Stories 1 AND 2 should both work independently. Dashboard can display both traces and metrics in real-time.

---

## Phase 5: User Story 3 - Arrow IPC File Streaming (Priority: P2)

**Goal**: Efficiently stream data from Arrow IPC files in real-time without blocking or consuming excessive resources using DuckDB-wasm

**Independent Test**: Monitor the dashboard's resource usage while streaming large Arrow IPC files and verify that data loads efficiently without performance degradation

### Tests for User Story 3 ⚠️

- [x] T057 [P] [US3] Unit test for file streaming with large files (>100MB) in dashboard/tests/unit/file/test-large-file-streaming.js
- [x] T058 [P] [US3] Integration test for DuckDB query performance (<500ms p95) in dashboard/tests/integration/test-duckdb-performance.js
- [x] T059 [P] [US3] Integration test for multiple file handling (100+ files) in dashboard/tests/integration/test-multiple-files.js
- [x] T060 [P] [US3] E2E test for file streaming performance in dashboard/tests/e2e/test-file-streaming.spec.js

### Implementation for User Story 3

- [x] T061 [US3] Implement incremental file reading in dashboard/src/file/file-reader.js to avoid loading entire large files into memory
- [x] T062 [US3] Implement file change detection in dashboard/src/file/file-watcher.js to detect new/changed files efficiently
- [x] T063 [US3] Optimize DuckDB table registration for large Arrow IPC files in dashboard/src/duckdb/duckdb-client.js
- [x] T064 [US3] Implement query result streaming in dashboard/src/duckdb/query-executor.js to avoid blocking on large result sets
- [x] T065 [US3] Implement memory management for DuckDB tables (unregister old tables) in dashboard/src/duckdb/duckdb-client.js per plan.md memory management
- [x] T066 [US3] Add performance monitoring for DuckDB queries in dashboard/src/duckdb/query-executor.js to ensure <500ms p95 latency per SC-006
- [x] T067 [US3] Implement file metadata caching in dashboard/src/file/file-watcher.js to avoid redundant file reads
- [x] T068 [US3] Add error handling for file read errors in dashboard/src/file/file-reader.js per FR-017

**Checkpoint**: At this point, User Stories 1, 2, AND 3 should all work. Dashboard efficiently streams data from Arrow IPC files with good performance.

---

## Phase 6: User Story 4 - Interactive Dashboard UI (Priority: P2)

**Goal**: Provide an intuitive, responsive web interface that allows users to interact with trace and metric data effectively

**Independent Test**: Perform common user interactions (filtering, searching, zooming graphs) and verify that the UI responds quickly and correctly

### Tests for User Story 4 ⚠️

- [x] T069 [P] [US4] Unit test for Layout component in dashboard/tests/unit/ui/test-layout.js
- [x] T070 [P] [US4] Unit test for Navigation component in dashboard/tests/unit/ui/test-navigation.js
- [x] T071 [P] [US4] Unit test for Loading component in dashboard/tests/unit/ui/test-loading.js
- [x] T072 [P] [US4] Integration test for navigation between views in dashboard/tests/integration/test-navigation.js
- [x] T073 [P] [US4] E2E test for interactive UI features in dashboard/tests/e2e/test-interactive-ui.spec.js

### Implementation for User Story 4

- [x] T074 [P] [US4] Create Layout component in dashboard/src/ui/layout.js with main layout structure per contracts/ui-components.md
- [x] T075 [P] [US4] Create Navigation component in dashboard/src/ui/navigation.js with navigation between trace view and metrics view per acceptance scenario 1
- [x] T076 [P] [US4] Create Loading component in dashboard/src/ui/loading.js with loading states and error messages per FR-019
- [x] T077 [US4] Implement responsive layout in dashboard/src/ui/layout.js for desktop and tablet screen sizes per FR-013
- [x] T078 [US4] Implement search functionality in dashboard/src/ui/search.js with search by trace ID or metric name per acceptance scenario 2
- [x] T079 [US4] Implement pause/resume live stream functionality in dashboard/src/app.js per FR-018
- [x] T080 [US4] Add keyboard navigation support in dashboard/src/ui/layout.js per FR-020 (WCAG 2.1 AA)
- [x] T081 [US4] Add screen reader support with ARIA labels in dashboard/src/ui/layout.js per FR-020
- [x] T082 [US4] Implement time range selector UI in dashboard/src/ui/time-range-selector.js for both traces and metrics per FR-014
- [x] T083 [US4] Implement dashboard state persistence (last known position) in dashboard/src/app.js per acceptance scenario 5
- [x] T084 [US4] Add error message display in dashboard/src/ui/loading.js for file read errors, query failures per FR-019

**Checkpoint**: At this point, all user stories should be functional. Dashboard provides a complete, interactive UI for monitoring traces and metrics.

---

## Phase 7: Rust Service Dashboard Integration

**Goal**: Enable Rust service to optionally serve dashboard static files via HTTP server

**Independent Test**: Configure Rust service with dashboard.enabled=true, start service, and verify dashboard is accessible via HTTP

### Tests for Rust Service Integration ⚠️

- [x] T085 [P] Unit test for DashboardConfig in tests/unit/config/test_dashboard_config.rs
- [x] T086 [P] Unit test for dashboard configuration loading from YAML in tests/unit/config/test_dashboard_yaml.rs
- [x] T087 [P] Unit test for dashboard configuration loading from environment variables in tests/test_dashboard_env.rs
- [x] T088 [P] Integration test for dashboard HTTP server startup in tests/integration/test_dashboard_server.rs
- [x] T089 [P] Integration test for dashboard static file serving in tests/integration/test_dashboard_static_files.rs

### Implementation for Rust Service Integration

- [x] T090 [P] Create DashboardConfig struct in src/config/types.rs with enabled, port, static_dir fields per contracts/rust-service-config.md
- [x] T091 [P] Add DashboardConfig to Config struct in src/config/types.rs
- [x] T092 [P] Implement dashboard configuration loading from YAML in src/config/loader.rs per contracts/rust-service-config.md
- [x] T093 [P] Implement dashboard configuration loading from environment variables (OTLP_DASHBOARD_ENABLED, OTLP_DASHBOARD_PORT, OTLP_DASHBOARD_STATIC_DIR) in src/config/loader.rs
- [x] T094 [P] Add dashboard configuration validation in src/config/types.rs (port validation, directory existence check) per contracts/rust-service-config.md
- [x] T095 [P] Create HTTP server module in src/dashboard/server.rs for serving static files
- [x] T096 [P] Implement static file serving in src/dashboard/server.rs with GET requests, content-type detection, index.html for root path
- [x] T097 [P] Integrate dashboard HTTP server startup in src/bin/main.rs when dashboard.enabled is true
- [x] T098 [P] Add dashboard server shutdown logic in src/bin/main.rs for graceful shutdown
- [x] T099 [P] Add logging for dashboard server startup and errors in src/dashboard/server.rs per contracts/rust-service-config.md
- [x] T100 [P] Update ConfigBuilder in src/config/types.rs with dashboard configuration builder methods

**Checkpoint**: At this point, Rust service can optionally serve dashboard files. Dashboard still uses direct file system access for Arrow IPC files.

---

## Phase 8: Polish & Cross-Cutting Concerns

**Purpose**: Improvements that affect multiple user stories

- [x] T101 [P] Add comprehensive documentation comments to all components in dashboard/src/
- [x] T102 [P] Add JSDoc comments to all public methods per contracts/ui-components.md
- [x] T103 [P] Create dashboard usage examples in dashboard/examples/
- [x] T104 [P] Add performance benchmarks for file streaming in dashboard/tests/bench/
- [x] T105 [P] Add performance benchmarks for DuckDB queries in dashboard/tests/bench/
- [x] T106 [P] Add performance benchmarks for UI rendering in dashboard/tests/bench/
- [x] T107 [P] Implement error boundary for unhandled errors in dashboard/src/app.js
- [x] T108 [P] Add browser compatibility detection and graceful degradation in dashboard/src/app.js
- [x] T109 [P] Implement data limits (max traces, max graph points) with cleanup in dashboard/src/app.js per data-model.md memory management
- [x] T110 [P] Add configuration UI for polling interval and data limits in dashboard/src/ui/settings.js
- [x] T111 [P] Update dashboard/README.md with complete usage examples and architecture documentation
- [x] T112 [P] Add CHANGELOG.md entry for dashboard feature
- [x] T113 [P] Validate quickstart.md examples work correctly
- [x] T114 [P] Add accessibility testing (WCAG 2.1 AA compliance) in dashboard/tests/e2e/test-accessibility.spec.js
- [x] T115 [P] Add cross-browser testing (Chrome, Firefox, Safari, Edge) in dashboard/tests/e2e/test-cross-browser.spec.js
- [x] T116 [P] (Optional) Integrate visual regression testing with Playwright (using @playwright/test screenshot comparison or Percy/Chromatic Playwright integration) for UI component visual validation per FR-026
- [x] T117 [P] (Optional) Integrate accessibility testing with Playwright (using @axe-core/playwright or similar Playwright plugin) for automated WCAG 2.1 AA compliance validation per FR-026
- [x] T118 [P] (Optional) Integrate code quality analysis tool (e.g., SonarJS, CodeQL, or ESLint with Playwright test coverage) for code quality and security analysis per FR-026

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies - can start immediately
- **Foundational (Phase 2)**: Depends on Setup completion - BLOCKS all user stories
- **User Stories (Phase 3+)**: All depend on Foundational phase completion
  - User Story 1 (P1) and User Story 2 (P1) can proceed in parallel after Foundational
  - User Story 3 (P2) enhances performance but can be done in parallel with US1/US2
  - User Story 4 (P2) depends on US1 and US2 for UI integration
- **Rust Service Integration (Phase 7)**: Can be done in parallel with dashboard development
- **Polish (Final Phase)**: Depends on all desired user stories being complete

### User Story Dependencies

- **User Story 1 (P1)**: Can start after Foundational (Phase 2) - No dependencies on other stories
- **User Story 2 (P1)**: Can start after Foundational (Phase 2) - Can proceed in parallel with US1
- **User Story 3 (P2)**: Can start after Foundational (Phase 2) - Enhances performance for US1 and US2
- **User Story 4 (P2)**: Can start after US1 and US2 - Integrates UI for both traces and metrics

### Within Each User Story

- Tests (if included) MUST be written and FAIL before implementation
- Data models before components
- Components before integration
- Core implementation before UI integration
- Story complete before moving to next priority

### Parallel Opportunities

- All Setup tasks marked [P] can run in parallel
- All Foundational tasks marked [P] can run in parallel (within Phase 2)
- Once Foundational phase completes, US1 and US2 can start in parallel
- All tests for a user story marked [P] can run in parallel
- Components within a story marked [P] can run in parallel
- Different user stories can be worked on in parallel by different team members
- Rust Service Integration can be developed in parallel with dashboard development

---

## Parallel Example: User Story 1

```bash
# Launch all tests for User Story 1 together:
Task: "Unit test for TraceList component in dashboard/tests/unit/traces/test-trace-list.js"
Task: "Unit test for TraceDetail component in dashboard/tests/unit/traces/test-trace-detail.js"
Task: "Unit test for TraceFilter component in dashboard/tests/unit/traces/test-trace-filter.js"

# Launch all components for User Story 1 together:
Task: "Create TraceEntry data model in dashboard/src/traces/trace-entry.js"
Task: "Create TraceList component in dashboard/src/traces/trace-list.js"
Task: "Create TraceDetail component in dashboard/src/traces/trace-detail.js"
Task: "Create TraceFilter component in dashboard/src/traces/trace-filter.js"
```

---

## Implementation Strategy

### MVP First (User Stories 1 & 2 Only)

1. Complete Phase 1: Setup
2. Complete Phase 2: Foundational (CRITICAL - blocks all stories)
3. Complete Phase 3: User Story 1 (Live Trace Tail Viewer)
4. Complete Phase 4: User Story 2 (Realtime Metrics Graphing)
5. **STOP and VALIDATE**: Test User Stories 1 and 2 independently
6. Deploy/demo if ready

### Incremental Delivery

1. Complete Setup + Foundational → Foundation ready
2. Add User Story 1 → Test independently → Deploy/Demo (Trace MVP!)
3. Add User Story 2 → Test independently → Deploy/Demo (Metrics MVP!)
4. Add User Story 3 → Test independently → Deploy/Demo (Performance!)
5. Add User Story 4 → Test independently → Deploy/Demo (Complete UI!)
6. Add Rust Service Integration → Test independently → Deploy/Demo
7. Add Polish → Test independently → Deploy/Demo
8. Each story adds value without breaking previous stories

### Parallel Team Strategy

With multiple developers:

1. Team completes Setup + Foundational together
2. Once Foundational is done:
   - Developer A: User Story 1 - Trace components
   - Developer B: User Story 2 - Metric components
   - Developer C: User Story 3 - Performance optimizations
   - Developer D: Rust Service Integration
3. Next iteration:
   - Developer A: User Story 4 - UI integration
   - Developer B: Polish tasks
   - Developer C: Testing and documentation
   - Developer D: Cross-browser testing
4. Stories complete and integrate independently

---

## Notes

- [P] tasks = different files, no dependencies
- [Story] label maps task to specific user story for traceability
- Each user story should be independently completable and testable
- Verify tests fail before implementing
- Commit after each task or logical group
- Stop at any checkpoint to validate story independently
- Avoid: vague tasks, same file conflicts, cross-story dependencies that break independence
- All tasks must maintain 80% code coverage per file (constitution requirement)
- All code must pass linting and formatting checks before merge
- Dashboard runs entirely in browser (no server-side processing)
- Rust service integration is optional (dashboard can be served statically)

