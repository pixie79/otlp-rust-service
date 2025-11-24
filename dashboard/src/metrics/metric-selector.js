/**
 * MetricSelector component for selecting which metrics to display
 * Implements the MetricSelector interface from contracts/ui-components.md
 */
export class MetricSelector {
  constructor(container) {
    if (!container) {
      throw new Error('MetricSelector requires a container element');
    }

    this.container = container;
    this.availableMetrics = [];
    this.selectedMetrics = new Set();
    this.onSelectionChanged = null;
  }

  /**
   * Set available metrics
   * @param {Array<string>} metrics - Array of metric names
   */
  setAvailableMetrics(metrics) {
    this.availableMetrics = [...metrics];
    this._render();
  }

  /**
   * Get selected metrics
   * @returns {Array<string>} Array of selected metric names
   */
  getSelectedMetrics() {
    return Array.from(this.selectedMetrics);
  }

  /**
   * Set selected metrics
   * @param {Array<string>} metrics - Array of metric names to select
   */
  setSelectedMetrics(metrics) {
    this.selectedMetrics = new Set(metrics);
    this._render();
  }

  /**
   * Render the selector UI
   * @private
   */
  _render() {
    if (this.availableMetrics.length === 0) {
      this.container.innerHTML = '<p class="metric-selector__empty">No metrics available</p>';
      return;
    }

    const items = this.availableMetrics
      .map(
        (metric) => `
      <label class="metric-selector__item">
        <input
          type="checkbox"
          value="${this._escapeHtml(metric)}"
          ${this.selectedMetrics.has(metric) ? 'checked' : ''}
        />
        <span>${this._escapeHtml(metric)}</span>
      </label>
    `,
      )
      .join('');

    this.container.innerHTML = `
      <div class="metric-selector">
        <div class="metric-selector__header">
          <h3>Select Metrics</h3>
          <div class="metric-selector__actions">
            <button class="button-ghost" id="select-all-metrics">Select All</button>
            <button class="button-ghost" id="deselect-all-metrics">Deselect All</button>
          </div>
        </div>
        <div class="metric-selector__list">
          ${items}
        </div>
      </div>
    `;

    this._attachEventHandlers();
  }

  /**
   * Attach event handlers for checkbox changes
   * @private
   */
  _attachEventHandlers() {
    const checkboxes = this.container.querySelectorAll('input[type="checkbox"]');
    checkboxes.forEach((checkbox) => {
      checkbox.addEventListener('change', () => {
        if (checkbox.checked) {
          this.selectedMetrics.add(checkbox.value);
        } else {
          this.selectedMetrics.delete(checkbox.value);
        }
        if (this.onSelectionChanged) {
          this.onSelectionChanged(this.getSelectedMetrics());
        }
      });
    });

    const selectAll = this.container.querySelector('#select-all-metrics');
    selectAll?.addEventListener('click', () => {
      this.setSelectedMetrics(this.availableMetrics);
      if (this.onSelectionChanged) {
        this.onSelectionChanged(this.getSelectedMetrics());
      }
    });

    const deselectAll = this.container.querySelector('#deselect-all-metrics');
    deselectAll?.addEventListener('click', () => {
      this.setSelectedMetrics([]);
      if (this.onSelectionChanged) {
        this.onSelectionChanged(this.getSelectedMetrics());
      }
    });
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

