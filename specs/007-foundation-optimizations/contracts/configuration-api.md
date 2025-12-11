# Configuration API Contract: Temporality Configuration

**Feature**: 007-foundation-optimizations  
**Date**: 2025-01-27  
**Status**: Complete

## Overview

This contract defines the API for configuring temporality in metric exporters.

## Temporality Configuration

### Builder Pattern

**Location**: `src/api/public.rs`, `src/otlp/exporter.rs`

**API**:
```rust
// Rust API
let exporter = OtlpMetricExporterBuilder::new()
    .with_temporality(Temporality::Cumulative)  // or Temporality::Delta
    .build();

// Default (if not specified)
let exporter = OtlpMetricExporterBuilder::new()
    .build();  // Uses Cumulative temporality by default
```

**Python API**:
```python
# Python API
exporter = library.metric_exporter()
exporter.set_temporality(Temporality.CUMULATIVE)  # or Temporality.DELTA

# Default (if not specified)
exporter = library.metric_exporter()  # Uses CUMULATIVE temporality by default
```

---

## Temporality Enum

**Type**: `opentelemetry_sdk::metrics::data::Temporality`

**Values**:
- `Cumulative`: Metrics accumulate values over time (default)
- `Delta`: Metrics represent changes since last export

**Default**: `Cumulative`

---

## API Methods

### Rust API

**Builder Method**:
```rust
impl OtlpMetricExporterBuilder {
    pub fn with_temporality(mut self, temporality: Temporality) -> Self {
        self.temporality = temporality;
        self
    }
}
```

**Getter Method** (required by OpenTelemetry SDK):
```rust
impl OtlpMetricExporter {
    pub fn temporality(&self) -> Temporality {
        self.temporality
    }
}
```

### Python API

**Setter Method**:
```python
def set_temporality(self, temporality: Temporality) -> None:
    """Set the temporality mode for metric export."""
    self._temporality = temporality
```

**Getter Method** (required by Python OpenTelemetry SDK):
```python
def temporality(self) -> Temporality:
    """Return the preferred temporality mode."""
    return self._temporality
```

---

## Backward Compatibility

### Default Behavior

- If temporality is not specified, default to `Cumulative`
- Existing code continues to work without changes
- New code can opt-in to `Delta` temporality

### Migration

**No migration required** - existing code continues to work with default `Cumulative` temporality.

**Opt-in to Delta**:
```rust
// Before (implicit Cumulative)
let exporter = OtlpMetricExporterBuilder::new().build();

// After (explicit Cumulative - same behavior)
let exporter = OtlpMetricExporterBuilder::new()
    .with_temporality(Temporality::Cumulative)
    .build();

// New (explicit Delta)
let exporter = OtlpMetricExporterBuilder::new()
    .with_temporality(Temporality::Delta)
    .build();
```

---

## Validation

### Validation Rules

1. Temporality must be a valid `Temporality` enum value
2. Default value must be `Cumulative` if not specified
3. Configuration must be applied before exporter is used
4. Temporality cannot be changed after exporter is created

### Error Handling

- Invalid temporality values: Compile-time error (enum type)
- No runtime errors for temporality configuration

---

## Examples

### Rust Example

```rust
use opentelemetry_sdk::metrics::data::Temporality;

// Create exporter with Cumulative temporality (default)
let exporter_cumulative = OtlpMetricExporterBuilder::new()
    .with_temporality(Temporality::Cumulative)
    .build();

// Create exporter with Delta temporality
let exporter_delta = OtlpMetricExporterBuilder::new()
    .with_temporality(Temporality::Delta)
    .build();

// Use default (Cumulative)
let exporter_default = OtlpMetricExporterBuilder::new()
    .build();
```

### Python Example

```python
from opentelemetry.sdk.metrics.export import Temporality

# Create exporter with Cumulative temporality (default)
exporter_cumulative = library.metric_exporter()
exporter_cumulative.set_temporality(Temporality.CUMULATIVE)

# Create exporter with Delta temporality
exporter_delta = library.metric_exporter()
exporter_delta.set_temporality(Temporality.DELTA)

# Use default (Cumulative)
exporter_default = library.metric_exporter()
```

---

## Notes

- Temporality configuration is optional (defaults to Cumulative)
- Backward compatible - existing code continues to work
- Builder pattern consistent with existing exporter APIs
- Python API matches Python OpenTelemetry SDK patterns
