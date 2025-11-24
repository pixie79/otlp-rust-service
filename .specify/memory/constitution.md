<!--
Sync Impact Report:
Version change: 1.0.1 → 1.1.0
Modified principles: None
Added sections:
  - Development Workflow & Quality Gates: Added "Commit Workflow" subsection with CHANGELOG.md requirements, documentation updates, and GPG signing mandate
Modified sections:
  - Development Workflow & Quality Gates: Enhanced Pre-Commit Checks to reference commit workflow requirements
Templates requiring updates:
  ✅ constitution.md (this file)
  ✅ plan-template.md (updated Constitution Check section to reference commit workflow)
  ⚠️ spec-template.md (optional: may add note about commit workflow in development process if applicable)
Follow-up TODOs: None
-->

# OTLP Rust Service Constitution

## Core Principles

### I. Code Quality (NON-NEGOTIABLE)

All code MUST adhere to Rust best practices and maintainability standards. Code quality is enforced through automated tooling and peer review. All code MUST pass `cargo clippy` with no warnings before merge. Code MUST be formatted with `rustfmt` using project-standard configuration. Functions MUST be documented with doc comments explaining purpose, parameters, return values, and error conditions. Complex logic MUST include inline comments explaining the "why" not just the "what". Code MUST follow SOLID principles: single responsibility, clear interfaces, dependency inversion. Cyclomatic complexity MUST be kept low; functions exceeding 15 complexity points MUST be refactored or justified. Dead code, unused imports, and commented-out code MUST be removed before merge. All public APIs MUST have comprehensive documentation with examples.

### II. Testing Standards (NON-NEGOTIABLE)

Test-Driven Development (TDD) is mandatory for all new features: tests written → reviewed → tests fail → implementation → tests pass → refactor. All code MUST have corresponding tests with minimum 85% code coverage for every file. Unit tests MUST be fast (<100ms each), isolated, and deterministic. Integration tests MUST cover all external interfaces including OTLP protocol endpoints, database interactions, and service boundaries. Contract tests MUST validate OTLP protocol compliance and schema conformance. Property-based tests MUST be used for data transformation and validation logic. Performance tests MUST be included for critical paths with defined latency and throughput targets. All tests MUST be independent and runnable in parallel. Flaky tests are treated as blocking issues and MUST be fixed immediately. Test code MUST follow the same quality standards as production code.

### III. User Experience Consistency

All user-facing interfaces (APIs, CLI, configuration) MUST follow consistent patterns and conventions. Error messages MUST be clear, actionable, and include context for debugging. API responses MUST follow consistent structure with standardized error formats. Configuration MUST use environment variables with clear naming conventions (OTLP_* prefix). Logging MUST be structured and consistent across all components using a unified format. All user-facing documentation MUST be kept up-to-date with code changes. Breaking changes to public APIs MUST follow semantic versioning and include migration guides. User experience decisions MUST be documented in ADRs (Architecture Decision Records) when they affect multiple components.

### IV. Performance Requirements

All performance-critical paths MUST meet defined Service Level Objectives (SLOs). OTLP ingestion endpoints MUST handle at least [TBD] requests per second with p95 latency < [TBD]ms. Memory usage MUST be bounded and monitored; memory leaks are treated as critical bugs. CPU usage MUST be optimized for high-throughput scenarios; profiling MUST be performed on hot paths. Database queries MUST be optimized and indexed; N+1 query patterns are prohibited. Async operations MUST be used for I/O-bound tasks; blocking operations in async contexts are prohibited. Resource cleanup MUST be guaranteed through RAII patterns and proper error handling. Performance regressions MUST be caught by automated benchmarks in CI/CD. All performance requirements MUST be documented with measurement methodology and target metrics.

### V. Observability & Reliability

All services MUST emit structured logs with appropriate log levels (ERROR, WARN, INFO, DEBUG, TRACE). Metrics MUST be exposed via Prometheus-compatible endpoints for all critical operations. Distributed tracing MUST be implemented for request flows across service boundaries. Health check endpoints MUST be provided and monitored. Error rates, latency, and throughput MUST be tracked and alertable. All external dependencies MUST have circuit breakers and timeout configurations. Graceful degradation MUST be implemented for non-critical features during failures. Incident response procedures MUST be documented and tested.

## Code Quality Standards

### Rust-Specific Requirements

- Minimum Rust version: stable channel (latest stable)
- All dependencies MUST be kept up-to-date with security patches applied within 48 hours
- `unsafe` code MUST be minimized and requires explicit justification and review
- Panics MUST be avoided in library code; use `Result<T, E>` for error handling
- All public types MUST implement `Debug` and `Clone` where appropriate
- Serialization/deserialization MUST use `serde` with explicit derive attributes
- Async code MUST use `tokio` runtime with proper error propagation

### Code Review Requirements

- All code MUST be reviewed by at least one other developer before merge
- Reviewers MUST verify compliance with all constitution principles
- PRs MUST include tests demonstrating new functionality
- PRs MUST include performance impact assessment for changes affecting hot paths
- PRs MUST update documentation for any user-facing changes

## Development Workflow & Quality Gates

### Commit Workflow (NON-NEGOTIABLE)

Before creating any commit, the following MUST be completed in order:

1. **Documentation Updates**: CHANGELOG.md MUST be updated with all changes for the commit. All relevant documentation (README.md, API docs, user guides) MUST be updated to reflect code changes. Documentation MUST be accurate and current before commit.

2. **Code Quality Checks**: The following commands MUST pass without errors or warnings:
   - `cargo fmt` MUST be run to format code (or `cargo fmt --check` must pass)
   - `cargo clippy -- -D warnings` MUST pass with no warnings
   - `cargo test` MUST pass (all unit, integration, and documentation tests)

3. **GPG Signing**: All commits MUST be GPG signed. Commits without valid GPG signatures are prohibited and MUST be rejected.

4. **Pre-Push Validation**: Before pushing commits, all quality gates MUST pass. No commit may be pushed if any of the above requirements are not met.

This workflow ensures code quality, documentation accuracy, and commit authenticity. Violations of this workflow MUST be corrected before merge or push.

### Pre-Commit Checks

- `cargo fmt --check` MUST pass (code formatting)
- `cargo clippy -- -D warnings` MUST pass (linting)
- `cargo test` MUST pass (all tests)
- `cargo test --doc` MUST pass (documentation tests)
- Code coverage MUST not decrease below 85% threshold per file
- CHANGELOG.md MUST be current (see Commit Workflow)
- All documentation MUST be updated (see Commit Workflow)

### CI/CD Pipeline Gates

- All pre-commit checks MUST pass
- Integration tests MUST pass in CI environment
- Performance benchmarks MUST not regress beyond 5% threshold
- Security scanning MUST pass (cargo audit, etc.)
- Documentation build MUST succeed

### Release Process

- All quality gates MUST pass
- Version MUST follow semantic versioning (MAJOR.MINOR.PATCH)
- Breaking changes MUST be documented in CHANGELOG.md
- Performance metrics MUST be validated against SLOs
- Release notes MUST be generated and reviewed

## Governance

This constitution supersedes all other development practices and standards. All team members MUST comply with these principles. Amendments to this constitution require:

1. **Proposal**: Document the proposed change with rationale and impact analysis
2. **Review**: Team review and discussion period (minimum 48 hours)
3. **Approval**: Consensus or majority vote from core maintainers
4. **Implementation**: Update constitution, propagate changes to templates, update documentation
5. **Versioning**: Increment version according to semantic versioning:
   - MAJOR: Backward incompatible principle removals or redefinitions
   - MINOR: New principle/section added or materially expanded guidance
   - PATCH: Clarifications, wording improvements, typo fixes

All PRs and code reviews MUST verify compliance with the constitution. Violations MUST be addressed before merge. Complexity that violates principles MUST be justified with documented rationale and approved exceptions.

**Version**: 1.1.0 | **Ratified**: 2024-11-23 | **Last Amended**: 2024-12-19
