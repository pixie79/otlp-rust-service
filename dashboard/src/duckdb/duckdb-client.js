import * as duckdb from '@duckdb/duckdb-wasm';
import { tableFromIPC } from 'apache-arrow';
import { DuckDBError } from '../error.js';

const sanitizeTableName = (fileName) =>
  fileName.replace(/[^a-z0-9_]/gi, '_').replace(/_{2,}/g, '_').replace(/^_+/, '').toLowerCase() ||
  'arrow_table';

export class DuckDBClient {
  constructor() {
    this.db = null;
    this.connection = null;
    this.worker = null;
    this.initialized = false;
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

  async registerArrowFile(fileName, arrowBuffer) {
    this._ensureReady();
    try {
      const tableName = sanitizeTableName(fileName);
      const table = tableFromIPC(arrowBuffer);
      await this.connection.insertArrowTable(table, {
        name: tableName,
        createTable: true,
        replace: true,
      });
      return tableName;
    } catch (error) {
      throw new DuckDBError(`registerArrowFile:${fileName}`, error);
    }
  }

  async query(sql, params = []) {
    this._ensureReady();
    try {
      const result = await this.connection.query(sql, params);
      return result?.toArray?.() ?? [];
    } catch (error) {
      throw new DuckDBError(sql, error);
    }
  }

  async unregisterTable(tableName) {
    this._ensureReady();
    try {
      const safeName = sanitizeTableName(tableName);
      await this.connection.run(`DROP TABLE IF EXISTS ${safeName};`);
    } catch (error) {
      throw new DuckDBError(`unregisterTable:${tableName}`, error);
    }
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
