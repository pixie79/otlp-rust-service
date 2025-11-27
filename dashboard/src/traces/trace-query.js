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
    // status_code is Int32: 0=Unset, 1=Ok, 2=Error
    clauses.push('status_code = 2');
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

    // Filter to only traces tables (tables starting with otlp_traces_)
    // Metrics tables don't have start_time_unix_nano column
    const tracesTables = tables.filter(tableName => 
      tableName.startsWith('otlp_traces_') || tableName.includes('traces')
    );

    if (!tracesTables.length) {
      return [];
    }

    const { sql: whereSql, params } = buildPredicate(filters);
    const limit = filters.limit ?? this.limit;

    const queries = tracesTables.map((tableName) => {
      const sql = `
        SELECT * FROM ${tableName}
        ${whereSql}
        ORDER BY start_time_unix_nano DESC
        LIMIT ${limit}
      `;
      return this.executor.execute(sql, [...params]).catch((error) => {
        // Handle missing tables gracefully (e.g., evicted tables, not yet created)
        // Check if it's a "table does not exist" error
        const isTableNotFound = error.message && (
          error.message.includes('does not exist') ||
          error.message.includes('Catalog Error')
        );
        
        if (isTableNotFound) {
          // Table doesn't exist - likely not ingested yet or was evicted
          // Return empty result and mark for cleanup
          console.warn(`Query failed for table ${tableName}: Table does not exist (may not be ingested yet or was evicted)`);
          return { rows: [], error: error.message, tableName, isTableNotFound: true };
        } else {
          // Other error - log but don't mark for cleanup
          console.warn(`Query failed for table ${tableName}:`, error.message);
          return { rows: [], error: error.message, tableName, isTableNotFound: false };
        }
      });
    });

    // Use allSettled to handle individual query failures gracefully
    const results = await Promise.allSettled(queries);
    const failedTables = [];
    const rows = results
      .filter((result) => {
        if (result.status === 'fulfilled') {
          const value = result.value;
          if (value.error) {
            // Only track tables that don't exist (not other errors)
            // This allows us to clean up stale state
            if (value.tableName && value.isTableNotFound) {
              failedTables.push(value.tableName);
            }
            return false;
          }
          return true;
        }
        return false;
      })
      .map((result) => result.value)
      .flatMap((result) => result.rows ?? []);
    
    // Return failed table names so caller can clean up state
    // Only clean up tables that actually don't exist (not other errors)
    if (failedTables.length > 0 && this.onTableMissing) {
      this.onTableMissing(failedTables);
    }
    
    const traces = rows.map(createTraceEntry).sort(sortByStartTimeDesc);
    return traces.slice(0, limit);
  }
}
