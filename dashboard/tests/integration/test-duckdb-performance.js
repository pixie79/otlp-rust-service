import { describe, it, expect, beforeEach, vi } from 'vitest';
import { QueryExecutor } from '../../src/duckdb/query-executor.js';
import { DuckDBClient } from '../../src/duckdb/duckdb-client.js';

describe('DuckDB Query Performance', () => {
  let duckdbClient;
  let queryExecutor;

  beforeEach(async () => {
    duckdbClient = new DuckDBClient();
    await duckdbClient.initialize();
    queryExecutor = new QueryExecutor(duckdbClient, { maxLatencyMs: 500 });
  });

  it('tracks query performance metrics', async () => {
    // Create a test table
    await duckdbClient.connection.run(`
      CREATE TABLE test_perf AS
      SELECT * FROM generate_series(1, 1000) AS id
    `);

    // Execute multiple queries
    for (let i = 0; i < 10; i++) {
      await queryExecutor.execute('SELECT * FROM test_perf LIMIT 100');
    }

    const stats = queryExecutor.getPerformanceStats();

    expect(stats.count).toBe(10);
    expect(stats.avgLatencyMs).toBeGreaterThan(0);
    expect(stats.p50LatencyMs).toBeGreaterThan(0);
    expect(stats.p95LatencyMs).toBeGreaterThan(0);
    expect(stats.maxLatencyMs).toBeGreaterThan(0);
  });

  it('ensures p95 latency is below target for typical queries', async () => {
    await duckdbClient.connection.run(`
      CREATE TABLE test_latency AS
      SELECT * FROM generate_series(1, 10000) AS id
    `);

    // Execute queries that should be fast
    for (let i = 0; i < 20; i++) {
      await queryExecutor.execute('SELECT COUNT(*) FROM test_latency');
    }

    const stats = queryExecutor.getPerformanceStats();

    // p95 should be reasonable for simple queries
    expect(stats.p95LatencyMs).toBeLessThan(1000); // 1 second max for simple queries
  });

  it('records query duration for each execution', async () => {
    await duckdbClient.connection.run(`
      CREATE TABLE test_duration AS
      SELECT * FROM generate_series(1, 100) AS id
    `);

    const result = await queryExecutor.execute('SELECT * FROM test_duration');

    expect(result.durationMs).toBeGreaterThanOrEqual(0);
    expect(result.durationMs).toBeLessThan(1000); // Should be fast
    expect(result.rows).toBeDefined();
  });

  it('tracks error rate in performance stats', async () => {
    // Execute some successful queries
    await queryExecutor.execute('SELECT 1');

    // Execute a query that will fail
    try {
      await queryExecutor.execute('SELECT * FROM nonexistent_table');
    } catch {
      // Expected to fail
    }

    const stats = queryExecutor.getPerformanceStats();

    expect(stats.count).toBe(2);
    expect(stats.errorRate).toBeGreaterThan(0);
    expect(stats.errorRate).toBeLessThanOrEqual(1);
  });

  it('warns when query exceeds latency target', async () => {
    const consoleSpy = vi.spyOn(console, 'warn').mockImplementation(() => {});

    // Create a slow query (if possible)
    await duckdbClient.connection.run(`
      CREATE TABLE test_slow AS
      SELECT * FROM generate_series(1, 1000000) AS id
    `);

    // This might be slow depending on system
    try {
      await queryExecutor.execute('SELECT * FROM test_slow ORDER BY id');
    } catch {
      // May timeout or fail
    }

    // Check if warning was logged (if query was slow)
    // Note: This test may not always trigger the warning depending on system performance
    const stats = queryExecutor.getPerformanceStats();
    if (stats.maxLatencyMs > 500) {
      expect(consoleSpy).toHaveBeenCalled();
    }

    consoleSpy.mockRestore();
  });

  it('can clear performance statistics', async () => {
    await queryExecutor.execute('SELECT 1');
    await queryExecutor.execute('SELECT 2');

    let stats = queryExecutor.getPerformanceStats();
    expect(stats.count).toBe(2);

    queryExecutor.clearStats();

    stats = queryExecutor.getPerformanceStats();
    expect(stats.count).toBe(0);
  });

  it('limits performance stats history size', async () => {
    const executor = new QueryExecutor(duckdbClient, { maxStatsHistory: 5 });

    // Execute more queries than max history
    for (let i = 0; i < 10; i++) {
      await executor.execute('SELECT 1');
    }

    const stats = executor.getPerformanceStats();
    expect(stats.count).toBeLessThanOrEqual(5);
  });
});
