/**
 * SQL Terminal component for running arbitrary SQL queries against DuckDB
 * Provides an advanced interface for power users to query the data directly
 */
export class SQLTerminal {
  constructor(container, options = {}) {
    if (!container) {
      throw new Error('SQLTerminal requires a container element');
    }

    this.container = container;
    this.queryExecutor = options.queryExecutor || null;
    this.onQueryResult = options.onQueryResult || null;
    this.queryHistory = this._loadHistory();
    this.historyIndex = -1;
  }

  /**
   * Render the SQL terminal UI
   */
  render() {
    this.container.innerHTML = `
      <div class="sql-terminal">
        <div class="sql-terminal__header">
          <h3>SQL Terminal</h3>
          <p class="sql-terminal__description">
            Run SQL queries directly against the loaded data. Use this for advanced analysis.
            <br>
            <strong>Available tables:</strong> <span id="sql-terminal-tables">Loading...</span>
            <br>
            <small style="color: #64748b;">Note: Use table names (e.g., <code>otlp_traces_20251126_004832_0000_arrow</code>), not file names. Table names are shown above.</small>
          </p>
        </div>
        <div class="sql-terminal__editor">
          <textarea 
            id="sql-terminal-input" 
            class="sql-terminal__input"
            placeholder="SELECT * FROM otlp_traces_20251126_004832_0000_arrow LIMIT 10;"
            rows="8"
            spellcheck="false"
          ></textarea>
          <div class="sql-terminal__toolbar">
            <button id="sql-terminal-execute" class="sql-terminal__button sql-terminal__button--primary">
              Execute (Ctrl+Enter)
            </button>
            <button id="sql-terminal-clear" class="sql-terminal__button">
              Clear
            </button>
            <button id="sql-terminal-history" class="sql-terminal__button">
              History (↑/↓)
            </button>
          </div>
        </div>
        <div class="sql-terminal__results">
          <div id="sql-terminal-status" class="sql-terminal__status"></div>
          <div id="sql-terminal-output" class="sql-terminal__output"></div>
        </div>
      </div>
    `;

    this._attachEventHandlers();
    this._updateTableList();
  }

  /**
   * Attach event handlers
   */
  _attachEventHandlers() {
    const input = this.container.querySelector('#sql-terminal-input');
    const executeBtn = this.container.querySelector('#sql-terminal-execute');
    const clearBtn = this.container.querySelector('#sql-terminal-clear');
    const historyBtn = this.container.querySelector('#sql-terminal-history');

    // Execute query
    executeBtn.addEventListener('click', () => this._executeQuery());
    
    // Clear input
    clearBtn.addEventListener('click', () => {
      input.value = '';
      input.focus();
    });

    // Keyboard shortcuts
    input.addEventListener('keydown', (e) => {
      // Ctrl+Enter or Cmd+Enter to execute
      if ((e.ctrlKey || e.metaKey) && e.key === 'Enter') {
        e.preventDefault();
        this._executeQuery();
      }
      // Arrow up/down for history
      else if (e.key === 'ArrowUp' && e.ctrlKey) {
        e.preventDefault();
        this._navigateHistory(-1);
      }
      else if (e.key === 'ArrowDown' && e.ctrlKey) {
        e.preventDefault();
        this._navigateHistory(1);
      }
    });

    // Tab key for indentation
    input.addEventListener('keydown', (e) => {
      if (e.key === 'Tab' && !e.ctrlKey && !e.metaKey) {
        e.preventDefault();
        const start = input.selectionStart;
        const end = input.selectionEnd;
        input.value = input.value.substring(0, start) + '  ' + input.value.substring(end);
        input.selectionStart = input.selectionEnd = start + 2;
      }
    });
  }

  /**
   * Execute the SQL query
   */
  async _executeQuery() {
    const input = this.container.querySelector('#sql-terminal-input');
    const sql = input.value.trim();
    const statusEl = this.container.querySelector('#sql-terminal-status');
    const outputEl = this.container.querySelector('#sql-terminal-output');

    if (!sql) {
      statusEl.textContent = 'Please enter a SQL query';
      statusEl.className = 'sql-terminal__status sql-terminal__status--warning';
      return;
    }

    if (!this.queryExecutor) {
      statusEl.textContent = 'Query executor not available';
      statusEl.className = 'sql-terminal__status sql-terminal__status--error';
      return;
    }

    // Add to history
    this._addToHistory(sql);

    // Show executing status
    statusEl.textContent = 'Executing query...';
    statusEl.className = 'sql-terminal__status sql-terminal__status--info';
    outputEl.innerHTML = '';

    try {
      const startTime = performance.now();
      const result = await this.queryExecutor.execute(sql, []);
      const duration = (performance.now() - startTime).toFixed(2);

      // Display results
      if (result.rows && result.rows.length > 0) {
        this._displayTable(result.rows, duration);
        statusEl.textContent = `Query executed successfully in ${duration}ms. Returned ${result.rows.length} row(s).`;
        statusEl.className = 'sql-terminal__status sql-terminal__status--success';
      } else {
        statusEl.textContent = `Query executed successfully in ${duration}ms. No rows returned.`;
        statusEl.className = 'sql-terminal__status sql-terminal__status--success';
        outputEl.innerHTML = '<div class="sql-terminal__empty">No results</div>';
      }

      // Notify callback if provided
      if (this.onQueryResult) {
        this.onQueryResult({ sql, result, duration: parseFloat(duration) });
      }
    } catch (error) {
      statusEl.textContent = `Error: ${error.message}`;
      statusEl.className = 'sql-terminal__status sql-terminal__status--error';
      outputEl.innerHTML = `
        <div class="sql-terminal__error">
          <pre>${error.message}</pre>
        </div>
      `;
    }
  }

  /**
   * Display query results as a table
   */
  _displayTable(rows, duration) {
    const outputEl = this.container.querySelector('#sql-terminal-output');
    
    if (rows.length === 0) {
      outputEl.innerHTML = '<div class="sql-terminal__empty">No results</div>';
      return;
    }

    // Get column names from first row
    const columns = Object.keys(rows[0]);
    
    // Build table HTML
    let html = '<div class="sql-terminal__table-container">';
    html += '<table class="sql-terminal__table">';
    
    // Header
    html += '<thead><tr>';
    for (const col of columns) {
      html += `<th>${this._escapeHtml(col)}</th>`;
    }
    html += '</tr></thead>';
    
    // Body
    html += '<tbody>';
    for (const row of rows) {
      html += '<tr>';
      for (const col of columns) {
        const value = row[col];
        const displayValue = value === null || value === undefined 
          ? '<em>NULL</em>' 
          : this._escapeHtml(String(value));
        html += `<td>${displayValue}</td>`;
      }
      html += '</tr>';
    }
    html += '</tbody>';
    html += '</table>';
    html += '</div>';

    outputEl.innerHTML = html;
  }

  /**
   * Update the list of available tables
   */
  async _updateTableList() {
    const tablesEl = this.container.querySelector('#sql-terminal-tables');
    if (!tablesEl || !this.queryExecutor) {
      return;
    }

    try {
      // Query DuckDB's information schema to get table names
      const result = await this.queryExecutor.execute(
        "SELECT table_name FROM information_schema.tables WHERE table_schema = 'main' AND table_type = 'BASE TABLE' ORDER BY table_name",
        []
      );

      if (result.rows && result.rows.length > 0) {
        const tableNames = result.rows.map(row => row.table_name);
        tablesEl.textContent = tableNames.join(', ') || 'None';
      } else {
        tablesEl.textContent = 'None (load data first)';
      }
    } catch (error) {
      // Handle initialization errors gracefully
      if (error.message && error.message.includes('not initialized')) {
        tablesEl.textContent = 'Initializing...';
        // Retry after a short delay
        setTimeout(() => this._updateTableList(), 1000);
      } else {
        tablesEl.textContent = 'Error loading tables';
        console.error('Failed to load table list:', error);
      }
    }
  }

  /**
   * Add query to history
   */
  _addToHistory(sql) {
    // Remove if already exists
    this.queryHistory = this.queryHistory.filter(q => q !== sql);
    // Add to front
    this.queryHistory.unshift(sql);
    // Keep only last 50
    if (this.queryHistory.length > 50) {
      this.queryHistory = this.queryHistory.slice(0, 50);
    }
    this._saveHistory();
    this.historyIndex = -1;
  }

  /**
   * Navigate query history
   */
  _navigateHistory(direction) {
    if (this.queryHistory.length === 0) {
      return;
    }

    const input = this.container.querySelector('#sql-terminal-input');
    
    if (this.historyIndex === -1) {
      // Save current query as "current"
      this.currentQuery = input.value;
    }

    this.historyIndex += direction;
    
    if (this.historyIndex < 0) {
      this.historyIndex = -1;
      input.value = this.currentQuery || '';
    } else if (this.historyIndex >= this.queryHistory.length) {
      this.historyIndex = this.queryHistory.length - 1;
    } else {
      input.value = this.queryHistory[this.historyIndex];
    }
  }

  /**
   * Load query history from localStorage
   */
  _loadHistory() {
    try {
      const stored = localStorage.getItem('sql-terminal-history');
      return stored ? JSON.parse(stored) : [];
    } catch {
      return [];
    }
  }

  /**
   * Save query history to localStorage
   */
  _saveHistory() {
    try {
      localStorage.setItem('sql-terminal-history', JSON.stringify(this.queryHistory));
    } catch {
      // Ignore storage errors
    }
  }

  /**
   * Escape HTML to prevent XSS
   */
  _escapeHtml(text) {
    const div = document.createElement('div');
    div.textContent = text;
    return div.innerHTML;
  }

  /**
   * Update table list (call this when tables change)
   */
  refreshTableList() {
    this._updateTableList();
  }
}

