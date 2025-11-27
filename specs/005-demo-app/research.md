# Research: Demo Rust Application

**Feature**: 005-demo-app  
**Date**: 2025-11-26  
**Status**: Complete

## Research Tasks

### Task 1: ResourceMetrics Construction with Private Fields

**Question**: How to create realistic ResourceMetrics for demo given that opentelemetry-sdk 0.31 has private fields?

**Findings**:
- `ResourceMetrics` fields are private in `opentelemetry-sdk` 0.31
- Current codebase uses `ResourceMetrics::default()` as placeholder
- Full metrics construction requires using `opentelemetry-proto` conversion utilities
- For demo purposes, we can use `ResourceMetrics::default()` and rely on the fact that the library preserves data from protobuf format
- Alternative: Use the public API `export_metrics_ref()` which accepts references

**Decision**: Use `ResourceMetrics::default()` for demo simplicity. The demo will focus on demonstrating the SDK usage patterns rather than complex metric construction. The dashboard will display data from spans primarily, with metrics serving as a demonstration of the export API.

**Rationale**: 
- Demo's primary purpose is to show SDK usage patterns, not complex metric construction
- Existing examples (`embedded.rs`) already use `ResourceMetrics::default()`
- Full metric construction would require significant research into opentelemetry-proto internals
- Dashboard visualization works with data from both metrics and spans

**Alternatives Considered**:
- Using opentelemetry-proto conversion: Too complex for a demo, requires deep understanding of protobuf structures
- Creating custom metric types: Not aligned with SDK patterns, would confuse developers
- Skipping metrics entirely: Rejected because FR-002 requires metrics demonstration

---

### Task 2: Best Practices for Rust Example/Demo Applications

**Question**: What are best practices for creating example applications in Rust that serve as reference implementations?

**Findings**:
- Examples should be self-contained and runnable with minimal setup
- Clear, extensive comments explaining each step
- Follow project's existing patterns (see `embedded.rs`, `standalone.rs`)
- Use `#[tokio::main]` for async examples
- Include error handling examples
- Demonstrate both happy path and error scenarios
- Use `println!` or `tracing` for user feedback
- Include graceful shutdown patterns

**Decision**: Follow existing example patterns from `embedded.rs` and `standalone.rs`. Include:
- Extensive doc comments and inline comments
- Clear section organization (initialization, metric creation, span creation, export, shutdown)
- Error handling with `?` operator and proper error messages
- User feedback via `println!` statements
- Graceful shutdown with `flush()` and `shutdown()`

**Rationale**: Consistency with existing examples helps developers understand patterns. Following established conventions reduces cognitive load.

**Alternatives Considered**:
- Minimal comments: Rejected - spec requires 80% comment coverage (SC-004)
- Different structure: Rejected - consistency with existing examples is more valuable

---

### Task 3: Creating Parent-Child Span Relationships

**Question**: How to create realistic span relationships (parent-child) for trace visualization?

**Findings**:
- Spans use `parent_span_id` field in `SpanData` to establish relationships
- `SpanId::INVALID` indicates root span (no parent)
- To create child spans, use the parent span's `span_id` as `parent_span_id`
- Same `trace_id` must be used for all spans in a trace
- Different `span_id` values for each span in the trace

**Decision**: Create a trace with multiple spans showing parent-child relationships:
- Root span (server request)
- Child spans (internal operations, database calls, external API calls)
- Use same `trace_id` for all spans in a trace
- Use parent's `span_id` as `parent_span_id` for child spans
- Demonstrate different span kinds (Server, Client, Internal)

**Rationale**: This demonstrates realistic trace patterns that developers will use in production. Shows how to structure distributed tracing.

**Alternatives Considered**:
- Flat spans (no relationships): Rejected - doesn't demonstrate trace relationships (FR-019)
- Complex multi-level hierarchy: Rejected - too complex for a demo, simple parent-child is sufficient

---

### Task 4: Continuous Data Generation Pattern

**Question**: How to implement continuous data generation for dashboard testing?

**Findings**:
- Use `tokio::time::interval()` for periodic generation
- Use `tokio::signal::ctrl_c()` for graceful shutdown
- Generate data in a loop with sleep intervals
- Use `tokio::select!` to handle both data generation and shutdown signal
- Flush data before shutdown

**Decision**: Use `tokio::time::interval()` with configurable interval (e.g., 2-5 seconds). Generate metrics and spans in each iteration. Use `tokio::select!` to handle both generation loop and Ctrl+C signal. Call `flush()` before shutdown.

**Rationale**: Standard async Rust pattern for periodic tasks. Allows demo to run continuously for dashboard testing while supporting graceful shutdown.

**Alternatives Considered**:
- Single batch generation: Rejected - doesn't meet FR-018 (time-series patterns) and User Story 3
- Thread-based generation: Rejected - async pattern is more appropriate for this codebase

---

### Task 5: Dashboard Configuration and Startup

**Question**: How to properly configure and start dashboard in demo application?

**Findings**:
- Dashboard is enabled via `ConfigBuilder::dashboard_enabled(true)`
- Dashboard requires static files in `./dashboard/dist` directory
- Dashboard server starts automatically when `OtlpLibrary::new()` is called with dashboard enabled
- Dashboard runs on port 8080 by default
- Dashboard server is started in background task, no manual server setup needed

**Decision**: Use `ConfigBuilder` to enable dashboard with `dashboard_enabled(true)`. Use default port (8080) and static directory (`./dashboard/dist`). The library handles dashboard server startup automatically. Demo should print dashboard URL for user convenience.

**Rationale**: Simplest approach that demonstrates dashboard configuration. Library handles complexity of server startup.

**Alternatives Considered**:
- Manual dashboard server setup: Rejected - library already handles this, would add unnecessary complexity
- Custom port/directory: Rejected - defaults are sufficient for demo, can be mentioned in comments

---

## Summary

All research tasks completed. Key decisions:
1. Use `ResourceMetrics::default()` for metrics (simplicity, aligns with existing examples)
2. Follow existing example patterns (`embedded.rs`, `standalone.rs`)
3. Create parent-child span relationships using `parent_span_id`
4. Use `tokio::time::interval()` for continuous generation
5. Use `ConfigBuilder::dashboard_enabled(true)` for dashboard configuration

No blocking issues identified. Implementation can proceed.

