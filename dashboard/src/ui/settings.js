/**
 * Settings component for configuring dashboard behavior
 * Allows users to configure polling interval, data limits, and other settings
 */
export class Settings {
  constructor(container, options = {}) {
    if (!container) {
      throw new Error('Settings requires a container element');
    }

    this.container = container;
    this.onSettingsChanged = options.onSettingsChanged || null;
    this.settings = {
      pollingInterval: options.pollingInterval || 1000,
      maxTraces: options.maxTraces || 10000,
      maxGraphPoints: options.maxGraphPoints || 10000,
      maxLoadedTables: options.maxLoadedTables || 50,
    };
  }

  /**
   * Render the settings UI
   */
  render() {
    this.container.innerHTML = `
      <div class="settings-panel">
        <h2>Settings</h2>
        <form id="settings-form">
          <div class="settings-group">
            <label for="polling-interval">
              Polling Interval (ms)
              <span class="settings-hint">How often to check for new files</span>
            </label>
            <input
              type="number"
              id="polling-interval"
              name="pollingInterval"
              min="100"
              max="10000"
              step="100"
              value="${this.settings.pollingInterval}"
            />
          </div>

          <div class="settings-group">
            <label for="max-traces">
              Max Traces
              <span class="settings-hint">Maximum number of traces to keep in memory</span>
            </label>
            <input
              type="number"
              id="max-traces"
              name="maxTraces"
              min="100"
              max="100000"
              step="100"
              value="${this.settings.maxTraces}"
            />
          </div>

          <div class="settings-group">
            <label for="max-graph-points">
              Max Graph Points
              <span class="settings-hint">Maximum number of data points per graph</span>
            </label>
            <input
              type="number"
              id="max-graph-points"
              name="maxGraphPoints"
              min="100"
              max="100000"
              step="100"
              value="${this.settings.maxGraphPoints}"
            />
          </div>

          <div class="settings-group">
            <label for="max-loaded-tables">
              Max Loaded Tables
              <span class="settings-hint">Maximum number of DuckDB tables to keep in memory</span>
            </label>
            <input
              type="number"
              id="max-loaded-tables"
              name="maxLoadedTables"
              min="5"
              max="200"
              step="5"
              value="${this.settings.maxLoadedTables}"
            />
          </div>

          <div class="settings-actions">
            <button type="submit" class="primary">Save Settings</button>
            <button type="button" class="ghost" id="reset-settings">Reset to Defaults</button>
          </div>
        </form>
      </div>
    `;

    this._attachEventHandlers();
  }

  /**
   * Attach event handlers
   * @private
   */
  _attachEventHandlers() {
    const form = document.getElementById('settings-form');
    form?.addEventListener('submit', (e) => {
      e.preventDefault();
      this._saveSettings();
    });

    const resetButton = document.getElementById('reset-settings');
    resetButton?.addEventListener('click', () => {
      this._resetToDefaults();
    });
  }

  /**
   * Save settings from form
   * @private
   */
  _saveSettings() {
    const form = document.getElementById('settings-form');
    const formData = new FormData(form);

    this.settings = {
      pollingInterval: parseInt(formData.get('pollingInterval') || '1000', 10),
      maxTraces: parseInt(formData.get('maxTraces') || '10000', 10),
      maxGraphPoints: parseInt(formData.get('maxGraphPoints') || '10000', 10),
      maxLoadedTables: parseInt(formData.get('maxLoadedTables') || '50', 10),
    };

    // Validate settings
    if (this.settings.pollingInterval < 100 || this.settings.pollingInterval > 10000) {
      alert('Polling interval must be between 100 and 10000 ms');
      return;
    }

    if (this.settings.maxTraces < 100 || this.settings.maxTraces > 100000) {
      alert('Max traces must be between 100 and 100000');
      return;
    }

    if (this.settings.maxGraphPoints < 100 || this.settings.maxGraphPoints > 100000) {
      alert('Max graph points must be between 100 and 100000');
      return;
    }

    if (this.settings.maxLoadedTables < 5 || this.settings.maxLoadedTables > 200) {
      alert('Max loaded tables must be between 5 and 200');
      return;
    }

    // Notify parent
    this.onSettingsChanged?.(this.settings);
  }

  /**
   * Reset settings to defaults
   * @private
   */
  _resetToDefaults() {
    this.settings = {
      pollingInterval: 1000,
      maxTraces: 10000,
      maxGraphPoints: 10000,
      maxLoadedTables: 50,
    };

    this.render();
    this.onSettingsChanged?.(this.settings);
  }

  /**
   * Get current settings
   * @returns {Object} Current settings
   */
  getSettings() {
    return { ...this.settings };
  }

  /**
   * Update settings programmatically
   * @param {Object} newSettings - New settings to apply
   */
  updateSettings(newSettings) {
    this.settings = { ...this.settings, ...newSettings };
    this.render();
  }
}

