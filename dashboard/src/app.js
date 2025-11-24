import { configManager } from './config.js';
import { FileReaderComponent } from './file/file-reader.js';
import { FileWatcher } from './file/file-watcher.js';
import { DuckDBClient } from './duckdb/duckdb-client.js';

const statusBadge = (text) => `<span class="status-badge">${text}</span>`;

export class App {
  constructor(rootEl) {
    this.root = rootEl;
    this.fileReader = new FileReaderComponent();
    this.fileWatcher = new FileWatcher(this.fileReader);
    this.duckdb = new DuckDBClient();
    this.state = {
      directory: null,
      tables: new Map(),
      status: 'Idle',
    };
  }

  async initialize() {
    this._renderShell();
    this._attachEventHandlers();
    await this._ensureDuckDBReady();
    this._setStatus('Ready');
  }

  destroy() {
    this.fileWatcher.stopWatching();
    this.duckdb.close();
  }

  _renderShell() {
    this.root.innerHTML = `
      <header class="app-header panel">
        <div>
          <h1>OTLP Realtime Dashboard</h1>
          <p class="subtitle">Monitor Arrow IPC traces and metrics directly in your browser.</p>
        </div>
        <div class="status-line" id="status-line">${statusBadge(this.state.status)}</div>
      </header>
      <main class="app-main">
        <section class="panel intro-panel">
          <p>Select the OTLP output directory to start streaming Arrow IPC files.</p>
          <button class="primary" id="select-directory">Choose Directory</button>
        </section>
        <section class="panel muted" id="log-panel">
          <h2>Activity</h2>
          <ul id="log-list"></ul>
        </section>
      </main>
    `;
  }

  _attachEventHandlers() {
    const selectButton = document.getElementById('select-directory');
    selectButton?.addEventListener('click', () => this._handleDirectorySelection());

    this.fileWatcher.onNewFile = (fileHandle, metadata) => {
      this._log(`Detected new file: ${metadata?.name ?? fileHandle.name}`);
      this._ingestFile(fileHandle, metadata);
    };

    this.fileWatcher.onFileChanged = (fileHandle, metadata) => {
      this._log(`Detected modified file: ${metadata?.name ?? fileHandle.name}`);
      this._ingestFile(fileHandle, metadata);
    };
  }

  async _handleDirectorySelection() {
    try {
      this._setStatus('Awaiting permission…');
      const directory = await this.fileReader.selectDirectory();
      this.state.directory = directory;
      this._setStatus('Watching for updates');
      this._log('Directory access granted. Starting watcher…');
      this.fileWatcher.startWatching(directory, configManager.get('pollingIntervalMs'));
      await this.fileWatcher.checkForChanges();
    } catch (error) {
      console.error(error);
      this._setStatus('Permission denied');
      this._log(`Directory selection failed: ${error.message}`);
    }
  }

  async _ensureDuckDBReady() {
    this._setStatus('Initializing DuckDB…');
    await this.duckdb.initialize();
  }

  async _ingestFile(fileHandle, metadata) {
    try {
      const name = metadata?.name ?? fileHandle.name ?? 'unknown.arrow';
      const buffer = await this.fileReader.readFile(fileHandle, name);
      const tableName = await this.duckdb.registerArrowFile(name, buffer);
      this.state.tables.set(name, tableName);
      this._log(`Registered ${name} as ${tableName} in DuckDB.`);
    } catch (error) {
      console.error('Failed to ingest file', error);
      this._log(`Failed to ingest ${metadata?.name ?? 'file'}: ${error.message}`);
    }
  }

  _setStatus(statusText) {
    this.state.status = statusText;
    const statusLine = document.getElementById('status-line');
    if (statusLine) {
      statusLine.innerHTML = statusBadge(statusText);
    }
  }

  _log(message) {
    const list = document.getElementById('log-list');
    if (!list) {
      return;
    }
    const item = document.createElement('li');
    item.textContent = `[${new Date().toLocaleTimeString()}] ${message}`;
    list.prepend(item);
    while (list.children.length > 50) {
      list.removeChild(list.lastChild);
    }
  }
}
