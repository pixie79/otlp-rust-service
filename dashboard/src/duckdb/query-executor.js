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
  constructor(duckdbClient) {
    this.client = duckdbClient;
  }

  async execute(sql, params = []) {
    const start = now();
    try {
      const rows = await this.client.query(sql, params);
      return { rows, durationMs: now() - start };
    } catch (error) {
      throw new DuckDBError(sql, error);
    }
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
