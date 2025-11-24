/**
 * MetricAggregator component for aggregating histogram metrics
 * Implements aggregation logic for histogram metrics (sum, avg, min, max, p50, p95, p99)
 * per data-model.md
 */
export class MetricAggregator {
  /**
   * Aggregate histogram metrics
   * @param {Array<MetricEntry>} metrics - Array of metric entries to aggregate
   * @returns {Object} Aggregated metrics with sum, count, min, max, avg, p50, p95, p99
   */
  static aggregateHistogram(metrics) {
    if (!metrics || metrics.length === 0) {
      return {
        sum: 0,
        count: 0,
        min: 0,
        max: 0,
        avg: 0,
        p50: 0,
        p95: 0,
        p99: 0,
      };
    }

    const values = metrics.map((m) => Number(m.value)).filter((v) => !isNaN(v));
    if (values.length === 0) {
      return {
        sum: 0,
        count: 0,
        min: 0,
        max: 0,
        avg: 0,
        p50: 0,
        p95: 0,
        p99: 0,
      };
    }

    const sorted = [...values].sort((a, b) => a - b);
    const sum = values.reduce((acc, v) => acc + v, 0);
    const count = values.length;
    const min = sorted[0];
    const max = sorted[sorted.length - 1];
    const avg = sum / count;

    const percentile = (sorted, p) => {
      if (sorted.length === 0) return 0;
      const index = Math.ceil((p / 100) * sorted.length) - 1;
      return sorted[Math.max(0, Math.min(index, sorted.length - 1))];
    };

    return {
      sum,
      count,
      min,
      max,
      avg,
      p50: percentile(sorted, 50),
      p95: percentile(sorted, 95),
      p99: percentile(sorted, 99),
    };
  }

  /**
   * Aggregate counter metrics (sum)
   * @param {Array<MetricEntry>} metrics - Array of metric entries to aggregate
   * @returns {number} Sum of all values
   */
  static aggregateCounter(metrics) {
    if (!metrics || metrics.length === 0) return 0;
    return metrics.reduce((sum, m) => sum + Number(m.value || 0), 0);
  }

  /**
   * Aggregate gauge metrics (average or latest)
   * @param {Array<MetricEntry>} metrics - Array of metric entries to aggregate
   * @param {string} method - 'avg' or 'latest' (default: 'latest')
   * @returns {number} Aggregated value
   */
  static aggregateGauge(metrics, method = 'latest') {
    if (!metrics || metrics.length === 0) return 0;

    if (method === 'latest') {
      // Return the most recent value
      const sorted = [...metrics].sort((a, b) => b.timestamp - a.timestamp);
      return Number(sorted[0]?.value || 0);
    }

    // Average
    const sum = metrics.reduce((acc, m) => acc + Number(m.value || 0), 0);
    return sum / metrics.length;
  }

  /**
   * Aggregate summary metrics (similar to histogram)
   * @param {Array<MetricEntry>} metrics - Array of metric entries to aggregate
   * @returns {Object} Aggregated metrics with sum, count, min, max, avg
   */
  static aggregateSummary(metrics) {
    if (!metrics || metrics.length === 0) {
      return {
        sum: 0,
        count: 0,
        min: 0,
        max: 0,
        avg: 0,
      };
    }

    const values = metrics.map((m) => Number(m.value)).filter((v) => !isNaN(v));
    if (values.length === 0) {
      return {
        sum: 0,
        count: 0,
        min: 0,
        max: 0,
        avg: 0,
      };
    }

    const sorted = [...values].sort((a, b) => a - b);
    const sum = values.reduce((acc, v) => acc + v, 0);
    const count = values.length;

    return {
      sum,
      count,
      min: sorted[0],
      max: sorted[sorted.length - 1],
      avg: sum / count,
    };
  }

  /**
   * Aggregate metrics based on metric type
   * @param {Array<MetricEntry>} metrics - Array of metric entries to aggregate
   * @param {string} metricType - Type of metric (histogram, counter, gauge, summary)
   * @returns {Object|number} Aggregated result
   */
  static aggregate(metrics, metricType) {
    switch (metricType?.toLowerCase()) {
      case 'histogram':
        return this.aggregateHistogram(metrics);
      case 'counter':
        return this.aggregateCounter(metrics);
      case 'gauge':
        return this.aggregateGauge(metrics);
      case 'summary':
        return this.aggregateSummary(metrics);
      default:
        return this.aggregateGauge(metrics);
    }
  }
}
