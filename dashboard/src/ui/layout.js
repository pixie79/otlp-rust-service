/**
 * Layout component for main dashboard layout structure
 * Implements responsive layout and accessibility features per contracts/ui-components.md
 */
export class Layout {
  constructor(container) {
    if (!container) {
      throw new Error('Layout requires a container element');
    }

    this.container = container;
    this.currentView = 'traces';
  }

  /**
   * Render the main layout structure
   */
  render() {
    this.container.innerHTML = `
      <header class="app-header panel" role="banner">
        <div>
          <h1>OTLP Realtime Dashboard</h1>
          <p class="subtitle">Monitor Arrow IPC traces and metrics directly in your browser.</p>
        </div>
        <div style="display: flex; gap: 1rem; align-items: center;">
          <button id="settings-button" class="ghost" aria-label="Open settings">⚙️ Settings</button>
          <div class="status-line" id="status-line" role="status" aria-live="polite" aria-atomic="true"></div>
        </div>
      </header>
      <main class="app-main" role="main">
        <section class="panel intro-panel" id="intro-panel" aria-label="Introduction">
          <p>Select the OTLP output directory to start streaming Arrow IPC files.</p>
          <button class="primary" id="select-directory" aria-label="Choose directory to monitor">Choose Directory</button>
        </section>
        <nav class="view-nav" role="tablist" aria-label="View navigation">
          <button 
            class="view-nav__button active" 
            id="nav-traces" 
            data-view="traces"
            role="tab"
            aria-selected="true"
            aria-controls="trace-panel"
            tabindex="0"
          >
            Traces
          </button>
          <button 
            class="view-nav__button" 
            id="nav-metrics" 
            data-view="metrics"
            role="tab"
            aria-selected="false"
            aria-controls="metric-panel"
            tabindex="-1"
          >
            Metrics
          </button>
        </nav>
        <section 
          class="panel trace-panel" 
          id="trace-panel" 
          data-testid="trace-panel"
          role="tabpanel"
          aria-labelledby="nav-traces"
        >
          <div class="trace-panel__header">
            <div>
              <h2>Traces</h2>
              <p class="subtitle">Live tail viewer with filtering and detail pane.</p>
            </div>
            <button class="ghost" id="refresh-traces" aria-label="Refresh traces">Refresh</button>
          </div>
          <div id="trace-filter" role="search" aria-label="Trace filters"></div>
          <div class="trace-panel__content">
            <div id="trace-list" role="list" aria-label="Trace list"></div>
            <div id="trace-detail" role="complementary" aria-label="Trace details"></div>
          </div>
        </section>
        <section 
          class="panel metric-panel" 
          id="metric-panel" 
          style="display: none;"
          role="tabpanel"
          aria-labelledby="nav-metrics"
        >
          <div class="metric-panel__header">
            <div>
              <h2>Metrics</h2>
              <p class="subtitle">Real-time time-series graphs with interactive features.</p>
            </div>
            <button class="ghost" id="refresh-metrics" aria-label="Refresh metrics">Refresh</button>
          </div>
          <div id="metric-selector" role="group" aria-label="Metric selection"></div>
          <div id="metric-time-range" role="group" aria-label="Time range selection"></div>
          <div id="metric-graphs-container" role="group" aria-label="Metric graphs"></div>
        </section>
        <section class="panel muted" id="log-panel" role="log" aria-label="Activity log">
          <h2>Activity</h2>
          <ul id="log-list" role="list"></ul>
        </section>
      </main>
    `;
  }

  /**
   * Switch to a different view
   */
  switchView(view) {
    this.currentView = view;
    const tracePanel = this.container.querySelector('#trace-panel');
    const metricPanel = this.container.querySelector('#metric-panel');
    const navTraces = this.container.querySelector('#nav-traces');
    const navMetrics = this.container.querySelector('#nav-metrics');

    if (view === 'traces') {
      tracePanel.style.display = 'block';
      metricPanel.style.display = 'none';
      navTraces?.classList.add('active');
      navMetrics?.classList.remove('active');
      navTraces?.setAttribute('aria-selected', 'true');
      navMetrics?.setAttribute('aria-selected', 'false');
      navTraces?.setAttribute('tabindex', '0');
      navMetrics?.setAttribute('tabindex', '-1');
    } else {
      tracePanel.style.display = 'none';
      metricPanel.style.display = 'block';
      navTraces?.classList.remove('active');
      navMetrics?.classList.add('active');
      navTraces?.setAttribute('aria-selected', 'false');
      navMetrics?.setAttribute('aria-selected', 'true');
      navTraces?.setAttribute('tabindex', '-1');
      navMetrics?.setAttribute('tabindex', '0');
    }
  }

  /**
   * Update status badge
   */
  setStatus(statusText) {
    const statusLine = this.container.querySelector('#status-line');
    if (statusLine) {
      statusLine.innerHTML = `<span class="status-badge">${this._escapeHtml(statusText)}</span>`;
      statusLine.setAttribute('aria-label', `Status: ${statusText}`);
    }
  }

  /**
   * Get container element for a specific section
   */
  getSection(sectionId) {
    return this.container.querySelector(`#${sectionId}`);
  }

  /**
   * Escape HTML to prevent XSS
   * @private
   */
  _escapeHtml(value) {
    return String(value ?? '')
      .replace(/&/g, '&amp;')
      .replace(/</g, '&lt;')
      .replace(/>/g, '&gt;')
      .replace(/"/g, '&quot;')
      .replace(/'/g, '&#39;');
  }
}

