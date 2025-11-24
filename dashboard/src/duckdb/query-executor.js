import { DuckDBError } from '../error.js';

const DEFAULT_TRACE_QUERY = `
  SELECT *
  FROM {tableName}
  ORDER BY start_time_unix_nano DESC
  LIMIT ?
`;

const DEFAULT_METRIC_QUERY = `
  SELECT *
  FROM {tableName}
  WHERE metric_name = ?
  ORDER BY timestamp_unix_nano DESC
  LIMIT ?
`;

const now = () => {
  if (typeof performance !== 'undefined' && performance.now) {
    return performance.now();
  }
  return Date.now();
};

export class QueryExecutor {
  constructor(duckdbClient, options = {}) {
    this.client = duckdbClient;
    this.maxLatencyMs = options.maxLatencyMs || 500; // p95 latency target
    this.queryStats = []; // Track query performance
    this.maxStatsHistory = options.maxStatsHistory || 100;
  }

  async execute(sql, params = []) {
    const start = now();
    try {
      const result = await this.client.query(sql, params);
      let rows = [];
      if (Array.isArray(result)) {
        rows = result;
      } else if (result?.toArray) {
        rows = result.toArray();
      } else if (result?.rows) {
        rows = result.rows;
      }
      
      const durationMs = now() - start;
      
      // Track performance metrics
      this._recordQueryStats(sql, durationMs, rows.length);
      
      // Warn if query exceeds latency target
      if (durationMs > this.maxLatencyMs) {
        console.warn(`Query exceeded latency target (${durationMs.toFixed(2)}ms > ${this.maxLatencyMs}ms): ${sql.substring(0, 100)}`);
      }

      return { rows, durationMs };
    } catch (error) {
      const durationMs = now() - start;
      this._recordQueryStats(sql, durationMs, 0, error);
      throw new DuckDBError(sql, error);
    }
  }

  /**
   * Record query statistics for performance monitoring
   * @private
   */
  _recordQueryStats(sql, durationMs, rowCount, error = null) {
    this.queryStats.push({
      sql: sql.substring(0, 200), // Truncate for memory efficiency
      durationMs,
      rowCount,
      timestamp: Date.now(),
      error: error?.message || null,
    });

    // Keep only recent stats
    if (this.queryStats.length > this.maxStatsHistory) {
      this.queryStats.shift();
    }
  }

  /**
   * Get performance statistics
   */
  getPerformanceStats() {
    if (this.queryStats.length === 0) {
      return {
        count: 0,
        avgLatencyMs: 0,
        p50LatencyMs: 0,
        p95LatencyMs: 0,
        p99LatencyMs: 0,
        maxLatencyMs: 0,
        errorRate: 0,
      };
    }

    const latencies = this.queryStats.map((s) => s.durationMs).sort((a, b) => a - b);
    const errors = this.queryStats.filter((s) => s.error).length;

    const percentile = (sorted, p) => {
      if (sorted.length === 0) return 0;
      const index = Math.ceil((p / 100) * sorted.length) - 1;
      return sorted[Math.max(0, Math.min(index, sorted.length - 1))];
    };

    return {
      count: this.queryStats.length,
      avgLatencyMs: latencies.reduce((sum, l) => sum + l, 0) / latencies.length,
      p50LatencyMs: percentile(latencies, 50),
      p95LatencyMs: percentile(latencies, 95),
      p99LatencyMs: percentile(latencies, 99),
      maxLatencyMs: latencies[latencies.length - 1],
      errorRate: errors / this.queryStats.length,
    };
  }

  /**
   * Clear performance statistics
   */
  clearStats() {
    this.queryStats = [];
  }

  async fetchTraces(tableName, limit) {
    const sql = DEFAULT_TRACE_QUERY.replace('{tableName}', tableName);
    return this.execute(sql, [limit]);
  }

  async fetchMetric(tableName, metricName, limit) {
    const sql = DEFAULT_METRIC_QUERY.replace('{tableName}', tableName);
    return this.execute(sql, [metricName, limit]);
  }
}
