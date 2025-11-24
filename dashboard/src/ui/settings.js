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

    // Validate settings and show inline errors
    const errors = [];

    if (this.settings.pollingInterval < 100 || this.settings.pollingInterval > 10000) {
      errors.push({
        field: 'polling-interval',
        message: 'Polling interval must be between 100 and 10000 ms',
      });
    }

    if (this.settings.maxTraces < 100 || this.settings.maxTraces > 100000) {
      errors.push({ field: 'max-traces', message: 'Max traces must be between 100 and 100000' });
    }

    if (this.settings.maxGraphPoints < 100 || this.settings.maxGraphPoints > 100000) {
      errors.push({
        field: 'max-graph-points',
        message: 'Max graph points must be between 100 and 100000',
      });
    }

    if (this.settings.maxLoadedTables < 5 || this.settings.maxLoadedTables > 200) {
      errors.push({
        field: 'max-loaded-tables',
        message: 'Max loaded tables must be between 5 and 200',
      });
    }

    if (errors.length > 0) {
      // Show inline error messages
      this._showValidationErrors(errors);
      return;
    }

    // Clear any previous errors
    this._clearValidationErrors();

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

  /**
   * Show validation errors inline
   * @private
   * @param {Array<{field: string, message: string}>} errors - Array of error objects
   */
  _showValidationErrors(errors) {
    // Clear previous errors
    this._clearValidationErrors();

    errors.forEach(({ field, message }) => {
      const input = document.getElementById(field);
      if (input) {
        input.classList.add('error');
        input.setAttribute('aria-invalid', 'true');

        // Create or update error message
        let errorMsg = input.parentElement.querySelector('.error-message');
        if (!errorMsg) {
          errorMsg = document.createElement('div');
          errorMsg.className = 'error-message';
          errorMsg.setAttribute('role', 'alert');
          input.parentElement.appendChild(errorMsg);
        }
        errorMsg.textContent = message;
      }
    });
  }

  /**
   * Clear validation errors
   * @private
   */
  _clearValidationErrors() {
    const form = document.getElementById('settings-form');
    if (!form) return;

    // Remove error styling and messages
    form.querySelectorAll('.error').forEach((el) => {
      el.classList.remove('error');
      el.removeAttribute('aria-invalid');
    });

    form.querySelectorAll('.error-message').forEach((el) => {
      el.remove();
    });
  }
}
