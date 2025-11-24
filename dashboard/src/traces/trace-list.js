import { sortByStartTimeDesc } from './trace-entry.js';

const escapeHtml = (value) =>
  String(value ?? '')
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;')
    .replace(/'/g, '&#39;');

const defaultFilters = () => ({
  traceId: '',
  serviceName: '',
  spanName: '',
  errorOnly: false,
  minDuration: null,
  maxDuration: null,
});

/**
 * TraceList component for displaying and managing trace entries
 * Implements virtual scrolling for performance with large datasets
 */
export class TraceList {
  constructor(container, options = {}) {
    if (!container) {
      throw new Error('TraceList requires a container element');
    }

    this.container = container;
    this.rowHeight = options.rowHeight ?? 56;
    this.windowSize = options.windowSize ?? 40;
    this.virtualBuffer = options.virtualBuffer ?? 10;
    this.traces = [];
    this.filteredTraces = [];
    this.renderedRange = { start: 0, end: 0 };
    this.selectedIndex = -1;
    this.filters = defaultFilters();
    this.onTraceSelected = () => {};
    this.onFilterChanged = () => {};
    this.unsubscribeWorker = null;

    this._buildDom();
  }

  _buildDom() {
    this.container.classList.add('trace-list');
    this.viewport = document.createElement('div');
    this.viewport.className = 'trace-list__viewport';

    this.spacer = document.createElement('div');
    this.spacer.className = 'trace-list__spacer';

    this.viewport.appendChild(this.spacer);
    this.container.appendChild(this.viewport);
    this.viewport.addEventListener('scroll', () => this._renderWindow());
  }

  setTraces(traces = []) {
    this.traces = [...traces].sort(sortByStartTimeDesc);
    this.selectedIndex = Math.min(this.selectedIndex, this.traces.length - 1);
    if (this.selectedIndex < 0) {
      this.selectedIndex = -1;
    }
    this.viewport.scrollTop = 0;
    this._applyFilters();
  }

  applyFilters(filters = {}) {
    this.filters = { ...this.filters, ...filters };
    this._applyFilters();
    this.onFilterChanged?.(this.filters);
  }

  getSelectedTrace() {
    if (this.selectedIndex < 0 || this.selectedIndex >= this.filteredTraces.length) {
      return null;
    }
    return this.filteredTraces[this.selectedIndex];
  }

  selectTraceByIndex(index) {
    if (index < 0 || index >= this.filteredTraces.length) {
      return;
    }
    this.selectedIndex = index;
    this._renderWindow();
    this.onTraceSelected?.(this.filteredTraces[index]);
  }

  bindWorker(workerClient) {
    this.unsubscribeWorker?.();
    if (!workerClient || typeof workerClient.subscribe !== 'function') {
      return;
    }
    this.unsubscribeWorker = workerClient.subscribe('TRACE_BATCH', ({ traces }) => {
      if (Array.isArray(traces)) {
        this.setTraces(traces);
      }
    });
  }

  _applyFilters() {
    const filters = this.filters;
    this.filteredTraces = this.traces.filter((trace) => {
      if (filters.traceId && !trace.traceId.startsWith(filters.traceId)) {
        return false;
      }
      if (
        filters.serviceName &&
        !trace.serviceName?.toLowerCase().includes(filters.serviceName.toLowerCase())
      ) {
        return false;
      }
      if (filters.spanName && !trace.name?.toLowerCase().includes(filters.spanName.toLowerCase())) {
        return false;
      }
      if (filters.errorOnly && !trace.error) {
        return false;
      }
      if (filters.minDuration != null && trace.duration < filters.minDuration) {
        return false;
      }
      if (filters.maxDuration != null && trace.duration > filters.maxDuration) {
        return false;
      }
      return true;
    });

    this.selectedIndex = Math.min(this.selectedIndex, this.filteredTraces.length - 1);
    this.spacer.style.height = `${this.filteredTraces.length * this.rowHeight}px`;
    this._renderWindow(true);
  }

  _renderWindow(force = false) {
    const scrollTop = this.viewport.scrollTop;
    const startIndex = Math.max(0, Math.floor(scrollTop / this.rowHeight) - this.virtualBuffer);
    const endIndex = Math.min(
      this.filteredTraces.length,
      startIndex + this.windowSize + this.virtualBuffer * 2
    );

    if (!force && startIndex === this.renderedRange.start && endIndex === this.renderedRange.end) {
      return;
    }

    this.renderedRange = { start: startIndex, end: endIndex };
    const fragment = document.createDocumentFragment();

    for (let i = startIndex; i < endIndex; i += 1) {
      fragment.appendChild(this._renderRow(this.filteredTraces[i], i));
    }

    this.viewport.querySelectorAll('.trace-row').forEach((node) => node.remove());
    this.viewport.appendChild(fragment);
  }

  _renderRow(trace, index) {
    const duration = typeof trace.duration === 'number' ? trace.duration : 0;
    const startTime =
      typeof trace.startTime === 'number' ? trace.startTime : Date.now() * 1_000_000;
    const row = document.createElement('div');
    row.className = 'trace-row';
    row.style.position = 'absolute';
    row.style.top = `${index * this.rowHeight}px`;
    row.style.height = `${this.rowHeight}px`;
    row.dataset.index = String(index);
    row.innerHTML = `
      <div class="trace-row__column span-name">${escapeHtml(trace.name)}</div>
      <div class="trace-row__column service-name">${escapeHtml(trace.serviceName)}</div>
      <div class="trace-row__column duration">${duration.toFixed(2)} ms</div>
      <div class="trace-row__column status ${escapeHtml(trace.statusCode)}">${escapeHtml(
        trace.statusCode.toUpperCase()
      )}</div>
      <div class="trace-row__column timestamp">${new Date(startTime / 1_000_000).toLocaleTimeString()}</div>
    `;

    if (index === this.selectedIndex) {
      row.classList.add('selected');
    }

    row.addEventListener('click', () => {
      this.selectedIndex = index;
      this._renderWindow(true);
      this.onTraceSelected?.(trace);
    });

    return row;
  }

  /**
   * Get the current number of traces
   * @returns {number} Number of traces
   */
  getTraceCount() {
    return this.traces.length;
  }

  /**
   * Remove the oldest traces
   * @param {number} count - Number of traces to remove
   */
  removeOldestTraces(count) {
    if (count <= 0 || count > this.traces.length) return;

    // Remove oldest traces (they are sorted by start time descending)
    this.traces = this.traces.slice(0, this.traces.length - count);
    this._applyFilters();
    this._renderWindow(true);
  }
}
