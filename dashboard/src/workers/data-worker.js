import { DuckDBClient } from '../duckdb/duckdb-client.js';
import { configManager } from '../config.js';

// Initialize DuckDB client with memory management
const duckdbClient = new DuckDBClient({
  maxTables: configManager.get('maxLoadedFiles'),
});

// Track initialization state
let isInitialized = false;

const handlers = {
  async INIT() {
    try {
      console.log('[Worker] Initializing DuckDB client...');
      await duckdbClient.initialize();
      isInitialized = true;
      console.log('[Worker] DuckDB client initialized successfully');
      return { status: 'ready' };
    } catch (error) {
      isInitialized = false;
      console.error('[Worker] DuckDB initialization failed:', error);
      console.error('[Worker] Error stack:', error.stack);
      throw error;
    }
  },
  async REGISTER_FILE(payload) {
    if (!isInitialized) {
      throw new Error('DuckDBClient is not initialized. Call INIT first.');
    }
    const { fileName, fileURL, buffer } = payload;
    
    // Support both fileURL (for server-served files) and buffer (for local files)
    if (buffer) {
      // Local file - use buffer directly
      if (!fileName) {
        throw new Error('REGISTER_FILE with buffer requires fileName');
      }
      const tableName = await duckdbClient.registerArrowFile(fileName, buffer);
      return { tableName };
    } else if (fileURL) {
      // Server-served file - use fileURL
      if (!fileName) {
        throw new Error('REGISTER_FILE with fileURL requires fileName');
      }
      const tableName = await duckdbClient.registerArrowFile(fileName, fileURL);
      return { tableName };
    } else {
      throw new Error('REGISTER_FILE requires either fileURL or buffer');
    }
  },
  async QUERY(payload) {
    if (!isInitialized) {
      throw new Error('DuckDBClient is not initialized. Call INIT first.');
    }
    const { sql, params } = payload;
    if (!sql) {
      throw new Error('QUERY requires sql');
    }
    const rows = await duckdbClient.query(sql, params);
    return { rows };
  },
  async UNREGISTER_TABLE(payload) {
    if (!isInitialized) {
      throw new Error('DuckDBClient is not initialized. Call INIT first.');
    }
    const { tableName } = payload;
    if (!tableName) {
      throw new Error('UNREGISTER_TABLE requires tableName');
    }
    await duckdbClient.unregisterTable(tableName);
    return { tableName };
  },
  async CLEAR_TABLES() {
    // If DuckDB isn't initialized yet, clearing tables is a no-op
    // This is OK - we'll clear tables when DuckDB initializes anyway
    if (!isInitialized) {
      console.log('[Worker] CLEAR_TABLES called but DuckDB not initialized yet (no-op)');
      return { status: 'cleared', note: 'DuckDB not initialized yet' };
    }
    await duckdbClient.clearAllTables();
    return { status: 'cleared' };
  },
  async SHUTDOWN() {
    if (isInitialized) {
      await duckdbClient.close();
      isInitialized = false;
    }
    return { status: 'closed' };
  },
};

const postResponse = (type, payload, id, transfer) => {
  const message = { type, payload, id };
  if (transfer?.length) {
    self.postMessage(message, transfer);
    return;
  }
  self.postMessage(message);
};

self.addEventListener('message', async (event) => {
  const { id, type, payload } = event.data ?? {};

  if (!type) {
    return;
  }

  const handler = handlers[type];
  if (!handler) {
    postResponse('ERROR', { message: `Unknown worker message type: ${type}` }, id);
    return;
  }

  try {
    const result = await handler(payload ?? {});
    postResponse(`${type}_OK`, result, id);
  } catch (error) {
    postResponse(
      'ERROR',
      {
        message: error.message,
        name: error.name,
        stack: error.stack,
        originatingType: type,
      },
      id
    );
  }
});
