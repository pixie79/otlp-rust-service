/**
 * Time range selector component for traces and metrics
 * Implements time range selection per FR-014
 */
export class TimeRangeSelector {
  constructor(container, options = {}) {
    if (!container) {
      throw new Error('TimeRangeSelector requires a container element');
    }

    this.container = container;
    this.presets = options.presets || [
      { value: 'last5m', label: 'Last 5 min' },
      { value: 'last15m', label: 'Last 15 min' },
      { value: 'last1h', label: 'Last 1 hour' },
      { value: 'last6h', label: 'Last 6 hours' },
      { value: 'last24h', label: 'Last 24 hours' },
    ];
    this.currentPreset = options.currentPreset || 'last1h';
    this.onTimeRangeChanged = options.onTimeRangeChanged || null;
  }

  /**
   * Render time range selector
   */
  render() {
    this.container.innerHTML = `
      <div class="time-range-selector" role="group" aria-label="Time range selection">
        <label for="time-range-select">Time Range:</label>
        <select
          id="time-range-select"
          class="time-range-select"
          aria-label="Select time range"
        >
          ${this.presets
            .map(
              (preset) => `
            <option value="${this._escapeHtml(preset.value)}" ${preset.value === this.currentPreset ? 'selected' : ''}>
              ${this._escapeHtml(preset.label)}
            </option>
          `
            )
            .join('')}
        </select>
      </div>
    `;

    this._attachEventHandlers();
  }

  /**
   * Get current time range preset
   */
  getPreset() {
    return this.currentPreset;
  }

  /**
   * Set time range preset
   */
  setPreset(preset) {
    this.currentPreset = preset;
    const select = this.container.querySelector('#time-range-select');
    if (select) {
      select.value = preset;
    }
  }

  /**
   * Calculate time range from preset
   */
  getTimeRange() {
    const now = Date.now() * 1_000_000; // Unix nanoseconds
    const ranges = {
      last5m: 5 * 60 * 1_000_000_000,
      last15m: 15 * 60 * 1_000_000_000,
      last1h: 60 * 60 * 1_000_000_000,
      last6h: 6 * 60 * 60 * 1_000_000_000,
      last24h: 24 * 60 * 60 * 1_000_000_000,
    };

    const duration = ranges[this.currentPreset] || ranges.last1h;
    return {
      start: now - duration,
      end: now,
      preset: this.currentPreset,
    };
  }

  /**
   * Attach event handlers
   * @private
   */
  _attachEventHandlers() {
    const select = this.container.querySelector('#time-range-select');
    if (select) {
      select.addEventListener('change', (e) => {
        this.currentPreset = e.target.value;
        if (this.onTimeRangeChanged) {
          this.onTimeRangeChanged(this.getTimeRange());
        }
      });
    }
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
