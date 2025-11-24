const hexFrom = (value, fallbackLength = 32) => {
  if (!value) {
    return '0'.repeat(fallbackLength);
  }

  if (typeof value === 'string') {
    return value.toLowerCase();
  }

  if (value instanceof Uint8Array) {
    return Array.from(value, (byte) => byte.toString(16).padStart(2, '0')).join('');
  }

  if (typeof value === 'number') {
    return value.toString(16).padStart(fallbackLength, '0');
  }

  if (typeof value === 'bigint') {
    return value.toString(16).padStart(fallbackLength, '0');
  }

  return String(value);
};

const parseAttributes = (rawAttributes) => {
  if (!rawAttributes) {
    return {};
  }
  if (typeof rawAttributes === 'object' && !Array.isArray(rawAttributes)) {
    return rawAttributes;
  }
  try {
    return JSON.parse(rawAttributes);
  } catch {
    return {};
  }
};

export const createTraceEntry = (row) => {
  const start = Number(row.start_time_unix_nano ?? row.startTime ?? Date.now() * 1_000_000);
  const end = Number(row.end_time_unix_nano ?? row.endTime ?? start);
  const attributes = parseAttributes(row.attributes);
  const serviceName = row.service_name ?? row.serviceName ?? attributes['service.name'] ?? 'unknown';
  const statusCode = String(row.status_code ?? row.statusCode ?? 'unset').toLowerCase();
  const duration = Math.max(0, (end - start) / 1_000_000);

  return {
    traceId: hexFrom(row.trace_id ?? row.traceId ?? '0', 32),
    spanId: hexFrom(row.span_id ?? row.spanId ?? '0', 16),
    parentSpanId: row.parent_span_id ? hexFrom(row.parent_span_id, 16) : null,
    name: row.name ?? 'unknown',
    serviceName,
    startTime: start,
    endTime: end,
    duration,
    statusCode,
    statusMessage: row.status_message ?? row.statusMessage ?? null,
    attributes,
    events: row.events ?? [],
    links: row.links ?? [],
    kind: row.kind ?? 'internal',
    error: statusCode === 'error',
  };
};

export const sortByStartTimeDesc = (a, b) => b.startTime - a.startTime;
