import { describe, it, expect, beforeEach } from 'vitest';
import { DuckDBClient } from '../../src/duckdb/duckdb-client.js';
import { tableFromArrays, tableToIPC } from 'apache-arrow';

describe('Multiple File Handling', () => {
  let duckdbClient;

  beforeEach(async () => {
    duckdbClient = new DuckDBClient({ maxTables: 150 }); // Allow more tables for this test
    await duckdbClient.initialize();
  });

  const createTestArrowFile = (rowCount = 10) => {
    const data = {
      id: Array.from({ length: rowCount }, (_, i) => i),
      value: Array.from({ length: rowCount }, (_, i) => `value_${i}`),
    };
    // Create Arrow table from arrays, then convert to IPC format
    const table = tableFromArrays(data);
    return tableToIPC(table);
  };

  it('registers multiple files successfully', async () => {
    const fileCount = 50;
    const registeredTables = [];

    for (let i = 0; i < fileCount; i++) {
      const buffer = createTestArrowFile(100);
      const tableName = await duckdbClient.registerArrowFile(`file_${i}.arrow`, buffer);
      registeredTables.push(tableName);
    }

    expect(registeredTables.length).toBe(fileCount);

    const stats = duckdbClient.getTableStats();
    expect(stats.count).toBe(fileCount);
  });

  it('handles 100+ files with memory management', async () => {
    const fileCount = 120;
    const maxTables = 100;

    duckdbClient.maxTables = maxTables;

    // Register files up to limit
    for (let i = 0; i < fileCount; i++) {
      const buffer = createTestArrowFile(50);
      await duckdbClient.registerArrowFile(`file_${i}.arrow`, buffer);
    }

    const stats = duckdbClient.getTableStats();
    // Should not exceed maxTables
    expect(stats.count).toBeLessThanOrEqual(maxTables);
  });

  it('evicts old tables when limit is reached (LRU)', async () => {
    duckdbClient.maxTables = 5;

    // Register 5 tables
    const tableNames = [];
    for (let i = 0; i < 5; i++) {
      const buffer = createTestArrowFile(10);
      const tableName = await duckdbClient.registerArrowFile(`file_${i}.arrow`, buffer);
      tableNames.push(tableName);
    }

    expect(duckdbClient.getTableStats().count).toBe(5);

    // Register one more - should evict oldest
    const buffer = createTestArrowFile(10);
    await duckdbClient.registerArrowFile('file_5.arrow', buffer);

    const stats = duckdbClient.getTableStats();
    expect(stats.count).toBe(5);
    // Oldest table should be evicted
    expect(stats.tables.find((t) => t.name === tableNames[0])).toBeUndefined();
  });

  it('tracks table access order for LRU eviction', async () => {
    duckdbClient.maxTables = 3;

    // Register 3 tables
    const tableNames = [];
    for (let i = 0; i < 3; i++) {
      const buffer = createTestArrowFile(10);
      const tableName = await duckdbClient.registerArrowFile(`file_${i}.arrow`, buffer);
      tableNames.push(tableName);
    }

    // Query the first table (should update access order)
    await duckdbClient.query(`SELECT * FROM ${tableNames[0]} LIMIT 1`);

    // Register new table - should evict least recently used (not the one we just queried)
    const buffer = createTestArrowFile(10);
    await duckdbClient.registerArrowFile('file_3.arrow', buffer);

    const stats = duckdbClient.getTableStats();
    // The table we queried should still be there
    expect(stats.tables.find((t) => t.name === tableNames[0])).toBeDefined();
  });

  it('can unregister old tables by age', async () => {
    // Register tables
    for (let i = 0; i < 10; i++) {
      const buffer = createTestArrowFile(10);
      await duckdbClient.registerArrowFile(`file_${i}.arrow`, buffer);
    }

    // Wait a bit (simulate time passing)
    await new Promise((resolve) => setTimeout(resolve, 10));

    // Unregister tables older than 5ms
    const removed = await duckdbClient.unregisterOldTables(5);

    expect(removed).toBeGreaterThan(0);
    const stats = duckdbClient.getTableStats();
    expect(stats.count).toBeLessThan(10);
  });

  it('maintains table statistics', async () => {
    const buffer = createTestArrowFile(100);
    const tableName = await duckdbClient.registerArrowFile('stats_test.arrow', buffer);

    const stats = duckdbClient.getTableStats();
    const tableInfo = stats.tables.find((t) => t.name === tableName);

    expect(tableInfo).toBeDefined();
    expect(tableInfo.fileName).toBe('stats_test.arrow');
    expect(tableInfo.registeredAt).toBeGreaterThan(0);
    expect(tableInfo.lastAccessed).toBeGreaterThan(0);
    expect(tableInfo.rowCount).toBe(100);
  });

  it('handles replacing existing tables', async () => {
    const buffer1 = createTestArrowFile(10);
    const tableName = await duckdbClient.registerArrowFile('replace_test.arrow', buffer1);

    const stats1 = duckdbClient.getTableStats();
    expect(stats1.count).toBe(1);

    // Replace with new data
    const buffer2 = createTestArrowFile(20);
    await duckdbClient.registerArrowFile('replace_test.arrow', buffer2);

    const stats2 = duckdbClient.getTableStats();
    expect(stats2.count).toBe(1); // Still one table
    const tableInfo = stats2.tables.find((t) => t.name === tableName);
    expect(tableInfo.rowCount).toBe(20); // Updated row count
  });
});
