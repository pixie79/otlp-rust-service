# Type Conversion Contract: Python OpenTelemetry SDK to Library Formats

**Date**: 2025-11-25  
**Feature**: 004-python-otel-adapters

## Overview

This contract defines the type conversion between Python OpenTelemetry SDK types and library-compatible dictionary formats. The conversion must preserve 100% of metric and span data without loss or corruption.

## Metric Type Conversion

### Input: Python OpenTelemetry SDK `MetricExportResult`

**Structure**:
```python
MetricExportResult(
    resource_metrics: ResourceMetrics
)
```

**ResourceMetrics Structure**:
```python
ResourceMetrics(
    resource: Resource,
    scope_metrics: List[ScopeMetrics]
)
```

**Resource Structure**:
```python
Resource(
    attributes: Dict[str, AttributeValue]
)
```

**ScopeMetrics Structure**:
```python
ScopeMetrics(
    scope: InstrumentationScope,
    metrics: List[Metric]
)
```

**Metric Structure**:
```python
Metric(
    name: str,
    description: Optional[str],
    unit: Optional[str],
    data: MetricData  # GaugeData, SumData, or HistogramData
)
```

### Output: Library-Compatible Dictionary

**Structure**:
```python
{
    "resource": {
        "attributes": {
            "key": value,  # String, int, float, bool
            ...
        }
    },
    "scope_metrics": [
        {
            "scope": {
                "name": str,
                "version": Optional[str],
                "attributes": Optional[Dict[str, AttributeValue]]
            },
            "metrics": [
                {
                    "name": str,
                    "description": Optional[str],
                    "unit": Optional[str],
                    "data": {
                        # GaugeData, SumData, or HistogramData structure
                    }
                },
                ...
            ]
        },
        ...
    ]
}
```

### Conversion Rules

1. **Resource Attributes**:
   - Convert `Dict[str, AttributeValue]` → `Dict[str, Union[str, int, float, bool]]`
   - Preserve all attribute keys and values
   - Handle all attribute value types (string, int, float, bool, list, dict)

2. **Instrumentation Scope**:
   - Extract `name` (required)
   - Extract `version` (optional)
   - Extract `attributes` (optional)
   - Preserve all scope information

3. **Metric Data Points**:
   - **GaugeData**: Preserve gauge values and timestamps
   - **SumData**: Preserve sum values, aggregation temporality, and timestamps
   - **HistogramData**: Preserve histogram buckets, counts, and timestamps
   - Preserve all metric metadata (name, description, unit)

4. **Data Preservation**:
   - All resource attributes preserved
   - All instrumentation scope information preserved
   - All metric data points preserved
   - All metric values preserved (no rounding or truncation)
   - All timestamps preserved (nanoseconds precision)

### Error Handling

- **Invalid Input**: Raise `ValueError` with details about invalid data
- **Conversion Failure**: Raise `RuntimeError` with conversion error details
- **Missing Required Fields**: Raise `ValueError` with missing field information

---

## Span Type Conversion

### Input: Python OpenTelemetry SDK `Sequence[ReadableSpan]`

**ReadableSpan Structure**:
```python
ReadableSpan(
    name: str,
    context: SpanContext,
    parent: Optional[SpanContext],
    kind: SpanKind,
    start_time: Optional[int],  # nanoseconds
    end_time: Optional[int],    # nanoseconds
    attributes: Dict[str, AttributeValue],
    events: List[SpanEvent],
    links: List[SpanLink],
    status: Status,
    resource: Resource,
    instrumentation_scope: InstrumentationScope
)
```

**SpanContext Structure**:
```python
SpanContext(
    trace_id: int,  # 128-bit
    span_id: int,   # 64-bit
    trace_flags: TraceFlags,
    trace_state: TraceState
)
```

**SpanEvent Structure**:
```python
SpanEvent(
    name: str,
    timestamp: int,  # nanoseconds
    attributes: Dict[str, AttributeValue]
)
```

**SpanLink Structure**:
```python
SpanLink(
    context: SpanContext,
    attributes: Dict[str, AttributeValue]
)
```

### Output: Library-Compatible Dictionary List

**Structure** (per span):
```python
{
    "trace_id": bytes,  # 16 bytes
    "span_id": bytes,    # 8 bytes
    "parent_span_id": Optional[bytes],  # 8 bytes
    "name": str,
    "kind": str,  # "server", "client", "internal", "producer", "consumer"
    "attributes": Dict[str, Union[str, int, float, bool]],
    "events": [
        {
            "name": str,
            "timestamp": int,  # nanoseconds
            "attributes": Dict[str, Union[str, int, float, bool]]
        },
        ...
    ],
    "links": [
        {
            "trace_id": bytes,  # 16 bytes
            "span_id": bytes,    # 8 bytes
            "attributes": Dict[str, Union[str, int, float, bool]]
        },
        ...
    ],
    "status": str,  # "ok", "error", "unset"
    "status_message": Optional[str],
    "start_time": int,  # nanoseconds
    "end_time": int     # nanoseconds
}
```

### Conversion Rules

1. **Trace and Span IDs**:
   - Convert `trace_id: int` (128-bit) → `bytes` (16 bytes, big-endian)
   - Convert `span_id: int` (64-bit) → `bytes` (8 bytes, big-endian)
   - Convert `parent.span_id: int` (64-bit) → `bytes` (8 bytes, big-endian, optional)

2. **Span Kind**:
   - Convert `SpanKind` enum → string:
     - `SpanKind.SERVER` → `"server"`
     - `SpanKind.CLIENT` → `"client"`
     - `SpanKind.INTERNAL` → `"internal"`
     - `SpanKind.PRODUCER` → `"producer"`
     - `SpanKind.CONSUMER` → `"consumer"`

3. **Attributes**:
   - Convert `Dict[str, AttributeValue]` → `Dict[str, Union[str, int, float, bool]]`
   - Preserve all attribute keys and values
   - Handle nested structures (lists, dicts) by converting to strings if needed

4. **Events**:
   - Preserve all event names, timestamps, and attributes
   - Convert event attributes using same rules as span attributes

5. **Links**:
   - Preserve all link trace_ids, span_ids, and attributes
   - Convert link trace_ids and span_ids using same rules as span IDs
   - Convert link attributes using same rules as span attributes

6. **Status**:
   - Convert `Status` enum → string:
     - `Status.OK` → `"ok"`
     - `Status.ERROR` → `"error"`
     - `Status.UNSET` → `"unset"`
   - Extract status message if available

7. **Timestamps**:
   - Preserve `start_time` and `end_time` as nanoseconds (int)
   - Preserve event timestamps as nanoseconds (int)

8. **Data Preservation**:
   - All span identifiers preserved (trace_id, span_id, parent_span_id)
   - All span metadata preserved (name, kind, status)
   - All attributes preserved (key-value pairs)
   - All events preserved (with timestamps and attributes)
   - All links preserved (with trace_id, span_id, attributes)
   - All timestamps preserved (start_time, end_time, event times)

### Error Handling

- **Invalid Input**: Raise `ValueError` with details about invalid data
- **Conversion Failure**: Raise `RuntimeError` with conversion error details
- **Missing Required Fields**: Raise `ValueError` with missing field information
- **Invalid ID Format**: Raise `ValueError` if trace_id or span_id cannot be converted

---

## Error Conversion

### Input: Rust `OtlpError` or Python Exceptions

**OtlpError Types**:
- `OtlpError::IoError`
- `OtlpError::ConfigError`
- `OtlpError::ExportError`
- `OtlpError::ConversionError`

### Output: Python Exceptions

**Conversion Rules**:
- `OtlpError::IoError` → `RuntimeError` with "IO error: {message}"
- `OtlpError::ConfigError` → `RuntimeError` with "Configuration error: {message}"
- `OtlpError::ExportError` → `RuntimeError` with "Export error: {message}"
- `OtlpError::ConversionError` → `RuntimeError` with "Conversion error: {message}"

**Error Context Preservation**:
- Original error message included in Python exception message
- Error type information included in exception message
- Stack traces preserved when available

---

## Performance Requirements

- **Type Conversion Overhead**: < 10% of total export time
- **Memory Usage**: Bounded, no memory leaks during conversion
- **Concurrent Conversion**: Thread-safe, supports concurrent conversions

## Validation

- **Data Completeness**: 100% of input data must be present in output
- **Data Correctness**: All converted values must match input values
- **Type Safety**: Output types must match library API expectations
- **Error Handling**: All conversion errors must be caught and converted appropriately

