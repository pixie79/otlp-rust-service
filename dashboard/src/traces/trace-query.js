import { createTraceEntry, sortByStartTimeDesc } from './trace-entry.js';

const buildPredicate = (filters = {}) => {
  const clauses = [];
  const params = [];

  if (filters.traceId) {
    clauses.push('trace_id LIKE ?');
    params.push(`${filters.traceId}%`);
  }
  if (filters.serviceName) {
    clauses.push('LOWER(service_name) LIKE ?');
    params.push(`%${filters.serviceName.toLowerCase()}%`);
  }
  if (filters.spanName) {
    clauses.push('LOWER(name) LIKE ?');
    params.push(`%${filters.spanName.toLowerCase()}%`);
  }
  if (filters.errorOnly) {
    clauses.push("LOWER(status_code) = 'error'");
  }
  if (filters.minDuration != null) {
    clauses.push('((end_time_unix_nano - start_time_unix_nano) / 1000000) >= ?');
    params.push(filters.minDuration);
  }
  if (filters.maxDuration != null) {
    clauses.push('((end_time_unix_nano - start_time_unix_nano) / 1000000) <= ?');
    params.push(filters.maxDuration);
  }

  return {
    sql: clauses.length ? `WHERE ${clauses.join(' AND ')}` : '',
    params,
  };
};

export class TraceQuery {
  constructor(queryExecutor, options = {}) {
    this.executor = queryExecutor;
    this.limit = options.limit ?? 1_000;
  }

  async fetchLatestFromTables(tables = [], filters = {}) {
    if (!tables.length) {
      return [];
    }
    const { sql: whereSql, params } = buildPredicate(filters);
    const limit = filters.limit ?? this.limit;

    const queries = tables.map((tableName) => {
      const sql = `
        SELECT * FROM ${tableName}
        ${whereSql}
        ORDER BY start_time_unix_nano DESC
        LIMIT ${limit}
      `;
      return this.executor.execute(sql, [...params]);
    });

    const results = await Promise.all(queries);
    const rows = results.flatMap((result) => result.rows ?? []);
    const traces = rows.map(createTraceEntry).sort(sortByStartTimeDesc);
    return traces.slice(0, limit);
  }
}
