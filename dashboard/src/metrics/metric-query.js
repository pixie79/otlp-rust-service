import { createMetricEntry } from './metric-entry.js';
import { MetricAggregator } from './metric-aggregator.js';

/**
 * MetricQuery component for querying metrics using DuckDB SQL
 * Implements metric data querying with filtering and aggregation per FR-008
 */
export class MetricQuery {
  constructor(options = {}) {
    this.execute = options.execute || (async () => ({ rows: [] }));
    this.limit = options.limit ?? 10000;
  }

  /**
   * Query metrics from multiple tables with filters
   * @param {Array<string>} tables - Array of table names to query
   * @param {Object} filters - Filter object with metricName, labels, timeRange
   * @returns {Promise<Array>} Array of MetricEntry objects
   */
  async queryMetricsFromTables(tables = [], filters = {}) {
    if (!tables.length) {
      return [];
    }

    const { sql: whereSql, params } = this._buildPredicate(filters);
    const limit = filters.limit ?? this.limit;

    // Query each table and combine results
    const queries = tables.map((tableName) => {
      const sql = `
        SELECT * FROM ${tableName}
        ${whereSql}
        ORDER BY timestamp_unix_nano ASC
        LIMIT ${limit}
      `;
      return this.execute(sql, [...params]);
    });

    try {
      const results = await Promise.all(queries);
      const rows = results.flatMap((result) => result.rows ?? []);
      return rows.map((row) => createMetricEntry(row));
    } catch (error) {
      console.error('Metric query failed:', error);
      throw error;
    }
  }

  /**
   * Query metrics with filters (single table - for backward compatibility)
   * @param {Object} filters - Filter object with metricName, labels, timeRange
   * @returns {Promise<Array>} Array of MetricEntry objects
   */
  async queryMetrics(filters = {}) {
    // This method is kept for compatibility but should use queryMetricsFromTables
    const sql = this._buildQuery(filters);
    const params = this._buildParams(filters);

    try {
      const result = await this.execute(sql, params);
      return (result.rows || []).map((row) => createMetricEntry(row));
    } catch (error) {
      console.error('Metric query failed:', error);
      throw error;
    }
  }

  /**
   * Query aggregated metrics (for histogram/summary types)
   * @param {string} metricName - Name of the metric
   * @param {string} metricType - Type of metric (histogram, summary, etc.)
   * @param {Object} filters - Filter object with labels, timeRange
   * @returns {Promise<Object>} Aggregated metrics
   */
  async queryAggregatedMetrics(metricName, metricType, filters = {}) {
    const metrics = await this.queryMetrics({ ...filters, metricName });
    return MetricAggregator.aggregate(metrics, metricType);
  }

  /**
   * Get available metric names from multiple tables
   * @param {Array<string>} tables - Array of table names to query
   * @returns {Promise<Array<string>>} Array of metric names
   */
  async getAvailableMetricsFromTables(tables = []) {
    if (!tables.length) {
      return [];
    }

    // Query each table for distinct metric names
    const queries = tables.map((tableName) => {
      const sql = `
        SELECT DISTINCT metric_name 
        FROM ${tableName}
        WHERE metric_name IS NOT NULL AND metric_name != ''
      `;
      return this.execute(sql, []);
    });

    try {
      const results = await Promise.all(queries);
      const allNames = new Set();
      for (const result of results) {
        for (const row of result.rows || []) {
          const name = row.metric_name || row.metricName;
          if (name) {
            allNames.add(name);
          }
        }
      }
      return Array.from(allNames).sort();
    } catch (error) {
      console.error('Failed to get available metrics:', error);
      return [];
    }
  }

  /**
   * Get available metric names (single table - for backward compatibility)
   * @returns {Promise<Array<string>>} Array of metric names
   */
  async getAvailableMetrics() {
    const sql = 'SELECT DISTINCT metric_name FROM metrics WHERE metric_name IS NOT NULL ORDER BY metric_name';
    try {
      const result = await this.execute(sql, []);
      return (result.rows || []).map((row) => row.metric_name || row.metricName).filter(Boolean);
    } catch (error) {
      console.error('Failed to get available metrics:', error);
      return [];
    }
  }

  /**
   * Build WHERE predicate from filters
   * @private
   */
  _buildPredicate(filters) {
    const clauses = [];
    const params = [];

    // Check if this table has metrics (has metric_name column)
    // We'll filter by checking if metric_name column exists and is not null
    clauses.push("metric_name IS NOT NULL AND metric_name != ''");

    if (filters.metricName) {
      clauses.push('metric_name = ?');
      params.push(filters.metricName);
    }

    if (filters.labels && Object.keys(filters.labels).length > 0) {
      // For label filtering, parse JSON attributes
      for (const [key, value] of Object.entries(filters.labels)) {
        clauses.push('attributes LIKE ?');
        params.push(`%"${key}":"${value}"%`);
      }
    }

    if (filters.timeRange?.start) {
      clauses.push('timestamp_unix_nano >= ?');
      params.push(filters.timeRange.start);
    }

    if (filters.timeRange?.end) {
      clauses.push('timestamp_unix_nano <= ?');
      params.push(filters.timeRange.end);
    }

    return {
      sql: clauses.length ? `WHERE ${clauses.join(' AND ')}` : '',
      params,
    };
  }

  /**
   * Build SQL query from filters (for single table queries)
   * @private
   */
  _buildQuery(filters) {
    const { sql: whereSql, params } = this._buildPredicate(filters);
    return {
      sql: `SELECT * FROM metrics ${whereSql} ORDER BY timestamp_unix_nano ASC`,
      params,
    };
  }

  /**
   * Build parameters array from filters (for single table queries)
   * @private
   */
  _buildParams(filters) {
    const { params } = this._buildPredicate(filters);
    return params;
  }
}

