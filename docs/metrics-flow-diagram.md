# Metrics Import/Export Flow Diagram

This document describes all the import and export mechanisms for metrics, showing the data format (Protobuf/Arrow) at each step.

## Visual Overview

```
┌─────────────────────────────────────────────────────────────────────────┐
│                         IMPORT PATHS (Receiving)                        │
└─────────────────────────────────────────────────────────────────────────┘

Path 1: gRPC Protobuf (Port 4317)
─────────────────────────────────
Client SDK → opentelemetry-otlp exporter
  ↓ [ResourceMetrics → Protobuf] (in client)
Protobuf (ExportMetricsServiceRequest)
  ↓ [gRPC network]
Our Server → export_metrics_from_protobuf()
  ↓ [Protobuf → Arrow RecordBatch] (internal format)
  ↓ [Arrow RecordBatch → Arrow IPC bytes]
Arrow IPC File (.arrow)

Path 2: gRPC Arrow Flight (Port 4318)
──────────────────────────────────────
Client → Arrow Flight client
  ↓
Arrow RecordBatch
  ↓ [gRPC Arrow Flight network]
Our Server → Arrow Flight handler
  ↓ [Arrow RecordBatch] (internal format, re-format if needed)
  ↓ [Arrow RecordBatch → Arrow IPC bytes]
Arrow IPC File (.arrow)

Path 3: Direct API - Protobuf
──────────────────────────────
User Code → ExportMetricsServiceRequest (Protobuf)
  ↓
library.export_metrics(protobuf)
  ↓ [Protobuf → Arrow RecordBatch] (internal format)
  ↓ [Arrow RecordBatch → Arrow IPC bytes]
Arrow IPC File (.arrow)

Path 4: Direct API - Arrow RecordBatch
───────────────────────────────────────
User Code → Arrow RecordBatch
  ↓ [Arrow RecordBatch] (internal format, direct)
  ↓ [Arrow RecordBatch → Arrow IPC bytes]
Arrow IPC File (.arrow)

If forwarding needed:
  ↓ [Arrow RecordBatch → Protobuf] (only if forwarding Protobuf)
  ↓ [Forward Protobuf to remote endpoint]
  OR
  ↓ [Arrow RecordBatch] (keep as Arrow if forwarding Arrow Flight)
  ↓ [Forward Arrow RecordBatch to remote endpoint]

┌─────────────────────────────────────────────────────────────────────────┐
│                      EXPORT PATHS (Forwarding)                          │
└─────────────────────────────────────────────────────────────────────────┘

Path 5: Forward Protobuf → Protobuf
────────────────────────────────────
gRPC Server receives Protobuf
  ↓ [Protobuf] (internal format for forwarding)
  ↓ [store in buffer as Protobuf]
  ↓ [if forwarding enabled]
Forward Protobuf directly
  ↓ [gRPC network]
Remote Endpoint

Path 6: Forward Protobuf → Arrow Flight
────────────────────────────────────────
gRPC Server receives Protobuf
  ↓ [Protobuf → Arrow RecordBatch] (internal format for forwarding)
  ↓ [if forwarding enabled]
Forward Arrow RecordBatch
  ↓ [Arrow Flight network]
Remote Endpoint

Path 7: Forward Arrow Flight → Protobuf
────────────────────────────────────────
Arrow Flight Server receives Arrow RecordBatch
  ↓ [Arrow RecordBatch → Protobuf] (internal format for forwarding)
  ↓ [if forwarding enabled]
Forward Protobuf
  ↓ [gRPC network]
Remote Endpoint

Path 8: Forward Arrow Flight → Arrow Flight
────────────────────────────────────────────
Arrow Flight Server receives Arrow RecordBatch
  ↓ [Arrow RecordBatch] (internal format for forwarding)
  ↓ [if forwarding enabled]
Forward Arrow RecordBatch directly
  ↓ [Arrow Flight network]
Remote Endpoint
```

## Import Paths (Receiving Metrics)

### Path 1: gRPC Protobuf Server (Port 4317)
```
Client Application
  ↓
opentelemetry-otlp exporter
  ↓ (converts ResourceMetrics → Protobuf internally)
ExportMetricsServiceRequest (Protobuf)
  ↓ (gRPC over network)
Our gRPC Server (MetricsServiceImpl)
  ↓ (receives Protobuf directly)
export_metrics_from_protobuf()
  ↓
Protobuf → Arrow RecordBatch (internal format)
  ↓
Arrow RecordBatch → Arrow IPC bytes
  ↓
Write to Arrow IPC file (.arrow)
```

**Format at each step:**
- Input: Protobuf (ExportMetricsServiceRequest)
- Internal: Arrow RecordBatch (ideal internal format for storage)
- Output: Arrow IPC file

---

### Path 2: gRPC Arrow Flight Server (Port 4318)
```
Client Application
  ↓
Arrow Flight client
  ↓
Arrow RecordBatch (Arrow Flight format)
  ↓ (gRPC Arrow Flight over network)
Our Arrow Flight Server
  ↓ (receives Arrow RecordBatch)
Arrow RecordBatch (internal format, re-format if needed)
  ↓
Arrow RecordBatch → Arrow IPC bytes
  ↓
Write to Arrow IPC file (.arrow)
```

**Format at each step:**
- Input: Arrow RecordBatch (Arrow Flight)
- Internal: Arrow RecordBatch (ideal internal format for storage)
- Output: Arrow IPC file

---

### Path 3: Direct API Call - export_metrics(protobuf)
```
User Code
  ↓
ExportMetricsServiceRequest (Protobuf) - user creates this
  ↓
library.export_metrics(protobuf)
  ↓
export_metrics_from_protobuf()
  ↓
Protobuf → Arrow RecordBatch (internal format)
  ↓
Arrow RecordBatch → Arrow IPC bytes
  ↓
Write to Arrow IPC file (.arrow)
```

**Format at each step:**
- Input: Protobuf (ExportMetricsServiceRequest)
- Internal: Arrow RecordBatch (ideal internal format for storage)
- Output: Arrow IPC file

---

### Path 4: Direct API Call - Arrow RecordBatch

**Ideal Path (Recommended):**
```
User Code
  ↓
Arrow RecordBatch - user creates this (Arrow Flight format)
  ↓
Arrow RecordBatch → Arrow IPC bytes (direct serialization)
  ↓
Write to Arrow IPC file (.arrow)
```

**Format at each step:**
- Input: Arrow RecordBatch (Arrow Flight format)
- Internal: Arrow RecordBatch (ideal internal format for storage)
- Direct conversion: Arrow RecordBatch → Arrow IPC bytes (no intermediate formats)
- Output: Arrow IPC file

**For Forwarding (if needed):**
```
Arrow RecordBatch (internal format)
  ↓ (if forwarding Protobuf)
Arrow RecordBatch → Protobuf (convert only when needed)
  ↓
Forward Protobuf to remote endpoint

OR

Arrow RecordBatch (internal format)
  ↓ (if forwarding Arrow Flight)
Forward Arrow RecordBatch directly to remote endpoint
```

**Note:** The ideal implementation keeps Arrow RecordBatch as the internal format, only converting to Protobuf when forwarding requires it.

---

## Export Paths (Forwarding Metrics)

### Path 5: Forward from gRPC Protobuf - Protobuf
```
gRPC Server receives Protobuf
  ↓
Protobuf (internal format for forwarding)
  ↓ (if forwarding enabled)
Protobuf stored in buffer (as Protobuf)
  ↓ (when forwarding)
Forward Protobuf directly to remote endpoint
  ↓
Send Protobuf via gRPC to remote endpoint
```

**Format at each step:**
- Input: Protobuf (from gRPC)
- Internal: Protobuf (ideal internal format for Protobuf forwarding)
- Buffer: Protobuf (ExportMetricsServiceRequest) - Clone-able format
- Output: Protobuf sent over network

---

### Path 6: Forward from gRPC Protobuf - Arrow Flight
```
gRPC Server receives Protobuf
  ↓
Protobuf → Arrow RecordBatch (internal format for forwarding)
  ↓ (if forwarding enabled)
Send Arrow RecordBatch via Arrow Flight to remote endpoint
```

**Format at each step:**
- Input: Protobuf (from gRPC)
- Internal: Arrow RecordBatch (ideal internal format for Arrow Flight forwarding)
- Output: Arrow RecordBatch (Arrow Flight) sent over network

---

### Path 7: Forward from Arrow Flight - Protobuf
```
Arrow Flight Server receives Arrow RecordBatch
  ↓
Arrow RecordBatch → Protobuf (internal format for forwarding)
  ↓ (if forwarding enabled)
Send Protobuf via gRPC to remote endpoint
```

**Format at each step:**
- Input: Arrow RecordBatch (from Arrow Flight)
- Internal: Protobuf (ideal internal format for Protobuf forwarding)
- Output: Protobuf sent over network

---

### Path 8: Forward from Arrow Flight - Arrow Flight
```
Arrow Flight Server receives Arrow RecordBatch
  ↓
Arrow RecordBatch (internal format for forwarding)
  ↓ (if forwarding enabled)
Forward Arrow RecordBatch directly
  ↓
Send Arrow RecordBatch via Arrow Flight to remote endpoint
```

**Format at each step:**
- Input: Arrow RecordBatch (from Arrow Flight)
- Internal: Arrow RecordBatch (ideal internal format for Arrow Flight forwarding)
- Output: Arrow RecordBatch (Arrow Flight) sent over network

---

## Summary Table

| Path | Input Format | Internal Format | Storage/Output Format | Proxy Needed? |
|------|-------------|-----------------|----------------------|---------------|
| **Import 1: gRPC Protobuf** | Protobuf | Arrow RecordBatch | Arrow IPC file | ❌ No |
| **Import 2: gRPC Arrow Flight** | Arrow Flight | Arrow RecordBatch | Arrow IPC file | ❌ No |
| **Import 3: Direct API - Protobuf** | Protobuf | Arrow RecordBatch | Arrow IPC file | ❌ No |
| **Import 4: Direct API - Arrow** | Arrow RecordBatch | Arrow RecordBatch | Arrow IPC file | ❌ No |
| **Forward 5: Protobuf → Protobuf** | Protobuf | Protobuf | Protobuf (network) | ❌ No |
| **Forward 6: Protobuf → Arrow Flight** | Protobuf | Arrow RecordBatch | Arrow Flight (network) | ❌ No |
| **Forward 7: Arrow Flight → Protobuf** | Arrow Flight | Protobuf | Protobuf (network) | ❌ No |
| **Forward 8: Arrow Flight → Arrow Flight** | Arrow Flight | Arrow RecordBatch | Arrow Flight (network) | ❌ No |

## Key Points

1. **All import paths convert to Arrow RecordBatch internally** - This is the ideal internal format for storage
2. **Internal format matches forwarding needs** - For forwarding:
   - Forward Protobuf → Keep as Protobuf internally
   - Forward Arrow Flight → Keep as Arrow RecordBatch internally
3. **No proxy needed** - All conversions use direct methods:
   - Protobuf → Arrow RecordBatch (for storage)
   - Arrow RecordBatch → Protobuf (only when forwarding Protobuf)
   - Arrow RecordBatch → Arrow IPC bytes (for file storage)
4. **gRPC servers receive data directly** - No ResourceMetrics involved, only Protobuf or Arrow Flight
5. **Efficient conversions** - Only convert formats when necessary (storage uses Arrow, forwarding uses target format)

## Removed Paths (No Longer Supported)

- ❌ `export_metrics_arrow(&ResourceMetrics)` - Removed (required proxy)
- ❌ `OtlpMetricExporter` - Removed (required proxy)

**Alternative:** Users can use `opentelemetry-otlp` exporter pointing to our gRPC server, which sends Protobuf directly (no proxy needed).

