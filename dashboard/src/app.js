import { configManager } from './config.js';
import { FileReaderComponent } from './file/file-reader.js';
import { FileWatcher } from './file/file-watcher.js';
import { TraceList } from './traces/trace-list.js';
import { TraceDetail } from './traces/trace-detail.js';
import { TraceFilter } from './traces/trace-filter.js';
import { TraceQuery } from './traces/trace-query.js';
import { MetricGraph } from './metrics/metric-graph.js';
import { MetricSelector } from './metrics/metric-selector.js';
import { MetricQuery } from './metrics/metric-query.js';
import { DataWorkerClient } from './workers/data-worker-client.js';
import { Layout } from './ui/layout.js';
import { Navigation } from './ui/navigation.js';
import { Loading } from './ui/loading.js';
import { Search } from './ui/search.js';
import { TimeRangeSelector } from './ui/time-range-selector.js';
import { Settings } from './ui/settings.js';
import { SQLTerminal } from './ui/sql-terminal.js';

// Status badge helper function (currently unused, kept for potential future use)
// const statusBadge = (text) => `<span class="status-badge">${text}</span>`;

/**
 * Main application class for OTLP Realtime Dashboard
 *
 * Manages the overall application state, coordinates between components,
 * handles file watching, data ingestion, and UI updates.
 *
 * @class App
 */
export class App {
  /**
   * Create a new App instance
   * @param {HTMLElement} rootEl - Root DOM element for the application
   */
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
    this.currentView = this._loadState('currentView') || 'traces'; // 'traces' or 'metrics'
    this.metricGraphs = new Map(); // Map of metric name to MetricGraph instance
    this.isPaused = this._loadState('isPaused') || false; // Pause/resume live stream
    this.loadingComponent = null;
    this.searchComponent = null;

    // Data limits for memory management
    this.maxTraces = 10000;
    this.maxGraphPoints = 10000;
    this.maxLoadedTables = 50;
  }

  /**
   * Initialize the application
   * Sets up UI components, event handlers, and starts the worker client
   * @returns {Promise<void>}
   */
  /**
   * Initialize the application
   * Sets up UI components, event handlers, and starts the worker client
   * @returns {Promise<void>}
   */
  async initialize() {
    // Check browser compatibility
    if (!this._checkBrowserCompatibility()) {
      this._showBrowserIncompatibility();
      return;
    }

    // Set up error boundary
    this._setupErrorBoundary();

    // Initialize UI components
    this.layout = new Layout(this.root);
    this.layout.render();

    // Initialize navigation
    const navContainer = this.root.querySelector('.view-nav');
    if (navContainer) {
      this.navigation = new Navigation(navContainer, {
        currentView: this.currentView,
        onViewChanged: (view) => this._switchView(view),
      });
      this.navigation.render();
    }

    // Initialize loading component
    const loadingContainer = document.createElement('div');
    loadingContainer.id = 'loading-container';
    this.root.querySelector('.app-main')?.prepend(loadingContainer);
    this.loadingComponent = new Loading(loadingContainer);

    // Initialize search
    const searchContainer = document.createElement('div');
    searchContainer.id = 'search-container';
    this.root.querySelector('.app-header')?.appendChild(searchContainer);
    this.searchComponent = new Search(searchContainer, {
      placeholder: 'Search by trace ID or metric name...',
      onSearch: (query) => this._handleSearch(query),
      onClear: () => this._handleSearchClear(),
    });
    this.searchComponent.render();

    // Initialize settings (hidden by default, accessible via button)
    this._initializeSettings();

    this._attachEventHandlers();
    this._attachKeyboardHandlers();
    this._instantiateTraceComponents();
    this._instantiateMetricComponents();
    this._instantiateSQLTerminal();

    await this.workerClient.init();

    // Clear state on initialization to ensure clean state after page refresh/restart
    this.state.tables.clear();
    if (this.fileWatcher) {
      this.fileWatcher.clearCache();
      // Clear known files to force re-detection
      this.fileWatcher.knownFiles?.clear();
    }

    this.traceQuery = new TraceQuery({
      execute: async (sql, params) => {
        const result = await this.workerClient.query(sql, params);
        return { rows: result.rows ?? [] };
      },
    });
    // Set callback to clean up missing tables from state
    this.traceQuery.onTableMissing = (tableNames) => {
      this._removeMissingTables(tableNames);
    };
    this.metricQuery = new MetricQuery({
      execute: async (sql, params) => {
        const result = await this.workerClient.query(sql, params);
        return { rows: result.rows ?? [] };
      },
    });
    // Set callback to clean up missing tables from state
    this.metricQuery.onTableMissing = (tableNames) => {
      this._removeMissingTables(tableNames);
    };

    // Restore paused state
    if (this.isPaused) {
      this.pauseStream();
    }

    this._setStatus('Ready');
  }

  destroy() {
    this.fileWatcher.stopWatching();
    this.workerClient.shutdown().catch((error) => {
      console.error('Failed to shutdown worker', error);
    });
  }

  // Removed _renderShell - now using Layout component

  _attachEventHandlers() {
    const selectButton = document.getElementById('select-directory');
    selectButton?.addEventListener('click', () => this._handleDirectorySelection());
    const refreshButton = document.getElementById('refresh-traces');
    refreshButton?.addEventListener('click', () => this._refreshTraces());
    const refreshMetricsButton = document.getElementById('refresh-metrics');
    refreshMetricsButton?.addEventListener('click', () => this._refreshMetrics());

    // Navigation between views
    const navTraces = document.getElementById('nav-traces');
    navTraces?.addEventListener('click', () => this._switchView('traces'));
    const navMetrics = document.getElementById('nav-metrics');
    navMetrics?.addEventListener('click', () => this._switchView('metrics'));

    // Settings button
    const settingsButton = document.getElementById('settings-button');
    if (settingsButton) {
      settingsButton.addEventListener('click', () => this._toggleSettings());
    }

    this.fileWatcher.onNewFile = (fileHandle, metadata) => {
      this._log(`Detected new file: ${metadata?.name ?? fileHandle.name}`);
      this._ingestFile(fileHandle, metadata);
    };

    this.fileWatcher.onFileChanged = async (fileHandle, metadata) => {
      const fileName = metadata?.name ?? fileHandle.name ?? 'unknown';
      this._log(`Detected modified file: ${fileName} (size: ${metadata?.size ?? 'unknown'} bytes)`);

      // Small delay to ensure file write is complete before reading
      // This is especially important for streaming Arrow files that are being actively written to
      await new Promise((resolve) => setTimeout(resolve, 100));

      await this._ingestFile(fileHandle, metadata);
    };
  }

  _switchView(view) {
    this.currentView = view;
    this._saveState('currentView', view);

    if (this.layout) {
      this.layout.switchView(view);
    }
    if (this.navigation) {
      this.navigation.switchToView(view);
    }

    if (view === 'traces') {
      this._refreshTraces();
    } else if (view === 'metrics') {
      this._refreshMetrics();
    } else if (view === 'sql') {
      // SQL terminal doesn't need refresh, it's user-driven
      if (this.sqlTerminal) {
        this.sqlTerminal.refreshTableList();
      }
    }
  }

  async _handleDirectorySelection() {
    try {
      this._setStatus('Awaiting permission…');
      const directory = await this.fileReader.selectDirectory();

      // Clear all tables and state when selecting a new directory
      // This ensures a fresh start when switching directories
      this._log('Clearing existing data...');

      // Only clear tables if DuckDB is initialized
      // If not initialized yet, just clear local state
      try {
        await this.workerClient.clearTables();
      } catch (clearError) {
        // DuckDB might not be initialized yet - that's OK, just clear local state
        if (clearError.message && clearError.message.includes('not initialized')) {
          this._log('DuckDB not initialized yet, skipping table clear');
        } else {
          throw clearError;
        }
      }

      this.state.tables.clear();
      if (this.fileWatcher) {
        this.fileWatcher.clearCache();
      }
      this._log('Data cleared. Starting fresh...');

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
      const name = metadata?.name ?? fileHandle.name ?? 'unknown.arrows';

      // Check if this file is already registered (might be an update)
      const existingTableName = this.state.tables.get(name);
      const isUpdate = existingTableName !== undefined;

      // For local files (File System Access API), read the buffer directly
      // For files served by the server, we can use fileURL
      // Since we're using File System Access API, we'll read the buffer directly
      // Always get fresh file data, especially important for files being actively written to
      const buffer = await this.fileReader.readFile(fileHandle, name, {
        chunkSize: 10 * 1024 * 1024, // 10MB chunks
        maxSize: 200 * 1024 * 1024, // 200MB max
      });

      console.log(
        `[App] Ingesting ${isUpdate ? 'updated' : 'new'} file: ${name}, size: ${buffer.byteLength} bytes`
      );

      const { tableName } = await this.workerClient.registerFile(name, buffer);
      this.state.tables.set(name, tableName);

      if (isUpdate) {
        this._log(
          `Updated ${name} as ${tableName} in DuckDB (${(buffer.byteLength / 1024).toFixed(2)} KB).`
        );
      } else {
        this._log(
          `Registered ${name} as ${tableName} in DuckDB (${(buffer.byteLength / 1024).toFixed(2)} KB).`
        );
      }

      // Don't proactively clean up tables - let DuckDB's LRU eviction handle it
      // This prevents removing tables that are actively being written to
      // The DuckDB client's eviction logic is smarter and respects grace periods

      // Refresh data views - use append mode if live tail is enabled
      const isLiveTail = this.traceList?.liveTailEnabled ?? false;
      await this._refreshTraces(isLiveTail && isUpdate);
      await this._refreshMetrics();

      // Refresh SQL terminal table list if it exists
      if (this.sqlTerminal) {
        this.sqlTerminal.refreshTableList();
      }
    } catch (error) {
      console.error('Failed to ingest file', error);
      const errorMessage = error.message || 'Unknown error';
      this._log(`Failed to ingest ${metadata?.name ?? 'file'}: ${errorMessage}`);

      // Show user-friendly error message in loading component
      if (this.loadingComponent) {
        this.loadingComponent.showError(error, {
          details: error.stack,
        });
      }

      // Also update status
      if (error.name === 'FileReadError') {
        this._setStatus('File read error');
      } else if (error.name === 'DuckDBError') {
        this._setStatus('Database error');
      }
    }
  }

  /**
   * Clean up old tables to manage memory
   * @private
   */
  async _cleanupOldTables(maxFiles) {
    const tables = Array.from(this.state.tables.entries());
    const toRemove = tables.slice(0, tables.length - maxFiles);

    for (const [fileName, tableName] of toRemove) {
      try {
        await this.workerClient.unregisterTable(tableName);
        this.state.tables.delete(fileName);
        this._log(`Unregistered old table: ${fileName}`);
      } catch (error) {
        console.error(`Failed to unregister table ${tableName}:`, error);
      }
    }
  }

  /**
   * Remove missing tables from state (e.g., evicted tables)
   * @private
   */
  _removeMissingTables(tableNames) {
    for (const tableName of tableNames) {
      // Find fileName(s) that map to this tableName
      for (const [fileName, mappedTableName] of this.state.tables.entries()) {
        if (mappedTableName === tableName) {
          this.state.tables.delete(fileName);
          this._log(`Removed missing table from state: ${fileName} (${tableName})`);
        }
      }
    }
  }

  _setStatus(statusText) {
    this.state.status = statusText;
    if (this.layout) {
      this.layout.setStatus(statusText);
    }
  }

  _log(message) {
    const list = this.root.querySelector('#log-list');
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
    this.traceList = new TraceList(this.root.querySelector('#trace-list'));
    this.traceDetail = new TraceDetail(this.root.querySelector('#trace-detail'));
    this.traceFilter = new TraceFilter(this.root.querySelector('#trace-filter'));

    this.traceFilter.onChange = (filters) => {
      this.activeFilters = filters;
      this.traceList.applyFilters(filters);
      if (!filters.liveTail) {
        this._refreshTraces();
      }
    };

    this.traceFilter.onLiveTailToggle = (enabled) => {
      this.traceList.toggleLiveTail(enabled, () => {
        this._refreshTraces(true);
      });
    };

    this.traceFilter.setFilters(this.activeFilters);
    this.traceList.applyFilters(this.activeFilters);

    this.traceList.onTraceSelected = (trace) => this.traceDetail.showTrace(trace);
    this.traceList.bindWorker(this.workerClient);
  }

  async _refreshTraces(append = false) {
    if (!this.traceQuery || !this.state.tables.size) {
      console.log('[App] _refreshTraces: No traceQuery or tables available');
      return;
    }
    const tableNames = Array.from(this.state.tables.values());
    console.log(`[App] _refreshTraces: Querying ${tableNames.length} tables, append=${append}`);
    try {
      const traces = await this.traceQuery.fetchLatestFromTables(tableNames, {
        ...this.activeFilters,
        limit: configManager.get('maxTraces'),
      });

      console.log(`[App] _refreshTraces: Fetched ${traces.length} traces`);

      if (append) {
        // For live tail: append new traces
        console.log('[App] _refreshTraces: Publishing TRACE_BATCH_APPEND');
        this.workerClient.publish('TRACE_BATCH_APPEND', { traces });
      } else {
        // Normal refresh: replace all traces
        console.log('[App] _refreshTraces: Publishing TRACE_BATCH');
        this.workerClient.publish('TRACE_BATCH', { traces });
      }

      if ((this.traceList.filteredTraces ?? []).length === 0) {
        this.traceDetail.clear();
      }
    } catch (error) {
      console.error('[App] Failed to refresh traces', error);
      // Only log error if it's not a missing table error (those are handled gracefully)
      if (!error.message?.includes('does not exist')) {
        this._log(`Trace refresh failed: ${error.message}`);
      }
    }
  }

  _instantiateMetricComponents() {
    const selectorContainer = this.root.querySelector('#metric-selector');
    if (selectorContainer) {
      this.metricSelector = new MetricSelector(selectorContainer);
      this.metricSelector.onSelectionChanged = (selected) => {
        this._updateMetricGraphs(selected);
      };
    }

    // Create time range selector
    this._createTimeRangeSelector();

    // Set up metric graphs container
    const graphsContainer = document.getElementById('metric-graphs-container');
    if (graphsContainer) {
      this.metricGraphsContainer = graphsContainer;
    }
  }

  _instantiateSQLTerminal() {
    const container = this.root.querySelector('#sql-terminal-container');
    if (container) {
      // Create query executor for SQL terminal
      const queryExecutor = {
        execute: async (sql, params) => {
          const result = await this.workerClient.query(sql, params);
          return { rows: result.rows ?? [] };
        },
      };

      this.sqlTerminal = new SQLTerminal(container, {
        queryExecutor,
        onQueryResult: (result) => {
          // Optional: log query results
          console.log('SQL query executed:', result);
        },
      });
      this.sqlTerminal.render();
    }
  }

  _createTimeRangeSelector() {
    const container = this.root.querySelector('#metric-time-range');
    if (!container) return;

    this.timeRangeSelector = new TimeRangeSelector(container, {
      currentPreset: this._loadState('timeRangePreset') || 'last1h',
      onTimeRangeChanged: (range) => {
        this._saveState('timeRangePreset', range.preset);
        for (const graph of this.metricGraphs.values()) {
          graph.setTimeRange(range);
        }
      },
    });
    this.timeRangeSelector.render();
  }

  async _refreshMetrics() {
    if (!this.metricQuery || !this.state.tables.size) {
      return;
    }

    try {
      const tableNames = Array.from(this.state.tables.values());
      if (tableNames.length === 0) {
        return;
      }

      // Get available metrics from all tables
      const availableMetrics = await this.metricQuery.getAvailableMetricsFromTables(tableNames);
      if (this.metricSelector) {
        this.metricSelector.setAvailableMetrics(availableMetrics);
      }

      // Refresh selected metrics
      const selected = this.metricSelector?.getSelectedMetrics() || availableMetrics.slice(0, 5);
      await this._updateMetricGraphs(selected);
    } catch (error) {
      console.error('Failed to refresh metrics', error);
      this._log(`Metric refresh failed: ${error.message}`);
    }
  }

  async _updateMetricGraphs(selectedMetrics) {
    if (!this.metricGraphsContainer || !this.metricQuery) {
      return;
    }

    const tableNames = Array.from(this.state.tables.values());
    if (tableNames.length === 0) {
      return;
    }

    // Remove graphs for unselected metrics
    for (const [metricName, graph] of this.metricGraphs.entries()) {
      if (!selectedMetrics.includes(metricName)) {
        graph.destroy();
        this.metricGraphs.delete(metricName);
        const container = document.getElementById(`metric-graph-${metricName}`);
        container?.remove();
      }
    }

    // Create/update graphs for selected metrics
    for (const metricName of selectedMetrics) {
      try {
        // Query metrics for this metric name from all tables
        const metrics = await this.metricQuery.queryMetricsFromTables(tableNames, {
          metricName,
          timeRange: this._getCurrentTimeRange(),
        });

        if (metrics.length === 0) continue;

        let graph = this.metricGraphs.get(metricName);
        if (!graph) {
          // Create new graph container
          const graphContainer = document.createElement('div');
          graphContainer.id = `metric-graph-${metricName}`;
          graphContainer.className = 'metric-graph-container';
          this.metricGraphsContainer.appendChild(graphContainer);

          graph = new MetricGraph(graphContainer, {
            metricName,
            maxDataPoints: configManager.get('maxGraphPoints'),
          });
          graph.onTimeRangeChanged = (range) => {
            // Update all graphs when time range changes
            for (const g of this.metricGraphs.values()) {
              g.setTimeRange(range);
            }
          };
          this.metricGraphs.set(metricName, graph);
        }

        graph.updateMetric(metricName, metrics);
      } catch (error) {
        console.error(`Failed to update graph for ${metricName}:`, error);
      }
    }
  }

  _getCurrentTimeRange() {
    if (this.timeRangeSelector) {
      return this.timeRangeSelector.getTimeRange();
    }
    // Fallback if time range selector not initialized
    const preset = 'last1h';
    const now = Date.now() * 1_000_000;
    const ranges = {
      last5m: 5 * 60 * 1_000_000_000,
      last15m: 15 * 60 * 1_000_000_000,
      last1h: 60 * 60 * 1_000_000_000,
      last6h: 6 * 60 * 60 * 1_000_000_000,
      last24h: 24 * 60 * 60 * 1_000_000_000,
    };
    const duration = ranges[preset] || ranges.last1h;
    return {
      start: now - duration,
      end: now,
      preset,
    };
  }

  /**
   * Pause live stream (stop file watching)
   */
  pauseStream() {
    this.isPaused = true;
    this.fileWatcher.stopWatching();
    this._setStatus('Paused');
    this._saveState('isPaused', true);
    this._log('Live stream paused');
  }

  /**
   * Resume live stream (restart file watching)
   */
  resumeStream() {
    if (!this.state.directory) {
      this._log('No directory selected. Cannot resume.');
      return;
    }

    this.isPaused = false;
    this.fileWatcher.startWatching(this.state.directory, configManager.get('pollingIntervalMs'));
    this._setStatus('Watching for updates');
    this._saveState('isPaused', false);
    this._log('Live stream resumed');
  }

  /**
   * Handle search query
   * @private
   */
  _handleSearch(query) {
    if (!query || query.trim() === '') {
      return;
    }

    const searchTerm = query.trim().toLowerCase();

    // Search in traces
    if (this.currentView === 'traces' && this.traceList) {
      // Filter traces by search term
      const filters = {
        ...this.activeFilters,
        traceId: searchTerm,
      };
      this.traceList.applyFilters(filters);
    }

    // Search in metrics
    if (this.currentView === 'metrics' && this.metricSelector) {
      // Filter available metrics
      const available = this.metricSelector.availableMetrics || [];
      const matching = available.filter((m) => m.toLowerCase().includes(searchTerm));
      if (matching.length > 0) {
        this.metricSelector.setSelectedMetrics(matching);
      }
    }
  }

  /**
   * Handle search clear
   * @private
   */
  _handleSearchClear() {
    if (this.currentView === 'traces' && this.traceList) {
      this.traceList.applyFilters(this.activeFilters);
    }
  }

  /**
   * Attach keyboard navigation handlers
   * @private
   */
  _attachKeyboardHandlers() {
    document.addEventListener('keydown', (e) => {
      // Don't interfere with input fields
      if (e.target.tagName === 'INPUT' || e.target.tagName === 'TEXTAREA') {
        return;
      }

      // Keyboard shortcuts
      switch (e.key) {
        case '1':
          if (e.ctrlKey || e.metaKey) {
            e.preventDefault();
            this._switchView('traces');
          }
          break;
        case '2':
          if (e.ctrlKey || e.metaKey) {
            e.preventDefault();
            this._switchView('metrics');
          }
          break;
        case 'p':
          if (e.ctrlKey || e.metaKey) {
            e.preventDefault();
            if (this.isPaused) {
              this.resumeStream();
            } else {
              this.pauseStream();
            }
          }
          break;
        case '/':
          if (!e.ctrlKey && !e.metaKey) {
            e.preventDefault();
            this.searchComponent?.focus();
          }
          break;
        case 'Escape':
          this.searchComponent?.clear();
          break;
      }
    });
  }

  /**
   * Save state to localStorage
   * @private
   */
  _saveState(key, value) {
    try {
      const state = JSON.parse(localStorage.getItem('dashboard_state') || '{}');
      state[key] = value;
      localStorage.setItem('dashboard_state', JSON.stringify(state));
    } catch (error) {
      console.warn('Failed to save state:', error);
    }
  }

  /**
   * Load state from localStorage
   * @private
   */
  _loadState(key) {
    try {
      const state = JSON.parse(localStorage.getItem('dashboard_state') || '{}');
      return state[key];
    } catch (error) {
      console.warn('Failed to load state:', error);
      return null;
    }
  }

  /**
   * Check browser compatibility
   * @private
   * @returns {boolean} True if browser is compatible
   */
  _checkBrowserCompatibility() {
    // Check for required features
    const requiredFeatures = {
      webAssembly: typeof WebAssembly !== 'undefined',
      workers: typeof Worker !== 'undefined',
      localStorage: typeof Storage !== 'undefined' && typeof localStorage !== 'undefined',
      fileSystem:
        typeof FileReader !== 'undefined' || typeof window.showDirectoryPicker !== 'undefined',
    };

    const missingFeatures = Object.entries(requiredFeatures)
      .filter(([_, supported]) => !supported)
      .map(([feature]) => feature);

    if (missingFeatures.length > 0) {
      console.warn('Missing required browser features:', missingFeatures);
      return false;
    }

    return true;
  }

  /**
   * Show browser incompatibility message
   * @private
   */
  _showBrowserIncompatibility() {
    this.root.innerHTML = `
      <div style="padding: 2rem; text-align: center;">
        <h1>Browser Not Supported</h1>
        <p>Your browser does not support the required features for this dashboard.</p>
        <p>Please use a modern browser with WebAssembly support (Chrome, Firefox, Safari, or Edge - latest 2 versions).</p>
        <p>Required features: WebAssembly, Web Workers, LocalStorage, File System Access API</p>
      </div>
    `;
  }

  /**
   * Set up error boundary for unhandled errors
   * @private
   */
  _setupErrorBoundary() {
    // Global error handler
    window.addEventListener('error', (event) => {
      console.error('Unhandled error:', event.error);
      this._handleError(event.error, 'Unhandled Error');
      event.preventDefault();
    });

    // Unhandled promise rejection handler
    window.addEventListener('unhandledrejection', (event) => {
      console.error('Unhandled promise rejection:', event.reason);
      this._handleError(event.reason, 'Unhandled Promise Rejection');
      event.preventDefault();
    });
  }

  /**
   * Handle errors gracefully
   * @private
   * @param {Error} error - The error object
   * @param {string} context - Context where error occurred
   */
  _handleError(error, context) {
    if (this.loadingComponent) {
      this.loadingComponent.showError(`${context}: ${error.message || String(error)}`);
    } else {
      // Fallback error display
      const errorDiv = document.createElement('div');
      errorDiv.className = 'error-message';
      errorDiv.style.cssText =
        'padding: 1rem; background: #fee; border: 1px solid #fcc; margin: 1rem;';
      errorDiv.innerHTML = `
        <strong>${context}</strong><br>
        ${error.message || String(error)}
      `;
      this.root.prepend(errorDiv);
    }
  }

  /**
   * Clean up old data to enforce limits
   * @private
   */
  _enforceDataLimits() {
    // Clean up old traces if limit exceeded
    if (
      this.traceList &&
      this.traceList.getTraceCount &&
      this.traceList.getTraceCount() > this.maxTraces
    ) {
      const excess = this.traceList.getTraceCount() - this.maxTraces;
      this.traceList.removeOldestTraces(excess);
    }

    // Clean up old graph points if limit exceeded
    this.metricGraphs.forEach((graph) => {
      if (graph.getPointCount && graph.getPointCount() > this.maxGraphPoints) {
        const excess = graph.getPointCount() - this.maxGraphPoints;
        graph.removeOldestPoints(excess);
      }
    });

    // Clean up old tables (handled by DuckDBClient LRU cache)
    // This is already implemented in duckdb-client.js
  }

  /**
   * Initialize settings component
   * @private
   */
  _initializeSettings() {
    // Create settings container (hidden by default)
    const settingsContainer = document.createElement('div');
    settingsContainer.id = 'settings-container';
    settingsContainer.style.display = 'none';
    this.root.querySelector('.app-main')?.appendChild(settingsContainer);

    this.settingsComponent = new Settings(settingsContainer, {
      pollingInterval: this.pollingInterval || 1000,
      maxTraces: this.maxTraces || 10000,
      maxGraphPoints: this.maxGraphPoints || 10000,
      maxLoadedTables: this.maxLoadedTables || 50,
      onSettingsChanged: (settings) => {
        this._applySettings(settings);
      },
    });
    this.settingsComponent.render();
  }

  /**
   * Toggle settings panel visibility
   * @private
   */
  _toggleSettings() {
    const container = document.getElementById('settings-container');
    if (container) {
      container.style.display = container.style.display === 'none' ? 'block' : 'none';
    }
  }

  /**
   * Apply settings changes
   * @private
   * @param {Object} settings - New settings
   */
  _applySettings(settings) {
    this.pollingInterval = settings.pollingInterval;
    this.maxTraces = settings.maxTraces;
    this.maxGraphPoints = settings.maxGraphPoints;
    this.maxLoadedTables = settings.maxLoadedTables;

    // Save to localStorage
    this._saveState('pollingInterval', this.pollingInterval);
    this._saveState('maxTraces', this.maxTraces);
    this._saveState('maxGraphPoints', this.maxGraphPoints);
    this._saveState('maxLoadedTables', this.maxLoadedTables);

    // Update file watcher polling interval
    if (this.fileWatcher && this.fileWatcher.setPollingInterval) {
      this.fileWatcher.setPollingInterval(this.pollingInterval);
    }

    // Enforce new limits
    this._enforceDataLimits();

    this._setStatus('Settings updated');
  }
}
