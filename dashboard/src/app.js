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

/**
 * Status badge helper function
 * @param {string} text - Text to display in badge
 * @returns {string} HTML string for status badge
 */
const statusBadge = (text) => `<span class="status-badge">${text}</span>`;

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
    
    await this.workerClient.init();
    this.traceQuery = new TraceQuery({
      execute: async (sql, params) => {
        const result = await this.workerClient.query(sql, params);
        return { rows: result.rows ?? [] };
      },
    });
    this.metricQuery = new MetricQuery({
      execute: async (sql, params) => {
        const result = await this.workerClient.query(sql, params);
        return { rows: result.rows ?? [] };
      },
    });
    
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

    this.fileWatcher.onNewFile = (fileHandle, metadata) => {
      this._log(`Detected new file: ${metadata?.name ?? fileHandle.name}`);
      this._ingestFile(fileHandle, metadata);
    };

    this.fileWatcher.onFileChanged = (fileHandle, metadata) => {
      this._log(`Detected modified file: ${metadata?.name ?? fileHandle.name}`);
      this._ingestFile(fileHandle, metadata);
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
    } else {
      this._refreshMetrics();
    }
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
      
      // Check file size and use appropriate reading strategy
      const fileMetadata = await this.fileReader.getFileMetadata(fileHandle);
      const fileSizeMB = fileMetadata.size / (1024 * 1024);
      
      if (fileSizeMB > 50) {
        this._log(`Reading large file ${name} (${fileSizeMB.toFixed(2)}MB) - this may take a moment...`);
      }

      // Read file with chunked reading for large files
      const buffer = await this.fileReader.readFile(fileHandle, name, {
        chunkSize: 10 * 1024 * 1024, // 10MB chunks
        maxSize: 200 * 1024 * 1024, // 200MB max
      });

      const { tableName } = await this.workerClient.registerFile(name, buffer);
      this.state.tables.set(name, tableName);
      this._log(`Registered ${name} as ${tableName} in DuckDB.`);

      // Check if we need to clean up old tables
      const maxFiles = configManager.get('maxLoadedFiles');
      if (this.state.tables.size > maxFiles) {
        await this._cleanupOldTables(maxFiles);
      }

      await this._refreshTraces();
      await this._refreshMetrics();
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
      fileSystem: typeof FileReader !== 'undefined' || typeof window.showDirectoryPicker !== 'undefined',
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
      this.loadingComponent.showError(
        `${context}: ${error.message || String(error)}`
      );
    } else {
      // Fallback error display
      const errorDiv = document.createElement('div');
      errorDiv.className = 'error-message';
      errorDiv.style.cssText = 'padding: 1rem; background: #fee; border: 1px solid #fcc; margin: 1rem;';
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
    if (this.traceList && this.traceList.getTraceCount && this.traceList.getTraceCount() > this.maxTraces) {
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
}
