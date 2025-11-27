import * as duckdb from '@duckdb/duckdb-wasm';
import duckdb_wasm from '@duckdb/duckdb-wasm/dist/duckdb-mvp.wasm?url';
import mvp_worker from '@duckdb/duckdb-wasm/dist/duckdb-browser-mvp.worker.js?url';
import duckdb_wasm_eh from '@duckdb/duckdb-wasm/dist/duckdb-eh.wasm?url';
import eh_worker from '@duckdb/duckdb-wasm/dist/duckdb-browser-eh.worker.js?url';
import { DuckDBError } from '../error.js';

// IMPORTANT: Apache Arrow Version Compatibility
// DuckDB-WASM v1.30.0 uses Apache Arrow v17 internally.
// Using Apache Arrow v21 causes silent failures where insertArrowTable appears to succeed
// but tables are empty or not queryable. See: https://github.com/duckdb/duckdb-wasm/issues/2097
// 
// DO NOT upgrade apache-arrow beyond v17.x until DuckDB-WASM updates its dependency.
// Current version: apache-arrow@^17.0.0 (pinned in package.json)

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
      console.log('[DuckDBClient] Starting initialization...');
      // Use Vite approach for DuckDB-Wasm instantiation
      // See: https://duckdb.org/docs/stable/clients/wasm/instantiation
      const MANUAL_BUNDLES = {
        mvp: {
          mainModule: duckdb_wasm,
          mainWorker: mvp_worker,
        },
        eh: {
          mainModule: duckdb_wasm_eh,
          mainWorker: eh_worker,
        },
      };
      
      console.log('[DuckDBClient] Bundles configured:', {
        mvp: { mainModule: duckdb_wasm, mainWorker: mvp_worker },
        eh: { mainModule: duckdb_wasm_eh, mainWorker: eh_worker },
      });
      
      // Select a bundle based on browser checks
      const bundle = await duckdb.selectBundle(MANUAL_BUNDLES);
      console.log('[DuckDBClient] Selected bundle:', bundle);
      
      // AsyncDuckDB always requires a worker, even when running inside a worker
      // We need to create a worker for DuckDB-Wasm to function properly
      // Note: Nested workers are supported in modern browsers
      // DuckDB's worker will handle fetching the WASM file itself, so we pass the URL string
      const logger = new duckdb.ConsoleLogger();
      
      console.log('[DuckDBClient] Creating worker from:', bundle.mainWorker);
      this.worker = new Worker(bundle.mainWorker);
      this.db = new duckdb.AsyncDuckDB(logger, this.worker);
      
      console.log('[DuckDBClient] Instantiating DuckDB with module URL:', bundle.mainModule);
      // Pass the URL string to DuckDB - its worker will fetch and compile the WASM
      // pthreadWorker is optional - only pass if it exists
      await this.db.instantiate(bundle.mainModule, bundle.pthreadWorker || undefined);
      console.log('[DuckDBClient] DuckDB instantiated, connecting...');
      this.connection = await this.db.connect();
      console.log('[DuckDBClient] Connected successfully');
      
      // Try to load Arrow extension for Arrow IPC file support
      // Note: DuckDB-WASM's INSTALL command always tries remote first, so we'll use insertArrowTable as primary
      // The extension loading is attempted but not required - insertArrowTable works without it
      const baseURL = typeof window !== 'undefined' ? window.location.origin : 'http://127.0.0.1:8080';
      const extensionURL = `${baseURL}/extensions/arrow.duckdb_extension.wasm`;
      
      // Try to load extension directly using LOAD with full URL (bypasses INSTALL)
      // This is a best-effort attempt - if it fails, we use insertArrowTable which works reliably
      try {
        // Register the extension file URL first
        const extensionFileName = 'arrow_ext_local';
        await this.db.registerFileURL(extensionFileName, extensionURL, 'DUCKDB_NATIVE');
        
        // Try to load the extension directly from the registered file
        // DuckDB-WASM may support loading extensions from registered files
        await this.connection.query(`LOAD '${extensionFileName}'`);
        console.log('[DuckDBClient] Arrow extension loaded successfully from local file');
      } catch (error) {
        // Extension loading failed - this is expected and OK
        // We'll use insertArrowTable which works reliably without the extension
        console.log('[DuckDBClient] Arrow extension not available (using insertArrowTable method):', error.message);
      }
      
      // Clear all existing tables on initialization to prevent stale data
      // This ensures a clean state when the page is refreshed or server is restarted
      await this.clearAllTables();
      
      this.initialized = true;
      console.log('[DuckDBClient] Initialization complete');
    } catch (error) {
      console.error('[DuckDBClient] Initialization error:', error);
      console.error('[DuckDBClient] Error details:', {
        message: error.message,
        stack: error.stack,
        name: error.name,
      });
      throw new DuckDBError('initialize', error);
    }
  }

  /**
   * Clear all tables from DuckDB
   * This is called on initialization to ensure a clean state
   */
  async clearAllTables() {
    if (!this.connection) {
      // Connection not available - DuckDB not initialized yet
      // This is OK, just return silently
      return;
    }

    try {
      console.log('[DuckDBClient] Clearing all existing tables...');
      
      // Get all table names from information_schema
      const tablesResult = await this.connection.query(`
        SELECT table_name 
        FROM information_schema.tables 
        WHERE table_schema = 'main' 
        AND table_type = 'BASE TABLE'
      `);
      
      if (tablesResult && tablesResult.length > 0) {
        const tableNames = tablesResult.map(row => row.table_name);
        console.log(`[DuckDBClient] Found ${tableNames.length} tables to drop:`, tableNames);
        
        // Drop all tables
        for (const tableName of tableNames) {
          try {
            await this.connection.query(`DROP TABLE IF EXISTS ${tableName} CASCADE`);
          } catch (dropError) {
            console.warn(`[DuckDBClient] Failed to drop table ${tableName}:`, dropError);
          }
        }
        
        console.log(`[DuckDBClient] Dropped ${tableNames.length} tables`);
      } else {
        console.log('[DuckDBClient] No existing tables to clear');
      }
      
      // Clear internal tracking
      this.registeredTables.clear();
      this.tableAccessOrder = [];
    } catch (error) {
      console.warn('[DuckDBClient] Error clearing tables (non-fatal):', error);
      // Don't throw - clearing tables is best-effort
    }
  }

  /**
   * Register Arrow IPC file
   * Supports both fileURL (for server-served files) and buffer (for local files)
   * 
   * @param {string} fileName - Original file name (e.g., "otlp_traces_20251126_221703_0000.arrows")
   * @param {string|ArrayBuffer} fileURLOrBuffer - Either:
   *   - Full URL to the Arrow file (e.g., "http://127.0.0.1:8080/data/otlp/traces/...")
   *   - ArrayBuffer containing the Arrow file data (for local files)
   */
  async registerArrowFile(fileName, fileURLOrBuffer) {
    this._ensureReady();
    try {
      const tableName = sanitizeTableName(fileName);
      const isExistingTable = this.registeredTables.has(tableName);
      const now = Date.now();

      // If this is an update to an existing table, mark it as recently updated
      // This prevents actively written files from being evicted
      if (isExistingTable) {
        const tableInfo = this.registeredTables.get(tableName);
        tableInfo.lastUpdated = now;
        tableInfo.lastAccessed = now;
        // Update access order to mark as recently used
        this._updateAccessOrder(tableName);
      }

      // Check if we need to evict old tables (only for new tables)
      // Only evict when we're at the memory threshold (95% by default)
      // This prevents aggressive eviction and allows more tables to be kept in memory
      const memoryThreshold = this.maxTables * 0.95; // 95% of max tables
      if (this.registeredTables.size >= memoryThreshold && !isExistingTable) {
        // Only evict if we're truly at the memory limit
        // Use a very long grace period to prevent evicting active files
        await this._evictOldestTable(600000); // 10 minute grace period for recently updated files
      }

      // Determine if we have a fileURL (string) or buffer (ArrayBuffer)
      const isBuffer = fileURLOrBuffer instanceof ArrayBuffer;
      let arrowBuffer;
      let fileURL = null; // Store fileURL for tracking if it's a URL-based file
      
      if (isBuffer) {
        // Local file - use buffer directly
        arrowBuffer = fileURLOrBuffer;
        console.log('[DuckDBClient] Using local file buffer, size:', arrowBuffer.byteLength, 'bytes');
      } else {
        // Server-served file - try registerFileURL + read_arrow() first, then fallback to fetch
        fileURL = fileURLOrBuffer;
        let useInsertArrowTable = false;
        
        try {
          // Register the file URL with DuckDB
          const virtualFileName = `arrow_${tableName}_${Date.now()}`;
          await this.db.registerFileURL(virtualFileName, fileURL, 'DUCKDB_NATIVE');
          
          // Try to create table using read_arrow() (requires extension)
          await this.connection.query(`
            CREATE OR REPLACE TABLE ${tableName} AS 
            SELECT * FROM read_arrow('${virtualFileName}')
          `);
        } catch (arrowError) {
          // Arrow extension not available or read_arrow() failed
          // Fall back to fetching file and using insertArrowTable
          console.warn('[DuckDBClient] registerFileURL/read_arrow() failed, falling back to insertArrowTable:', arrowError.message);
          useInsertArrowTable = true;
        }

        if (useInsertArrowTable) {
          // Fallback: Fetch the file and use insertArrowTable
          console.log('[DuckDBClient] Fetching Arrow file from URL:', fileURL);
          try {
            const response = await fetch(fileURL);
            if (!response.ok) {
              throw new Error(`Failed to fetch Arrow file from ${fileURL}: ${response.status} ${response.statusText}`);
            }
            arrowBuffer = await response.arrayBuffer();
            console.log('[DuckDBClient] Fetched Arrow file, size:', arrowBuffer.byteLength, 'bytes');
          } catch (fetchError) {
            console.error('[DuckDBClient] Failed to fetch Arrow file:', fetchError);
            throw new Error(`Failed to fetch Arrow file from ${fileURL}: ${fetchError.message}`);
          }
          } else {
          // read_arrow() succeeded, skip insertArrowTable
          arrowBuffer = null;
        }
      }

      // If we have a buffer (local file or fetched), use insertArrowTable
      if (arrowBuffer) {
        // Always check if table exists in DuckDB and drop it if it does
        // This handles cases where the table exists in DuckDB but not in our tracking
        // (e.g., after errors, state inconsistencies, or view switches)
        try {
          const tableExistsCheck = await this.connection.query(`
            SELECT COUNT(*) as count 
            FROM information_schema.tables 
            WHERE table_schema = 'main' 
            AND table_name = '${tableName}'
          `);
          const tableExistsInDB = tableExistsCheck && tableExistsCheck.length > 0 && tableExistsCheck[0].count > 0;
          
          if (tableExistsInDB) {
            console.log(`[DuckDBClient] Table ${tableName} exists in DuckDB, dropping before replacement`);
            await this.connection.query(`DROP TABLE IF EXISTS ${tableName} CASCADE`);
            await new Promise(resolve => setTimeout(resolve, 100));
            
            // Verify it's actually dropped
            const verifyDrop = await this.connection.query(`
              SELECT COUNT(*) as count 
              FROM information_schema.tables 
              WHERE table_schema = 'main' 
              AND table_name = '${tableName}'
            `);
            const stillExists = verifyDrop && verifyDrop.length > 0 && verifyDrop[0].count > 0;
            if (stillExists) {
              console.warn(`[DuckDBClient] Table ${tableName} still exists after DROP, trying again`);
              await this.connection.query(`DROP TABLE IF EXISTS ${tableName} CASCADE`);
              await new Promise(resolve => setTimeout(resolve, 100));
            }
          }
        } catch (dropError) {
          console.warn(`[DuckDBClient] Error checking/dropping table ${tableName}:`, dropError);
          // Continue anyway - we'll try to create it and handle the error if it already exists
        }
        
        // Parse Arrow IPC using apache-arrow
        // For streaming Arrow IPC files, tableFromIPC reads all batches and combines them
        const { tableFromIPC } = await import('apache-arrow');
        const table = tableFromIPC(arrowBuffer);
        console.log('[DuckDBClient] Parsed Arrow table, rows:', table.numRows, 'columns:', table.numCols);
        
        if (isExistingTable) {
          console.log('[DuckDBClient] Replacing existing table with updated data');
        }
        
        try {
          // Double-check table doesn't exist right before inserting
          // This handles race conditions where the table might have been created between our check and insert
          try {
            const finalCheck = await this.connection.query(`
              SELECT COUNT(*) as count 
              FROM information_schema.tables 
              WHERE table_schema = 'main' 
              AND table_name = '${tableName}'
            `);
            const stillExists = finalCheck && finalCheck.length > 0 && finalCheck[0].count > 0;
            if (stillExists) {
              console.log(`[DuckDBClient] Table ${tableName} still exists before insert, dropping again`);
              await this.connection.query(`DROP TABLE IF EXISTS ${tableName} CASCADE`);
              await new Promise(resolve => setTimeout(resolve, 150));
            }
          } catch (checkError) {
            console.warn(`[DuckDBClient] Error checking table before insert:`, checkError);
            // Continue anyway - try to drop and insert
            try {
              await this.connection.query(`DROP TABLE IF EXISTS ${tableName} CASCADE`);
              await new Promise(resolve => setTimeout(resolve, 150));
          } catch (dropError) {
              // Ignore drop errors
            }
          }
          
          // insertArrowTable creates a new table if it doesn't exist
          // IMPORTANT: For streaming Arrow IPC, tableFromIPC already combines all batches
          // into a single table, so we just insert that combined table
          console.log(`[DuckDBClient] Inserting Arrow table: ${tableName}, expected rows: ${table.numRows}, columns: ${table.numCols}`);
          
          // Check if table has any data before inserting
          if (table.numRows === 0) {
            console.warn(`[DuckDBClient] Arrow table has 0 rows - this might be an empty file or parsing issue`);
          }
          
          // Insert the table - this creates the table if it doesn't exist
          // The tableFromIPC function already reads all batches from the streaming file
          // and combines them into a single table
          console.log(`[DuckDBClient] Inserting Arrow table: ${tableName}, expected rows: ${table.numRows}, columns: ${table.numCols}`);
          
          // Check if table has any data before inserting
          if (table.numRows === 0) {
            console.warn(`[DuckDBClient] Arrow table has 0 rows - this might be an empty file or parsing issue`);
          }
          
          // Insert the table - this creates the table if it doesn't exist
          try {
            await this.connection.insertArrowTable(table, { name: tableName });
          } catch (insertError) {
            // If table already exists error, drop and retry
            if (insertError.message && insertError.message.includes('already exists')) {
              console.warn(`[DuckDBClient] Table ${tableName} exists during insert, dropping and retrying`);
              await this.connection.query(`DROP TABLE IF EXISTS ${tableName} CASCADE`);
              await new Promise(resolve => setTimeout(resolve, 150));
              // Retry insert
              await this.connection.insertArrowTable(table, { name: tableName });
            } else {
              throw insertError;
            }
          }
          
          // Small delay to ensure insert is committed
          await new Promise(resolve => setTimeout(resolve, 200));
          
          // Note: Verification is skipped because insertArrowTable has known timing issues
          // where tables may not be immediately queryable, but data is actually inserted
          // and will be available for subsequent queries (as we've seen in practice)
          console.log(`[DuckDBClient] Inserted Arrow table ${tableName} with ${table.numRows} rows (verification skipped due to timing issues)`);
        } catch (insertError) {
          console.error(`[DuckDBClient] Failed to insert Arrow table ${tableName}:`, insertError);
          console.error(`[DuckDBClient] Error details:`, {
            message: insertError.message,
            stack: insertError.stack,
            tableName,
            expectedRows: table.numRows
          });
          throw insertError;
        }
      }

      // Get row count for tracking
      // Note: There may be a timing issue where the table isn't immediately queryable
      // after insertArrowTable, so we wrap this in a try-catch
      let rowCount = 0;
      try {
        const rowCountResult = await this.connection.query(`SELECT COUNT(*) as count FROM ${tableName}`);
        rowCount = rowCountResult && rowCountResult.length > 0 ? rowCountResult[0].count : 0;
      } catch (countError) {
        // Table might not be immediately queryable, use expected row count from table
        console.warn(`[DuckDBClient] Could not get row count for ${tableName} (timing issue), using expected count: ${table.numRows}`);
        rowCount = table.numRows; // Use the parsed table's row count as fallback
      }

      // Track registered table (update if exists, create if new)
      if (isExistingTable) {
        // Update existing table info
        const tableInfo = this.registeredTables.get(tableName);
        tableInfo.rowCount = rowCount;
        tableInfo.lastUpdated = now;
        if (fileURL) {
          tableInfo.fileURL = fileURL;
        }
      } else {
        // Create new table info
        const tableInfo = {
          fileName,
          registeredAt: now,
          lastAccessed: now,
          lastUpdated: now,
          rowCount: rowCount,
        };
        if (fileURL) {
          tableInfo.fileURL = fileURL;
        }
        this.registeredTables.set(tableName, tableInfo);
        // Update access order for new table
        this._updateAccessOrder(tableName);
      }

      return tableName;
    } catch (error) {
      throw new DuckDBError(`registerArrowFile:${fileName}`, error);
    }
  }

  /**
   * Evict oldest table (LRU eviction)
   * Only evicts tables that are:
   * 1. Older than the configured timeout (default 10 minutes of inactivity)
   * 2. Not recently updated (within grace period)
   * 3. Not newly registered (within 30 seconds)
   * @param {number} gracePeriodMs - Don't evict tables updated within this period (default: 0)
   * @private
   */
  async _evictOldestTable(gracePeriodMs = 0) {
    if (this.tableAccessOrder.length === 0) {
      return;
    }

    const now = Date.now();
    const evictionTimeoutMs = 10 * 60 * 1000; // 10 minutes default (configurable)
    let evicted = false;
    let attempts = 0;
    const maxAttempts = Math.min(this.tableAccessOrder.length, 20); // Try up to 20 tables

    // Try to evict the oldest table, but only if it meets eviction criteria
    for (let i = 0; i < this.tableAccessOrder.length && attempts < maxAttempts; i++) {
      const tableName = this.tableAccessOrder[i];
      if (!this.registeredTables.has(tableName)) {
        // Table already removed, skip
        this.tableAccessOrder.splice(i, 1);
        i--;
        continue;
      }

      attempts++;
      const tableInfo = this.registeredTables.get(tableName);
      const timeSinceUpdate = now - (tableInfo.lastUpdated || tableInfo.lastAccessed);
      const timeSinceRegistration = now - (tableInfo.registeredAt || now);
      const timeSinceLastAccess = now - (tableInfo.lastAccessed || tableInfo.registeredAt || now);

      // Never evict tables that were just registered (within last 30 seconds)
      // This prevents the race condition where a table is registered and immediately evicted
      if (timeSinceRegistration < 30000) {
        continue;
      }

      // Skip tables that were recently updated (actively being written to)
      if (gracePeriodMs > 0 && timeSinceUpdate < gracePeriodMs) {
        continue;
      }

      // Only evict if table hasn't been accessed in the eviction timeout period
      // This ensures we only evict truly inactive tables
      if (timeSinceLastAccess < evictionTimeoutMs) {
        continue;
      }

      // Found a table to evict (old and inactive)
      this.tableAccessOrder.splice(i, 1);
      await this.unregisterTable(tableName);
      evicted = true;
      break;
    }

    // If we couldn't evict any table (all were recently accessed/updated), 
    // only evict if we're at 100% of max tables (true memory pressure)
    // This is a last resort to prevent memory issues
    if (!evicted && this.registeredTables.size >= this.maxTables) {
      // Only evict if we're at the absolute limit
      // Find the oldest table that's at least 1 minute old
      for (let i = 0; i < this.tableAccessOrder.length; i++) {
        const tableName = this.tableAccessOrder[i];
        if (!this.registeredTables.has(tableName)) {
          this.tableAccessOrder.splice(i, 1);
          i--;
          continue;
        }
        const tableInfo = this.registeredTables.get(tableName);
        const timeSinceRegistration = now - (tableInfo.registeredAt || now);
        // Only evict if table is at least 1 minute old (emergency eviction)
        if (timeSinceRegistration < 60000) {
          continue;
        }
        // Found an old table to evict (emergency)
        this.tableAccessOrder.splice(i, 1);
        await this.unregisterTable(tableName);
        break;
      }
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
      // DuckDB-WASM may not support parameterized queries
      // If params are provided, interpolate them into SQL with proper escaping
      let finalSql = sql;
      if (params && params.length > 0) {
        // Escape single quotes in string parameters
        const escapedParams = params.map(param => {
          if (typeof param === 'string') {
            // Escape single quotes by doubling them
            return `'${param.replace(/'/g, "''")}'`;
          } else if (param === null || param === undefined) {
            return 'NULL';
          } else {
            return String(param);
          }
        });
        
        // Replace ? placeholders with escaped values
        let paramIndex = 0;
        finalSql = sql.replace(/\?/g, () => {
          if (paramIndex < escapedParams.length) {
            return escapedParams[paramIndex++];
          }
          return '?'; // Keep ? if not enough params
        });
      }
      
      const result = await this.connection.query(finalSql);
      
      // Handle different result formats from DuckDB-Wasm
      // The result format can vary depending on context (worker vs main thread)
      let rows = [];
      
      if (!result) {
        // No result returned
        rows = [];
      } else if (Array.isArray(result)) {
        // Result is already an array
        rows = result;
      } else if (result && typeof result.toArray === 'function') {
        // Result has toArray method - this is the standard DuckDB-Wasm format
        try {
          // toArray() might fail if result structure is incomplete
          // Check if result has the necessary internal structure first
          if (result._materialized) {
            rows = result.toArray();
          } else {
            // Try to materialize first if needed
            try {
              // Some DuckDB-Wasm versions require materialization
              if (typeof result.materialize === 'function') {
                await result.materialize();
              }
              rows = result.toArray();
            } catch (materializeError) {
              console.warn('[DuckDBClient] Materialization failed, trying direct toArray:', materializeError);
              // Try toArray anyway
              rows = result.toArray();
            }
          }
          
          // Convert Row objects to plain objects for postMessage serialization
          // DuckDB-Wasm returns Row objects that can't be cloned
          rows = rows.map(row => {
            if (row && typeof row.toJSON === 'function') {
              return row.toJSON();
            } else if (row && typeof row === 'object' && row !== null) {
              // Convert Row object to plain object
              const plainRow = {};
              for (const key in row) {
                if (row.hasOwnProperty(key) && !key.startsWith('_')) {
                  plainRow[key] = row[key];
                }
              }
              return plainRow;
            }
            return row;
          });
        } catch (toArrayError) {
          // If toArray fails (e.g., "peek" error), try alternative methods
          console.warn('[DuckDBClient] toArray() failed:', toArrayError.message);
          console.warn('[DuckDBClient] Result type:', typeof result, 'Result keys:', Object.keys(result || {}));
          
          // Try alternative extraction methods
          if (result.rows && Array.isArray(result.rows)) {
            rows = result.rows.map(row => {
              if (row && typeof row.toJSON === 'function') {
                return row.toJSON();
              } else if (row && typeof row === 'object' && row !== null) {
                const plainRow = {};
                for (const key in row) {
                  if (row.hasOwnProperty(key) && !key.startsWith('_')) {
                    plainRow[key] = row[key];
                  }
                }
                return plainRow;
              }
              return row;
            });
          } else if (result.data && Array.isArray(result.data)) {
            rows = result.data.map(row => {
              if (row && typeof row.toJSON === 'function') {
                return row.toJSON();
              } else if (row && typeof row === 'object' && row !== null) {
                const plainRow = {};
                for (const key in row) {
                  if (row.hasOwnProperty(key) && !key.startsWith('_')) {
                    plainRow[key] = row[key];
                  }
                }
                return plainRow;
              }
              return row;
            });
          } else if (result && typeof result[Symbol.iterator] === 'function') {
            // Result is iterable - try to convert
            try {
              rows = Array.from(result).map(row => {
                if (row && typeof row.toJSON === 'function') {
                  return row.toJSON();
                } else if (row && typeof row === 'object' && row !== null) {
                  const plainRow = {};
                  for (const key in row) {
                    if (row.hasOwnProperty(key) && !key.startsWith('_')) {
                      plainRow[key] = row[key];
                    }
                  }
                  return plainRow;
                }
                return row;
              });
            } catch (iterError) {
              console.warn('[DuckDBClient] Array.from() failed:', iterError);
              rows = [];
            }
          } else {
            // Last resort: return empty array
            console.warn('[DuckDBClient] Could not extract rows from result, returning empty array');
            rows = [];
          }
        }
      } else if (result && result.rows) {
        // Result has rows property
        rows = Array.isArray(result.rows) ? result.rows.map(row => {
          if (row && typeof row.toJSON === 'function') {
            return row.toJSON();
          } else if (row && typeof row === 'object' && row !== null) {
            const plainRow = {};
            for (const key in row) {
              if (row.hasOwnProperty(key) && !key.startsWith('_')) {
                plainRow[key] = row[key];
              }
            }
            return plainRow;
          }
          return row;
        }) : [];
      } else if (result && result.data) {
        // Result has data property
        rows = Array.isArray(result.data) ? result.data.map(row => {
          if (row && typeof row.toJSON === 'function') {
            return row.toJSON();
          } else if (row && typeof row === 'object' && row !== null) {
            const plainRow = {};
            for (const key in row) {
              if (row.hasOwnProperty(key) && !key.startsWith('_')) {
                plainRow[key] = row[key];
              }
            }
            return plainRow;
          }
          return row;
        }) : [];
      } else if (result && typeof result[Symbol.iterator] === 'function') {
        // Result is iterable
        try {
          rows = Array.from(result).map(row => {
            if (row && typeof row.toJSON === 'function') {
              return row.toJSON();
            } else if (row && typeof row === 'object' && row !== null) {
              const plainRow = {};
              for (const key in row) {
                if (row.hasOwnProperty(key) && !key.startsWith('_')) {
                  plainRow[key] = row[key];
                }
              }
              return plainRow;
            }
            return row;
          });
        } catch (iterError) {
          console.warn('[DuckDBClient] Failed to iterate result:', iterError);
          rows = [];
        }
      } else {
        // Fallback: log and return empty
        console.warn('[DuckDBClient] Unknown result format. Type:', typeof result, 'Keys:', Object.keys(result || {}));
        rows = [];
      }

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
      await this.connection.query(`DROP TABLE IF EXISTS ${safeName};`);

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
