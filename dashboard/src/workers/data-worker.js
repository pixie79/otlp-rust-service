import { DuckDBClient } from '../duckdb/duckdb-client.js';
import { configManager } from '../config.js';

// Initialize DuckDB client with memory management
const duckdbClient = new DuckDBClient({
  maxTables: configManager.get('maxLoadedFiles'),
});

const handlers = {
  async INIT() {
    await duckdbClient.initialize();
    return { status: 'ready' };
  },
  async REGISTER_FILE(payload) {
    const { fileName, buffer } = payload;
    if (!fileName || !buffer) {
      throw new Error('REGISTER_FILE requires fileName and buffer');
    }
    const tableName = await duckdbClient.registerArrowFile(fileName, buffer);
    return { tableName };
  },
  async QUERY(payload) {
    const { sql, params } = payload;
    if (!sql) {
      throw new Error('QUERY requires sql');
    }
    const rows = await duckdbClient.query(sql, params);
    return { rows };
  },
  async UNREGISTER_TABLE(payload) {
    const { tableName } = payload;
    if (!tableName) {
      throw new Error('UNREGISTER_TABLE requires tableName');
    }
    await duckdbClient.unregisterTable(tableName);
    return { tableName };
  },
  async SHUTDOWN() {
    await duckdbClient.close();
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
