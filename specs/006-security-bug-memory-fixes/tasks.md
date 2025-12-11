# Tasks: Fix Security, Bug, and Memory Issues

**Input**: Design documents from `/specs/006-security-bug-memory-fixes/`
**Prerequisites**: plan.md ✅, spec.md ✅, research.md ✅, data-model.md ✅, contracts/ ✅

**Tests**: Tests are REQUIRED per constitution - TDD approach with tests written first

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

## Phase 1: Setup (Dependencies & Infrastructure)

**Purpose**: Add required dependencies and prepare project structure

- [x] T001 Add `secrecy = "0.8"` dependency to `Cargo.toml` for secure credential storage
- [x] T002 Add `url = "2.5"` dependency to `Cargo.toml` for comprehensive URL validation
- [x] T003 [P] Update `Cargo.toml` to ensure `prost` dependency is available (already present, verify version)

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core infrastructure that MUST be complete before user story implementation

**⚠️ CRITICAL**: These are minimal - most fixes are independent. However, dependency additions must be complete first.

- [x] T004 Verify all dependencies compile successfully: `cargo check --all-features`
- [x] T005 [P] Review existing error types in `src/error.rs` to ensure `BufferFull` error exists (may need to add if missing)

**Checkpoint**: Dependencies ready - user story implementation can now begin

---

## Phase 3: User Story 1 - Secure Credential Handling (Priority: P1) 🎯 MVP

**Goal**: Implement secure credential storage using `SecretString` to prevent memory exposure in logs, errors, and memory dumps

**Independent Test**: Verify credentials stored in memory are never exposed in logs, error messages, or memory dumps. System uses secure string types that zero memory on drop.

### Tests for User Story 1

> **NOTE: Write these tests FIRST, ensure they FAIL before implementation**

- [ ] T006 [P] [US1] Unit test for credential sanitization in logs in `tests/unit/config/test_secure_credentials.rs`
- [ ] T007 [P] [US1] Unit test for SecretString memory zeroing in `tests/unit/config/test_secure_credentials.rs`
- [ ] T008 [P] [US1] Unit test for credential not appearing in Debug output in `tests/unit/config/test_secure_credentials.rs`
- [ ] T009 [P] [US1] Integration test for credential handling in `tests/integration/test_forwarding_auth.rs` (update existing)

### Implementation for User Story 1

- [x] T010 [US1] Update `AuthConfig` struct in `src/config/types.rs` to use `SecretString` for credentials field
- [x] T011 [US1] Update `AuthConfig::validate()` in `src/config/types.rs` to work with `SecretString` credentials
- [x] T012 [US1] Update credential access in `src/otlp/forwarder.rs::add_auth_headers()` to use `SecretString`
- [x] T013 [US1] Add credential sanitization in all logging points (search for credential usage, sanitize in logs)
- [x] T014 [US1] Update `ConfigLoader` in `src/config/loader.rs` to convert `String` to `SecretString` when loading from YAML/env
- [x] T015 [US1] Update all tests that create `AuthConfig` to use `SecretString::new()`

**Checkpoint**: User Story 1 complete - credentials are securely stored and never exposed

---

## Phase 4: User Story 2 - Prevent Path Traversal Attacks (Priority: P1) 🎯 MVP

**Goal**: Implement comprehensive path validation to prevent directory traversal, symlink, absolute path, and UNC path attacks

**Independent Test**: Attempt various path traversal attacks (symlinks, absolute paths, UNC paths, double slashes) and verify all are rejected with 403 Forbidden responses.

### Tests for User Story 2

- [ ] T016 [P] [US2] Integration test for `../` path traversal rejection in `tests/integration/test_dashboard_security.rs`
- [ ] T017 [P] [US2] Integration test for absolute path rejection in `tests/integration/test_dashboard_security.rs`
- [ ] T018 [P] [US2] Integration test for symlink traversal rejection in `tests/integration/test_dashboard_security.rs`
- [ ] T019 [P] [US2] Integration test for Windows UNC path rejection in `tests/integration/test_dashboard_security.rs`
- [ ] T020 [P] [US2] Integration test for double slash normalization in `tests/integration/test_dashboard_security.rs`
- [ ] T021 [P] [US2] Unit test for path validation logic in `tests/unit/otlp/test_path_validation.rs`

### Implementation for User Story 2

- [x] T022 [US2] Enhance path validation in `src/dashboard/server.rs::handle_request()` to reject absolute paths
- [x] T023 [US2] Enhance path validation in `src/dashboard/server.rs::handle_request()` to check for UNC paths (Windows)
- [x] T024 [US2] Add path normalization in `src/dashboard/server.rs::handle_request()` (remove `//`, `.`, etc.)
- [x] T025 [US2] Add symlink handling using `canonicalize()` with proper error handling in `src/dashboard/server.rs`
- [x] T026 [US2] Ensure all path validation failures return 403 Forbidden with appropriate error logging (without exposing paths)

**Checkpoint**: User Story 2 complete - path traversal attacks are prevented

---

## Phase 5: User Story 3 - Fix Critical Syntax and Logic Errors (Priority: P1) 🎯 MVP

**Goal**: Fix syntax error (missing brace) and auth validation logic mismatch to enable compilation and correct runtime behavior

**Independent Test**: Verify code compiles successfully and authentication validation logic matches actual credential key names used in the code.

### Tests for User Story 3

- [ ] T027 [P] [US3] Unit test for auth validation logic matching usage in `tests/unit/otlp/test_auth_config.rs` (update existing)
- [ ] T028 [P] [US3] Integration test for basic auth header generation in `tests/integration/test_forwarding_auth.rs`

### Implementation for User Story 3

- [x] T029 [US3] Fix missing opening brace in `src/otlp/forwarder.rs::add_auth_headers()` for `"basic"` match arm (line ~424)
- [x] T030 [US3] Fix auth validation logic in `src/config/types.rs::AuthConfig::validate()` to check for `"key"` for `api_key` type (not `"api_key"` or `"token"`)
- [ ] T031 [US3] Verify code compiles: `cargo build --all-features`
- [x] T032 [US3] Update tests to reflect corrected validation logic

**Checkpoint**: User Story 3 complete - code compiles and validation logic is correct

---

## Phase 6: User Story 4 - Prevent Unbounded Memory Growth (Priority: P1) 🎯 MVP

**Goal**: Add configurable buffer size limits to prevent unbounded memory growth when writes are delayed or fail

**Independent Test**: Configure buffer limits and verify system rejects new data when limits are reached, rather than consuming unlimited memory.

### Tests for User Story 4

- [ ] T033 [P] [US4] Unit test for buffer limit enforcement in `tests/unit/otlp/test_batch_buffer.rs` (update existing)
- [ ] T034 [P] [US4] Unit test for BufferFull error when limit reached in `tests/unit/otlp/test_batch_buffer.rs`
- [ ] T035 [P] [US4] Integration test for buffer backpressure under load in `tests/integration/test_buffer_limits.rs`
- [ ] T036 [P] [US4] Unit test for buffer limit configuration validation in `tests/unit/config/test_buffer_limits.rs`

### Implementation for User Story 4

- [x] T037 [US4] Add `max_trace_buffer_size: usize` field to `Config` struct in `src/config/types.rs` (default: 10000)
- [x] T038 [US4] Add `max_metric_buffer_size: usize` field to `Config` struct in `src/config/types.rs` (default: 10000)
- [x] T039 [US4] Add validation for buffer size limits in `Config::validate()` in `src/config/types.rs` (> 0, <= 1,000,000)
- [x] T040 [US4] Add `max_trace_size` and `max_metric_size` fields to `BatchBuffer` struct in `src/otlp/batch_writer.rs`
- [x] T041 [US4] Update `BatchBuffer::new()` in `src/otlp/batch_writer.rs` to accept and store buffer limits from config
- [x] T042 [US4] Add size check in `BatchBuffer::add_trace()` in `src/otlp/batch_writer.rs` to return `BufferFull` when limit reached
- [x] T043 [US4] Add size check in `BatchBuffer::add_metrics_protobuf()` in `src/otlp/batch_writer.rs` to return `BufferFull` when limit reached
- [x] T044 [US4] Update `OtlpLibrary::new()` in `src/api/public.rs` to pass buffer limits to `BatchBuffer::new()`
- [x] T045 [US4] Add buffer size limit support to `ConfigBuilder` in `src/config/types.rs`
- [x] T046 [US4] Add environment variable support for buffer limits in `ConfigLoader` in `src/config/loader.rs` (OTLP_MAX_TRACE_BUFFER_SIZE, OTLP_MAX_METRIC_BUFFER_SIZE)

**Checkpoint**: User Story 4 complete - buffer limits prevent unbounded memory growth

---

## Phase 7: User Story 5 - Add Security Headers to HTTP Responses (Priority: P2)

**Goal**: Add security headers (CSP, X-Frame-Options, X-Content-Type-Options, X-XSS-Protection) to all HTTP responses

**Independent Test**: Inspect HTTP response headers and verify all required security headers are present with appropriate values.

### Tests for User Story 5

- [ ] T047 [P] [US5] Integration test for security headers presence in `tests/integration/test_dashboard_security.rs`
- [ ] T048 [P] [US5] Integration test for X-Frame-Options header value in `tests/integration/test_dashboard_security.rs`
- [ ] T049 [P] [US5] Integration test for Content-Security-Policy header in `tests/integration/test_dashboard_security.rs`
- [ ] T050 [P] [US5] Integration test for X-Content-Type-Options header in `tests/integration/test_dashboard_security.rs`

### Implementation for User Story 5

- [x] T051 [US5] Add `x_frame_options: Option<String>` field to `DashboardConfig` struct in `src/config/types.rs` (default: None → "DENY")
- [x] T052 [US5] Add validation for `x_frame_options` in `DashboardConfig::validate()` in `src/config/types.rs` (must be "DENY" or "SAMEORIGIN" if Some)
- [x] T053 [US5] Update `DashboardServer::send_response()` in `src/dashboard/server.rs` to add X-Content-Type-Options: nosniff header
- [x] T054 [US5] Update `DashboardServer::send_response()` in `src/dashboard/server.rs` to add X-Frame-Options header (use config value or default "DENY")
- [x] T055 [US5] Update `DashboardServer::send_response()` in `src/dashboard/server.rs` to add Content-Security-Policy: default-src 'self' header
- [x] T056 [US5] Update `DashboardServer::send_response()` in `src/dashboard/server.rs` to add X-XSS-Protection: 1; mode=block header
- [x] T057 [US5] Ensure security headers are added to all response types (HTML, JSON, Arrow files) in `src/dashboard/server.rs`

**Checkpoint**: User Story 5 complete - all HTTP responses include security headers

---

## Phase 8: User Story 6 - Complete Circuit Breaker Functionality (Priority: P2)

**Goal**: Implement complete circuit breaker half-open state logic to enable automatic recovery from remote service failures

**Independent Test**: Simulate remote service failures and recoveries, verify circuit breaker transitions through all states correctly (Closed → Open → HalfOpen → Closed).

### Tests for User Story 6

- [ ] T058 [P] [US6] Unit test for half-open to closed transition on success in `tests/unit/otlp/test_circuit_breaker.rs`
- [ ] T059 [P] [US6] Unit test for half-open to open transition on failure in `tests/unit/otlp/test_circuit_breaker.rs`
- [ ] T060 [P] [US6] Unit test for open to half-open transition after timeout in `tests/unit/otlp/test_circuit_breaker.rs`
- [ ] T061 [P] [US6] Integration test for circuit breaker state machine in `tests/integration/test_forwarding.rs` (update existing)

### Implementation for User Story 6

- [x] T062 [US6] Add `half_open_test_in_progress: Arc<Mutex<bool>>` field to `CircuitBreaker` struct in `src/otlp/forwarder.rs` to prevent concurrent tests
- [x] T063 [US6] Implement half-open state logic in `CircuitBreaker::call()` in `src/otlp/forwarder.rs` to allow single test request
- [x] T064 [US6] Implement success handling in half-open state: transition to Closed, reset counters in `src/otlp/forwarder.rs`
- [x] T065 [US6] Implement failure handling in half-open state: transition back to Open, update failure time in `src/otlp/forwarder.rs`
- [x] T066 [US6] Add timeout check to prevent indefinite half-open state in `src/otlp/forwarder.rs`

**Checkpoint**: User Story 6 complete - circuit breaker properly handles all state transitions

---

## Phase 9: User Story 7 - Comprehensive Input Validation (Priority: P2)

**Goal**: Implement comprehensive URL and input validation using proper parsing libraries and bounds checking

**Independent Test**: Provide various invalid inputs (malformed URLs, invalid paths, out-of-bounds values) and verify appropriate validation errors are returned.

### Tests for User Story 7

- [ ] T067 [P] [US7] Contract test for URL validation in `tests/contract/test_input_validation.rs`
- [ ] T068 [P] [US7] Contract test for bounds checking in `tests/contract/test_input_validation.rs`
- [ ] T069 [P] [US7] Unit test for URL parsing with IDN domains in `tests/unit/config/test_config_validation.rs` (update existing)
- [ ] T070 [P] [US7] Unit test for URL validation error messages in `tests/unit/config/test_config_validation.rs`

### Implementation for User Story 7

- [x] T071 [US7] Replace simple prefix check with `url::Url::parse()` in `ForwardingConfig::validate()` in `src/config/types.rs`
- [x] T072 [US7] Add scheme validation (must be http or https) in `ForwardingConfig::validate()` in `src/config/types.rs`
- [x] T073 [US7] Add host presence validation in `ForwardingConfig::validate()` in `src/config/types.rs`
- [x] T074 [US7] Improve error messages for URL validation failures in `src/config/types.rs`
- [x] T075 [US7] Add comprehensive bounds checking for all numeric config values in `Config::validate()` in `src/config/types.rs`
- [x] T076 [US7] Add file size limit validation to `Config` if needed (check `src/otlp/exporter.rs` for hardcoded limits)

**Checkpoint**: User Story 7 complete - comprehensive input validation implemented

---

## Phase 10: User Story 8 - Fix Memory Safety Issues in Python Bindings (Priority: P2)

**Goal**: Investigate and fix segfaults in Python bindings to ensure memory safety

**Independent Test**: Run Python test suite and verify no segfaults occur. Use memory sanitizers to detect memory safety violations.

### Tests for User Story 8

- [ ] T077 [P] [US8] Unit test for Python object lifecycle in `tests/unit/python/test_memory_safety.rs`
- [ ] T078 [P] [US8] Integration test for Python bindings without segfaults in `tests/python/test_integration.py` (update existing)
- [ ] T079 [P] [US8] Memory sanitizer test configuration in CI workflow (update `.github/workflows/ci.yml`)

### Implementation for User Story 8

- [x] T080 [US8] Review all `unsafe` blocks in `src/python/bindings.rs` for memory safety issues
- [x] T081 [US8] Review all `unsafe` blocks in `src/python/adapters.rs` for memory safety issues
- [x] T082 [US8] Verify Python object reference counting is correct (use `PyRef`, `Py` types appropriately) in `src/python/bindings.rs`
- [x] T083 [US8] Check for double-free or use-after-free patterns in Python bindings
- [x] T084 [US8] Add explicit lifetime management if needed in `src/python/bindings.rs` and `src/python/adapters.rs`
- [x] T085 [US8] Consider updating PyO3 version if issues persist (check `Cargo.toml` for current version)
- [ ] T086 [US8] Remove segfault workaround logic from `.github/workflows/ci.yml` after fixes are verified

**Checkpoint**: User Story 8 complete - Python bindings are memory-safe

---

## Phase 11: User Story 9 - Complete Protobuf Encoding Implementation (Priority: P3)

**Goal**: Implement proper Protobuf encoding for HTTP forwarding instead of empty buffers

**Independent Test**: Configure forwarding and verify data is properly encoded and sent to remote endpoints.

### Tests for User Story 9

- [ ] T087 [P] [US9] Integration test for Protobuf trace encoding in `tests/integration/test_forwarding.rs` (update existing)
- [ ] T088 [P] [US9] Integration test for Protobuf metric encoding in `tests/integration/test_forwarding.rs` (update existing)
- [ ] T089 [P] [US9] Contract test for Protobuf message structure in `tests/contract/test_otlp_protocol.rs` (update existing)

### Implementation for User Story 9

- [x] T090 [US9] Replace empty buffer placeholder with `prost::Message::encode()` in `send_protobuf_traces()` in `src/otlp/forwarder.rs` (line ~303)
- [x] T091 [US9] Replace empty buffer placeholder with `prost::Message::encode()` in `send_protobuf_metrics()` in `src/otlp/forwarder.rs` (line ~345)
- [x] T092 [US9] Ensure proper error handling for encoding failures in `src/otlp/forwarder.rs`
- [x] T093 [US9] Verify encoded Protobuf data is valid and can be decoded by remote endpoints

**Checkpoint**: User Story 9 complete - Protobuf encoding works correctly

---

## Phase 12: User Story 10 - Security Documentation (Priority: P3)

**Goal**: Create SECURITY.md documentation covering security model, threat model, vulnerability reporting, and best practices

**Independent Test**: Verify SECURITY.md exists and contains all required sections (security model, threat model, reporting process, best practices).

### Tests for User Story 10

- [ ] T094 [P] [US10] Documentation test to verify SECURITY.md exists and is readable
- [ ] T095 [P] [US10] Documentation test to verify all required sections are present in `SECURITY.md`

### Implementation for User Story 10

- [x] T096 [US10] Create `SECURITY.md` in repository root with security model section
- [x] T097 [US10] Add threat model section to `SECURITY.md` describing what the system protects against
- [x] T098 [US10] Add vulnerability reporting process section to `SECURITY.md` (responsible disclosure)
- [x] T099 [US10] Add security best practices section to `SECURITY.md` for users configuring the system
- [x] T100 [US10] Add known security considerations section to `SECURITY.md`
- [x] T101 [US10] Add security update policy section to `SECURITY.md`

**Checkpoint**: User Story 10 complete - security documentation is comprehensive

---

## Phase 13: Polish & Cross-Cutting Concerns

**Purpose**: Final improvements, documentation updates, and validation

- [x] T102 [P] Update `CHANGELOG.md` with all security fixes, bug fixes, and memory management improvements
- [x] T103 [P] Update `README.md` with new configuration options (buffer limits, security headers)
- [x] T104 [P] Update API documentation in `src/config/types.rs` doc comments for breaking changes (credential storage)
- [ ] T105 [P] Run `cargo fmt --all` to ensure code formatting
- [ ] T106 [P] Run `cargo clippy --all-targets --all-features -- -D warnings` to ensure no linting issues
- [x] T107 [P] Run `cargo test --all-features --workspace` to ensure all tests pass (all tests migrated to ConfigBuilder)
- [ ] T108 [P] Verify code coverage remains above 85% per file: `cargo tarpaulin --workspace`
- [ ] T109 [P] Run Python tests: `pytest tests/python/` to verify no segfaults
- [ ] T110 [P] Validate quickstart.md examples work correctly
- [ ] T111 [P] Update version numbers if needed (check `Cargo.toml`, `pyproject.toml`, `CHANGELOG.md` consistency)
- [ ] T112 [P] Run `scripts/validate-versions.sh` to ensure version consistency

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies - can start immediately
- **Foundational (Phase 2)**: Depends on Setup completion - BLOCKS all user stories
- **User Stories P1 (Phases 3-6)**: All depend on Foundational completion
  - Can proceed sequentially in priority order (US1 → US2 → US3 → US4)
  - Or in parallel if team capacity allows (different files, minimal overlap)
- **User Stories P2 (Phases 7-10)**: Depend on Foundational completion
  - Can proceed after P1 stories or in parallel if no dependencies
  - US5 (security headers) can be parallel with US6 (circuit breaker)
  - US7 (input validation) can be parallel with US8 (Python fixes)
- **User Stories P3 (Phases 11-12)**: Depend on Foundational completion
  - Can proceed after P1/P2 or in parallel
- **Polish (Phase 13)**: Depends on all desired user stories being complete

### User Story Dependencies

- **User Story 1 (P1)**: Can start after Foundational - No dependencies on other stories
- **User Story 2 (P1)**: Can start after Foundational - No dependencies on other stories
- **User Story 3 (P1)**: Can start after Foundational - No dependencies on other stories (fixes blocking bugs)
- **User Story 4 (P1)**: Can start after Foundational - No dependencies on other stories
- **User Story 5 (P2)**: Can start after Foundational - No dependencies on other stories
- **User Story 6 (P2)**: Can start after Foundational - No dependencies on other stories
- **User Story 7 (P2)**: Can start after Foundational - No dependencies on other stories
- **User Story 8 (P2)**: Can start after Foundational - No dependencies on other stories
- **User Story 9 (P3)**: Can start after Foundational - No dependencies on other stories
- **User Story 10 (P3)**: Can start after Foundational - No dependencies on other stories

### Within Each User Story

- Tests MUST be written and FAIL before implementation (TDD approach)
- Configuration/model changes before implementation code
- Core implementation before integration
- Story complete before moving to next priority (recommended, but can parallelize)

### Parallel Opportunities

- **Setup phase**: All tasks can run in parallel (T001-T003)
- **Foundational phase**: T004 and T005 can run in parallel
- **User Story 1**: Tests (T006-T009) can run in parallel
- **User Story 2**: Tests (T016-T021) can run in parallel
- **User Story 3**: Tests (T027-T028) can run in parallel
- **User Story 4**: Tests (T033-T036) can run in parallel, config/model tasks (T037-T039) can run in parallel
- **User Story 5**: Tests (T047-T050) can run in parallel
- **User Story 6**: Tests (T058-T061) can run in parallel
- **User Story 7**: Tests (T067-T070) can run in parallel
- **User Story 8**: Tests (T077-T079) can run in parallel
- **User Story 9**: Tests (T087-T089) can run in parallel
- **User Story 10**: Tests (T094-T095) can run in parallel
- **Polish phase**: Most tasks can run in parallel (T102-T112)

**Cross-Story Parallelization**: Once Foundational phase completes, all user stories can be worked on in parallel by different team members (they touch different files and have no dependencies).

---

## Parallel Example: User Story 1

```bash
# Launch all tests for User Story 1 together:
Task T006: "Unit test for credential sanitization in logs in tests/unit/config/test_secure_credentials.rs"
Task T007: "Unit test for SecretString memory zeroing in tests/unit/config/test_secure_credentials.rs"
Task T008: "Unit test for credential not appearing in Debug output in tests/unit/config/test_secure_credentials.rs"
Task T009: "Integration test for credential handling in tests/integration/test_forwarding_auth.rs"

# After tests are written and fail, launch implementation:
Task T010: "Update AuthConfig struct in src/config/types.rs"
Task T011: "Update AuthConfig::validate() in src/config/types.rs"
Task T012: "Update credential access in src/otlp/forwarder.rs"
```

---

## Parallel Example: User Story 4

```bash
# Launch all tests together:
Task T033: "Unit test for buffer limit enforcement in tests/unit/otlp/test_batch_buffer.rs"
Task T034: "Unit test for BufferFull error when limit reached in tests/unit/otlp/test_batch_buffer.rs"
Task T035: "Integration test for buffer backpressure under load in tests/integration/test_buffer_limits.rs"
Task T036: "Unit test for buffer limit configuration validation in tests/unit/config/test_buffer_limits.rs"

# Launch config/model changes in parallel:
Task T037: "Add max_trace_buffer_size field to Config struct in src/config/types.rs"
Task T038: "Add max_metric_buffer_size field to Config struct in src/config/types.rs"
Task T039: "Add validation for buffer size limits in Config::validate() in src/config/types.rs"
```

---

## Implementation Strategy

### MVP First (Critical P1 Stories Only)

1. Complete Phase 1: Setup (dependencies)
2. Complete Phase 2: Foundational (verify dependencies compile)
3. Complete Phase 3: User Story 1 (Secure Credential Handling) ✅ MVP
4. **STOP and VALIDATE**: Test User Story 1 independently
5. Complete Phase 4: User Story 2 (Path Traversal Protection) ✅ MVP
6. **STOP and VALIDATE**: Test User Story 2 independently
7. Complete Phase 5: User Story 3 (Syntax/Logic Fixes) ✅ MVP
8. **STOP and VALIDATE**: Verify code compiles and runs
9. Complete Phase 6: User Story 4 (Memory Limits) ✅ MVP
10. **STOP and VALIDATE**: Test buffer limits independently
11. Deploy/demo MVP with critical security and stability fixes

### Incremental Delivery

1. Complete Setup + Foundational → Foundation ready
2. Add User Story 1 → Test independently → Deploy (Critical Security Fix #1)
3. Add User Story 2 → Test independently → Deploy (Critical Security Fix #2)
4. Add User Story 3 → Test independently → Deploy (Critical Bug Fix)
5. Add User Story 4 → Test independently → Deploy (Critical Memory Fix)
6. Add User Stories 5-8 (P2) → Test independently → Deploy (Important Fixes)
7. Add User Stories 9-10 (P3) → Test independently → Deploy (Enhancements)
8. Each story adds value without breaking previous stories

### Parallel Team Strategy

With multiple developers:

1. Team completes Setup + Foundational together
2. Once Foundational is done:
   - **Developer A**: User Story 1 (Secure Credentials) - `src/config/types.rs`, `src/otlp/forwarder.rs`
   - **Developer B**: User Story 2 (Path Traversal) - `src/dashboard/server.rs`
   - **Developer C**: User Story 3 (Syntax Fixes) - `src/otlp/forwarder.rs`, `src/config/types.rs`
   - **Developer D**: User Story 4 (Memory Limits) - `src/config/types.rs`, `src/otlp/batch_writer.rs`
3. Stories complete and integrate independently (different files, minimal conflicts)
4. Continue with P2 stories in parallel

---

## Notes

- **[P] tasks** = different files, no dependencies - can run in parallel
- **[Story] label** maps task to specific user story for traceability
- Each user story should be independently completable and testable
- **TDD approach**: Write tests first, ensure they FAIL, then implement fixes
- Commit after each task or logical group (per user story recommended)
- Stop at any checkpoint to validate story independently
- **Avoid**: vague tasks, same file conflicts, cross-story dependencies that break independence
- **Breaking change**: User Story 1 (credential storage) requires API updates - document in CHANGELOG.md
- **Test coverage**: Maintain 85% per file requirement per constitution
- **All fixes are localized**: Each story touches specific files with minimal overlap

---

## Task Summary

- **Total Tasks**: 112
- **Setup Tasks**: 3 (T001-T003)
- **Foundational Tasks**: 2 (T004-T005)
- **User Story 1 Tasks**: 10 (T006-T015) - 4 tests, 6 implementation
- **User Story 2 Tasks**: 11 (T016-T026) - 6 tests, 5 implementation
- **User Story 3 Tasks**: 6 (T027-T032) - 2 tests, 4 implementation
- **User Story 4 Tasks**: 14 (T033-T046) - 4 tests, 10 implementation
- **User Story 5 Tasks**: 11 (T047-T057) - 4 tests, 7 implementation
- **User Story 6 Tasks**: 9 (T058-T066) - 4 tests, 5 implementation
- **User Story 7 Tasks**: 10 (T067-T076) - 4 tests, 6 implementation
- **User Story 8 Tasks**: 10 (T077-T086) - 3 tests, 7 implementation
- **User Story 9 Tasks**: 7 (T087-T093) - 3 tests, 4 implementation
- **User Story 10 Tasks**: 8 (T094-T101) - 2 tests, 6 implementation
- **Polish Tasks**: 11 (T102-T112)

**Parallel Opportunities**: High - most user stories can be worked on in parallel after foundational phase

**Suggested MVP Scope**: User Stories 1-4 (P1) - Critical security and stability fixes

