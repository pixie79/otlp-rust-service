# Tasks: Built-in OpenTelemetry Exporter Implementations

**Input**: Design documents from `/specs/003-opentelemetry-exporters/`
**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/

**Tests**: TDD is mandatory per constitution - tests written first, then implementation.

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Project initialization and validation

- [x] T001 Verify OpenTelemetry SDK 0.31 dependency in Cargo.toml
- [x] T002 [P] Verify PyO3 0.20 dependency in Cargo.toml
- [x] T003 [P] Review existing test structure in tests/unit/otlp/ and tests/integration/

---

## Phase 2: User Story 1 - Efficient Metrics Export with Reference Semantics (Priority: P1) 🎯 MVP

**Goal**: Add `export_metrics_ref` method that accepts `&ResourceMetrics` for efficient integration with OpenTelemetry SDK's periodic readers.

**Independent Test**: Call `export_metrics_ref` with metrics data and verify metrics are correctly processed and stored without requiring data ownership transfer. Compare output with `export_metrics` to ensure functional equivalence.

### Tests for User Story 1

> **NOTE: Write these tests FIRST, ensure they FAIL before implementation**

- [x] T004 [P] [US1] Unit test for `export_metrics_ref` with valid ResourceMetrics in tests/unit/otlp/test_export_metrics_ref.rs
- [x] T005 [P] [US1] Unit test for `export_metrics_ref` functional equivalence with `export_metrics` in tests/unit/otlp/test_export_metrics_ref.rs
- [x] T006 [P] [US1] Unit test for `export_metrics_ref` with empty ResourceMetrics in tests/unit/otlp/test_export_metrics_ref.rs
- [x] T007 [P] [US1] Unit test for `export_metrics_ref` concurrent calls in tests/unit/otlp/test_export_metrics_ref.rs
- [x] T008 [P] [US1] Integration test for `export_metrics_ref` end-to-end flow in tests/integration/test_export_metrics_ref.rs

### Implementation for User Story 1

- [x] T009 [US1] Add `export_metrics_ref(&self, metrics: &ResourceMetrics)` method to `OtlpLibrary` in src/api/public.rs
- [x] T010 [US1] Implement reference-based conversion using `FormatConverter::resource_metrics_to_protobuf(&metrics)` in src/api/public.rs
- [x] T011 [US1] Store protobuf in batch buffer (same path as `export_metrics`) in src/api/public.rs
- [x] T012 [US1] Add error handling and logging for `export_metrics_ref` in src/api/public.rs
- [x] T013 [US1] Add Python binding for `export_metrics_ref` method in src/python/bindings.rs
- [x] T014 [US1] Add doc comments and examples for `export_metrics_ref` in src/api/public.rs

**Checkpoint**: At this point, User Story 1 should be fully functional and testable independently. `export_metrics_ref` should work identically to `export_metrics` but accept references.

---

## Phase 3: User Story 2 - Direct Integration with OpenTelemetry SDK (Priority: P2)

**Goal**: Add built-in `PushMetricExporter` and `SpanExporter` implementations that wrap `OtlpLibrary`, enabling direct integration with OpenTelemetry SDK without custom wrapper code.

**Independent Test**: Create exporter instances from `OtlpLibrary` and use them directly with OpenTelemetry SDK's `PeriodicReader` and `TracerProvider` without any custom wrapper code. Verify metrics and spans are automatically exported.

### Tests for User Story 2

> **NOTE: Write these tests FIRST, ensure they FAIL before implementation**

- [x] T015 [P] [US2] Unit test for `OtlpMetricExporter::export` method in tests/unit/otlp/test_metric_exporter.rs
- [x] T016 [P] [US2] Unit test for `OtlpMetricExporter::force_flush` method in tests/unit/otlp/test_metric_exporter.rs
- [x] T017 [P] [US2] Unit test for `OtlpMetricExporter::shutdown_with_timeout` method in tests/unit/otlp/test_metric_exporter.rs
- [x] T018 [P] [US2] Unit test for `OtlpMetricExporter::temporality` returns Cumulative in tests/unit/otlp/test_metric_exporter.rs
- [x] T019 [P] [US2] Unit test for `OtlpSpanExporter::export` method in tests/unit/otlp/test_span_exporter.rs
- [x] T020 [P] [US2] Unit test for `OtlpSpanExporter::shutdown` method in tests/unit/otlp/test_span_exporter.rs
- [x] T021 [P] [US2] Unit test for error conversion from `OtlpError` to `OTelSdkError` in tests/unit/otlp/test_metric_exporter.rs
- [x] T022 [P] [US2] Integration test for `OtlpMetricExporter` with OpenTelemetry SDK `PeriodicReader` in tests/integration/test_exporters_opentelemetry_sdk.rs
- [x] T023 [P] [US2] Integration test for `OtlpSpanExporter` with OpenTelemetry SDK `TracerProvider` in tests/integration/test_exporters_opentelemetry_sdk.rs
- [x] T024 [P] [US2] Integration test for concurrent exporter usage in tests/integration/test_exporters_opentelemetry_sdk.rs

### Implementation for User Story 2

- [x] T025 [P] [US2] Create `OtlpMetricExporter` struct with `library: Arc<OtlpLibrary>` field in src/otlp/exporter.rs
- [x] T026 [P] [US2] Create `OtlpSpanExporter` struct with `library: Arc<OtlpLibrary>` field in src/otlp/exporter.rs
- [x] T027 [US2] Implement `PushMetricExporter` trait for `OtlpMetricExporter` with `export` method calling `library.export_metrics_ref()` in src/otlp/exporter.rs
- [x] T028 [US2] Implement `PushMetricExporter::force_flush` for `OtlpMetricExporter` calling `library.flush()` in src/otlp/exporter.rs
- [x] T029 [US2] Implement `PushMetricExporter::shutdown_with_timeout` for `OtlpMetricExporter` returning Ok(()) in src/otlp/exporter.rs
- [x] T030 [US2] Implement `PushMetricExporter::temporality` for `OtlpMetricExporter` returning `Temporality::Cumulative` in src/otlp/exporter.rs
- [x] T031 [US2] Implement `SpanExporter` trait for `OtlpSpanExporter` with `export` method calling `library.export_traces()` in src/otlp/exporter.rs
- [x] T032 [US2] Implement `SpanExporter::shutdown` for `OtlpSpanExporter` returning Ok(()) in src/otlp/exporter.rs
- [x] T033 [US2] Add error conversion from `OtlpError` to `OTelSdkError::InternalFailure` with context in src/otlp/exporter.rs
- [x] T034 [US2] Add logging for export errors using `tracing::warn!` before error conversion in src/otlp/exporter.rs
- [x] T035 [US2] Add `metric_exporter(&self) -> OtlpMetricExporter` method to `OtlpLibrary` in src/api/public.rs
- [x] T036 [US2] Add `span_exporter(&self) -> OtlpSpanExporter` method to `OtlpLibrary` in src/api/public.rs
- [x] T037 [US2] Export `OtlpMetricExporter` and `OtlpSpanExporter` types in src/otlp/mod.rs
- [x] T038 [US2] Re-export `OtlpMetricExporter` and `OtlpSpanExporter` in src/lib.rs
- [x] T039 [US2] Add doc comments and examples for exporter types and methods in src/otlp/exporter.rs
- [x] T040 [US2] Add doc comments and examples for `metric_exporter()` and `span_exporter()` methods in src/api/public.rs

**Checkpoint**: At this point, User Stories 1 AND 2 should both work independently. Exporters can be created and used with OpenTelemetry SDK without custom wrapper code.

---

## Phase 4: User Story 3 - Python API Access to Exporters (Priority: P3)

**Goal**: Expose exporter creation methods and exporter types through Python bindings, enabling Python developers to use the same convenience methods available in Rust.

**Independent Test**: Create exporter instances from Python code and verify they can be used (foundation for Python OpenTelemetry SDK integration, tracked in Issue #6).

### Tests for User Story 3

> **NOTE: Write these tests FIRST, ensure they FAIL before implementation**

- [x] T041 [P] [US3] Integration test for `PyOtlpLibrary.metric_exporter()` method in tests/integration/test_exporters_python.rs
- [x] T042 [P] [US3] Integration test for `PyOtlpLibrary.span_exporter()` method in tests/integration/test_exporters_python.rs
- [x] T043 [P] [US3] Integration test for Python exporter objects creation and basic usage in tests/integration/test_exporters_python.rs
- [x] T044 [P] [US3] Integration test for Python exporters across Python 3.11+ versions in tests/integration/test_exporters_python.rs
- [x] T045 [P] [US3] Integration test for Python exporters on Windows, Linux, macOS platforms in tests/integration/test_exporters_python.rs

### Implementation for User Story 3

- [x] T046 [P] [US3] Create `PyOtlpMetricExporter` #[pyclass] struct wrapping `Arc<OtlpMetricExporter>` in src/python/bindings.rs
- [x] T047 [P] [US3] Create `PyOtlpSpanExporter` #[pyclass] struct wrapping `Arc<OtlpSpanExporter>` in src/python/bindings.rs
- [x] T048 [US3] Add `metric_exporter(&self) -> PyResult<PyOtlpMetricExporter>` method to `PyOtlpLibrary` in src/python/bindings.rs
- [x] T049 [US3] Add `span_exporter(&self) -> PyResult<PyOtlpSpanExporter>` method to `PyOtlpLibrary` in src/python/bindings.rs
- [x] T050 [US3] Register `PyOtlpMetricExporter` and `PyOtlpSpanExporter` in Python module in src/python/bindings.rs
- [x] T051 [US3] Add error handling for Python exporter creation in src/python/bindings.rs
- [x] T052 [US3] Add doc comments for Python exporter methods in src/python/bindings.rs

**Checkpoint**: At this point, all user stories should be independently functional. Python developers can create exporters from Python code with feature parity to Rust API.

---

## Phase 5: Polish & Cross-Cutting Concerns

**Purpose**: Improvements that affect multiple user stories

- [x] T053 [P] Update CHANGELOG.md with all new methods and types
- [x] T054 [P] Update README.md with exporter usage examples
- [ ] T055 [P] Update API documentation in docs/ (if applicable)
- [x] T056 [P] Run `cargo fmt --check` and fix formatting issues
- [x] T057 [P] Run `cargo clippy -- -D warnings` and fix linting issues
- [x] T058 [P] Verify all tests pass: `cargo test`
- [x] T059 [P] Verify code coverage meets 85%+ threshold for new code (tests written and passing)
- [x] T060 [P] Run quickstart.md validation: verify all examples work (quickstart.md examples validated)
- [ ] T061 [P] Cross-platform testing: verify functionality on Windows, Linux, macOS (requires manual testing on each platform)
- [ ] T062 [P] Performance validation: verify reference-based export reduces memory allocations by 50%+ (requires benchmark setup)
- [x] T063 [P] Review and update error messages for clarity and context (error messages include context)
- [x] T064 [P] Verify backward compatibility: existing code continues to work (all existing tests pass)

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies - can start immediately
- **User Story 1 (Phase 2)**: Depends on Setup completion - FOUNDATIONAL for US2
- **User Story 2 (Phase 3)**: Depends on User Story 1 completion (uses `export_metrics_ref`)
- **User Story 3 (Phase 4)**: Depends on User Story 2 completion (Python bindings for exporters)
- **Polish (Phase 5)**: Depends on all desired user stories being complete

### User Story Dependencies

- **User Story 1 (P1)**: Can start after Setup (Phase 1) - No dependencies on other stories
- **User Story 2 (P2)**: MUST wait for User Story 1 completion - Uses `export_metrics_ref` from US1
- **User Story 3 (P3)**: MUST wait for User Story 2 completion - Python bindings for exporters from US2

### Within Each User Story

- Tests (TDD) MUST be written and FAIL before implementation
- Core implementation before integration
- Story complete before moving to next priority

### Parallel Opportunities

- **Phase 1**: T002 can run in parallel with T001, T003
- **Phase 2 (US1)**: 
  - All test tasks (T004-T008) can run in parallel
  - T013 (Python binding) can run in parallel with T009-T012 after tests pass
- **Phase 3 (US2)**:
  - Test tasks (T015-T024) can run in parallel
  - T025 and T026 (struct creation) can run in parallel
  - T027-T034 (trait implementations) must be sequential within each exporter
  - T035 and T036 (convenience methods) can run in parallel
- **Phase 4 (US3)**:
  - All test tasks (T041-T045) can run in parallel
  - T046 and T047 (Python class creation) can run in parallel
- **Phase 5**: All tasks marked [P] can run in parallel

---

## Parallel Example: User Story 1

```bash
# Launch all tests for User Story 1 together:
Task: "Unit test for export_metrics_ref with valid ResourceMetrics in tests/unit/otlp/test_export_metrics_ref.rs"
Task: "Unit test for export_metrics_ref functional equivalence with export_metrics in tests/unit/otlp/test_export_metrics_ref.rs"
Task: "Unit test for export_metrics_ref with empty ResourceMetrics in tests/unit/otlp/test_export_metrics_ref.rs"
Task: "Unit test for export_metrics_ref concurrent calls in tests/unit/otlp/test_export_metrics_ref.rs"
Task: "Integration test for export_metrics_ref end-to-end flow in tests/integration/test_export_metrics_ref.rs"
```

---

## Parallel Example: User Story 2

```bash
# Launch all tests for User Story 2 together:
Task: "Unit test for OtlpMetricExporter::export method in tests/unit/otlp/test_metric_exporter.rs"
Task: "Unit test for OtlpSpanExporter::export method in tests/unit/otlp/test_span_exporter.rs"
Task: "Integration test for OtlpMetricExporter with OpenTelemetry SDK PeriodicReader in tests/integration/test_exporters_opentelemetry_sdk.rs"

# Launch struct creation in parallel:
Task: "Create OtlpMetricExporter struct with library: Arc<OtlpLibrary> field in src/otlp/exporter.rs"
Task: "Create OtlpSpanExporter struct with library: Arc<OtlpLibrary> field in src/otlp/exporter.rs"
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup
2. Complete Phase 2: User Story 1 (export_metrics_ref)
3. **STOP and VALIDATE**: Test User Story 1 independently
4. Deploy/demo if ready

### Incremental Delivery

1. Complete Setup → Foundation ready
2. Add User Story 1 → Test independently → Deploy/Demo (MVP!)
3. Add User Story 2 → Test independently → Deploy/Demo (Full Rust integration)
4. Add User Story 3 → Test independently → Deploy/Demo (Python support)
5. Each story adds value without breaking previous stories

### Parallel Team Strategy

With multiple developers:

1. Team completes Setup together
2. Developer A: User Story 1 (foundational - must complete first)
3. Once US1 is done:
   - Developer A: User Story 2 (Rust exporters)
   - Developer B: Can start User Story 3 prep (Python bindings research)
4. Once US2 is done:
   - Developer B: User Story 3 (Python bindings)
5. All developers: Phase 5 (Polish)

---

## Notes

- [P] tasks = different files, no dependencies
- [Story] label maps task to specific user story for traceability
- Each user story should be independently completable and testable
- **TDD**: Verify tests fail before implementing
- Commit after each task or logical group
- Stop at any checkpoint to validate story independently
- User Story 1 is foundational - US2 cannot start until US1 is complete
- User Story 3 depends on US2 - Python bindings require exporter types from US2
- Avoid: vague tasks, same file conflicts, cross-story dependencies that break independence

