import { configManager } from './config.js';
import { FileReaderComponent } from './file/file-reader.js';
import { FileWatcher } from './file/file-watcher.js';
import { TraceList } from './traces/trace-list.js';
import { TraceDetail } from './traces/trace-detail.js';
import { TraceFilter } from './traces/trace-filter.js';
import { TraceQuery } from './traces/trace-query.js';
import { DataWorkerClient } from './workers/data-worker-client.js';

const statusBadge = (text) => `<span class="status-badge">${text}</span>`;

export class App {
  constructor(rootEl) {
    this.root = rootEl;
    this.fileReader = new FileReaderComponent();
    this.fileWatcher = new FileWatcher(this.fileReader);
    this.workerClient = new DataWorkerClient();
    this.state = {
      directory: null,
      tables: new Map(),
      status: 'Idle',
    };
    this.activeFilters = {
      traceId: '',
      serviceName: '',
      spanName: '',
      errorOnly: false,
    };
  }

  async initialize() {
    this._renderShell();
    this._attachEventHandlers();
    this._instantiateTraceComponents();
    await this.workerClient.init();
    this.traceQuery = new TraceQuery({
      execute: async (sql, params) => {
        const result = await this.workerClient.query(sql, params);
        return { rows: result.rows ?? [] };
      },
    });
    this._setStatus('Ready');
  }

  destroy() {
    this.fileWatcher.stopWatching();
    this.workerClient.shutdown().catch((error) => {
      console.error('Failed to shutdown worker', error);
    });
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
        <section class="panel trace-panel" data-testid="trace-panel">
          <div class="trace-panel__header">
            <div>
              <h2>Traces</h2>
              <p class="subtitle">Live tail viewer with filtering and detail pane.</p>
            </div>
            <button class="ghost" id="refresh-traces">Refresh</button>
          </div>
          <div id="trace-filter"></div>
          <div class="trace-panel__content">
            <div id="trace-list"></div>
            <div id="trace-detail"></div>
          </div>
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
    const refreshButton = document.getElementById('refresh-traces');
    refreshButton?.addEventListener('click', () => this._refreshTraces());

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

  async _ingestFile(fileHandle, metadata) {
    try {
      const name = metadata?.name ?? fileHandle.name ?? 'unknown.arrow';
      const buffer = await this.fileReader.readFile(fileHandle, name);
      const { tableName } = await this.workerClient.registerFile(name, buffer);
      this.state.tables.set(name, tableName);
      this._log(`Registered ${name} as ${tableName} in DuckDB.`);
      await this._refreshTraces();
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

  _instantiateTraceComponents() {
    this.traceList = new TraceList(document.getElementById('trace-list'));
    this.traceDetail = new TraceDetail(document.getElementById('trace-detail'));
    this.traceFilter = new TraceFilter(document.getElementById('trace-filter'));

    this.traceFilter.onChange = (filters) => {
      this.activeFilters = filters;
      this.traceList.applyFilters(filters);
      this._refreshTraces();
    };

    this.traceFilter.setFilters(this.activeFilters);
    this.traceList.applyFilters(this.activeFilters);

    this.traceList.onTraceSelected = (trace) => this.traceDetail.showTrace(trace);
    this.traceList.bindWorker(this.workerClient);
  }

  async _refreshTraces() {
    if (!this.traceQuery || !this.state.tables.size) {
      return;
    }
    const tableNames = Array.from(this.state.tables.values());
    try {
      const traces = await this.traceQuery.fetchLatestFromTables(tableNames, {
        ...this.activeFilters,
        limit: configManager.get('maxTraces'),
      });
      this.workerClient.publish('TRACE_BATCH', { traces });

      if ((this.traceList.filteredTraces ?? []).length === 0) {
        this.traceDetail.clear();
      }
    } catch (error) {
      console.error('Failed to refresh traces', error);
      this._log(`Trace refresh failed: ${error.message}`);
    }
  }
}
