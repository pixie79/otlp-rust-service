const eventTarget = () => {
  if (typeof EventTarget !== 'undefined') {
    return new EventTarget();
  }
  // Minimal polyfill for environments lacking EventTarget (e.g., some tests)
  return {
    listeners: {},
    addEventListener(type, listener) {
      this.listeners[type] = this.listeners[type] ?? new Set();
      this.listeners[type].add(listener);
    },
    removeEventListener(type, listener) {
      this.listeners[type]?.delete(listener);
    },
    dispatchEvent(event) {
      this.listeners[event.type]?.forEach((listener) => listener(event));
    },
  };
};

export class DataWorkerClient {
  constructor() {
    const workerSupported = typeof window !== 'undefined' && typeof Worker !== 'undefined';
    this.worker = workerSupported
      ? new Worker(new URL('./data-worker.js', import.meta.url), { type: 'module' })
      : null;
    this._pending = new Map();
    this._counter = 0;
    this._events = eventTarget();

    this._onMessage = this._onMessage.bind(this);
    this.worker?.addEventListener('message', this._onMessage);
  }

  async init() {
    return this._request('INIT');
  }

  async registerFile(fileName, fileURLOrBuffer) {
    // Support both fileURL (string) and buffer (ArrayBuffer)
    if (fileURLOrBuffer instanceof ArrayBuffer) {
      // Local file - pass buffer with transfer
      return this._request('REGISTER_FILE', { fileName, buffer: fileURLOrBuffer }, [fileURLOrBuffer]);
    } else {
      // Server-served file - pass fileURL
      return this._request('REGISTER_FILE', { fileName, fileURL: fileURLOrBuffer });
    }
  }

  async query(sql, params = []) {
    return this._request('QUERY', { sql, params });
  }

  async clearTables() {
    return this._request('CLEAR_TABLES');
  }

  async unregisterTable(tableName) {
    return this._request('UNREGISTER_TABLE', { tableName });
  }

  async shutdown() {
    await this._request('SHUTDOWN');
    this.worker?.terminate();
    this._pending.clear();
  }

  publish(eventType, detail) {
    if (!this._events) return;
    const event =
      typeof CustomEvent !== 'undefined'
        ? new CustomEvent(eventType, { detail })
        : { type: eventType, detail };
    this._events.dispatchEvent(event);
  }

  subscribe(eventType, callback) {
    if (!this._events) {
      return () => {};
    }
    const handler = (event) => callback(event.detail ?? event);
    this._events.addEventListener(eventType, handler);
    return () => this._events.removeEventListener(eventType, handler);
  }

  _request(type, payload = {}, transferList) {
    if (!this.worker) {
      // Worker not supported - log warning and return empty result
      // This makes the lack of Worker support explicit rather than silently failing
      console.warn(
        'Web Workers are not supported in this browser. Dashboard functionality will be limited.'
      );
      return Promise.resolve({});
    }
    const id = ++this._counter;
    const message = { id, type, payload };
    const promise = new Promise((resolve, reject) => {
      this._pending.set(id, { resolve, reject });
    });
    if (transferList) {
      this.worker.postMessage(message, transferList);
    } else {
      this.worker.postMessage(message);
    }
    return promise;
  }

  _onMessage(event) {
    const { id, type, payload } = event.data ?? {};

    if (type === 'ERROR' && id) {
      this._pending.get(id)?.reject(new Error(payload?.message ?? 'Worker error'));
      this._pending.delete(id);
      return;
    }

    if (id && this._pending.has(id)) {
      this._pending.get(id)?.resolve(payload ?? {});
      this._pending.delete(id);
    }

    if (type && !['ERROR'].includes(type)) {
      this.publish(type, payload);
    }
  }
}
