import * as duckdb from '@duckdb/duckdb-wasm';
import { tableFromIPC } from 'apache-arrow';
import { DuckDBError } from '../error.js';

const sanitizeTableName = (fileName) =>
  fileName
    .replace(/[^a-z0-9_]/gi, '_')
    .replace(/_{2,}/g, '_')
    .replace(/^_+/, '')
    .toLowerCase() || 'arrow_table';

export class DuckDBClient {
  constructor(options = {}) {
    this.db = null;
    this.connection = null;
    this.worker = null;
    this.initialized = false;
    this.registeredTables = new Map(); // Track registered tables with metadata
    this.maxTables = options.maxTables || 100; // Maximum number of tables to keep in memory
    this.tableAccessOrder = []; // Track table access order for LRU eviction
  }

  async initialize() {
    if (this.initialized) {
      return;
    }

    try {
      const bundles = duckdb.getJsDelivrBundles();
      const bundle = await duckdb.selectBundle(bundles);
      this.worker = await duckdb.createWorker(bundle.mainWorker);
      const logger = new duckdb.ConsoleLogger();
      this.db = new duckdb.AsyncDuckDB(logger, this.worker);
      await this.db.instantiate(bundle.mainModule);
      this.connection = await this.db.connect();
      this.initialized = true;
    } catch (error) {
      throw new DuckDBError('initialize', error);
    }
  }

  /**
   * Register Arrow IPC file with optimization for large files
   * Implements memory management by unregistering old tables when limit is reached
   */
  async registerArrowFile(fileName, arrowBuffer) {
    this._ensureReady();
    try {
      const tableName = sanitizeTableName(fileName);

      // Check if we need to evict old tables
      if (this.registeredTables.size >= this.maxTables && !this.registeredTables.has(tableName)) {
        await this._evictOldestTable();
      }

      // Parse Arrow table (this is necessary for DuckDB)
      // For very large files, this might take time, but Arrow IPC format requires full parsing
      const table = tableFromIPC(arrowBuffer);

      // Register table in DuckDB
      await this.connection.insertArrowTable(table, {
        name: tableName,
        createTable: true,
        replace: true,
      });

      // Track registered table
      this.registeredTables.set(tableName, {
        fileName,
        registeredAt: Date.now(),
        lastAccessed: Date.now(),
        rowCount: table.numRows || 0,
      });

      // Update access order
      this._updateAccessOrder(tableName);

      return tableName;
    } catch (error) {
      throw new DuckDBError(`registerArrowFile:${fileName}`, error);
    }
  }

  /**
   * Evict oldest table (LRU eviction)
   * @private
   */
  async _evictOldestTable() {
    if (this.tableAccessOrder.length === 0) {
      return;
    }

    // Remove oldest accessed table
    const oldestTable = this.tableAccessOrder.shift();
    if (oldestTable && this.registeredTables.has(oldestTable)) {
      await this.unregisterTable(oldestTable);
    }
  }

  /**
   * Update access order for LRU tracking
   * @private
   */
  _updateAccessOrder(tableName) {
    // Remove from current position if exists
    const index = this.tableAccessOrder.indexOf(tableName);
    if (index > -1) {
      this.tableAccessOrder.splice(index, 1);
    }
    // Add to end (most recently used)
    this.tableAccessOrder.push(tableName);

    // Update last accessed time
    const tableInfo = this.registeredTables.get(tableName);
    if (tableInfo) {
      tableInfo.lastAccessed = Date.now();
    }
  }

  async query(sql, params = []) {
    this._ensureReady();
    try {
      const result = await this.connection.query(sql, params);
      const rows = result?.toArray?.() ?? [];

      // Update access order for tables referenced in query
      // Extract table names from SQL (simple heuristic)
      const tableMatches = sql.match(/FROM\s+(\w+)/gi);
      if (tableMatches) {
        for (const match of tableMatches) {
          const tableName = match.replace(/FROM\s+/i, '').trim();
          if (this.registeredTables.has(tableName)) {
            this._updateAccessOrder(tableName);
          }
        }
      }

      return rows;
    } catch (error) {
      throw new DuckDBError(sql, error);
    }
  }

  async unregisterTable(tableName) {
    this._ensureReady();
    try {
      const safeName = sanitizeTableName(tableName);
      await this.connection.run(`DROP TABLE IF EXISTS ${safeName};`);

      // Remove from tracking
      this.registeredTables.delete(safeName);
      const index = this.tableAccessOrder.indexOf(safeName);
      if (index > -1) {
        this.tableAccessOrder.splice(index, 1);
      }
    } catch (error) {
      throw new DuckDBError(`unregisterTable:${tableName}`, error);
    }
  }

  /**
   * Get statistics about registered tables
   */
  getTableStats() {
    return {
      count: this.registeredTables.size,
      maxTables: this.maxTables,
      tables: Array.from(this.registeredTables.entries()).map(([name, info]) => ({
        name,
        ...info,
      })),
    };
  }

  /**
   * Unregister tables older than specified age (in milliseconds)
   */
  async unregisterOldTables(maxAgeMs) {
    const now = Date.now();
    const tablesToRemove = [];

    for (const [tableName, info] of this.registeredTables.entries()) {
      if (now - info.registeredAt > maxAgeMs) {
        tablesToRemove.push(tableName);
      }
    }

    for (const tableName of tablesToRemove) {
      await this.unregisterTable(tableName);
    }

    return tablesToRemove.length;
  }

  async close() {
    if (!this.initialized) {
      return;
    }

    await this.connection?.close();
    await this.db?.close();
    await this.worker?.terminate();

    this.connection = null;
    this.db = null;
    this.worker = null;
    this.initialized = false;
  }

  _ensureReady() {
    if (!this.initialized || !this.connection) {
      throw new Error('DuckDBClient is not initialized');
    }
  }
}
