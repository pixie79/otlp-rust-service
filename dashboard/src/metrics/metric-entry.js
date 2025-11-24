const parseLabels = (rawLabels) => {
  if (!rawLabels) {
    return {};
  }
  if (typeof rawLabels === 'object' && !Array.isArray(rawLabels)) {
    return rawLabels;
  }
  try {
    return JSON.parse(rawLabels);
  } catch {
    return {};
  }
};

/**
 * Create a MetricEntry from a database row or object
 * @param {Object} row - Row from DuckDB query or object with metric data
 * @returns {Object} MetricEntry
 */
export const createMetricEntry = (row) => {
  const timestamp = Number(row.timestamp_unix_nano ?? row.timestamp ?? Date.now() * 1_000_000);
  const value = Number(row.value ?? 0);
  const labels = parseLabels(row.labels ?? row.attributes);

  return {
    metricName: row.metric_name ?? row.metricName ?? 'unknown',
    value,
    timestamp,
    labels,
    metricType: row.metric_type ?? row.metricType ?? 'gauge',
    unit: row.unit ?? null,
  };
};

/**
 * Sort metrics by timestamp ascending (for time-series display)
 */
export const sortByTimestampAsc = (a, b) => a.timestamp - b.timestamp;

/**
 * Sort metrics by timestamp descending (newest first)
 */
export const sortByTimestampDesc = (a, b) => b.timestamp - a.timestamp;

/**
 * Group metrics by metric name and labels
 * @param {Array<MetricEntry>} metrics
 * @returns {Map<string, Array<MetricEntry>>} Map of series key to metrics
 */
export const groupMetricsBySeries = (metrics) => {
  const grouped = new Map();

  for (const metric of metrics) {
    // Create series key from metric name and sorted labels
    const labelKeys = Object.keys(metric.labels).sort();
    const labelString = labelKeys.map((k) => `${k}=${metric.labels[k]}`).join(',');
    const seriesKey = labelString ? `${metric.metricName}{${labelString}}` : metric.metricName;

    if (!grouped.has(seriesKey)) {
      grouped.set(seriesKey, []);
    }
    grouped.get(seriesKey).push(metric);
  }

  // Sort each series by timestamp
  for (const series of grouped.values()) {
    series.sort(sortByTimestampAsc);
  }

  return grouped;
};
