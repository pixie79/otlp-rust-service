/**
 * Loading component for displaying loading states and error messages
 * Implements loading states and error messages per FR-019
 */
export class Loading {
  constructor(container) {
    if (!container) {
      throw new Error('Loading requires a container element');
    }

    this.container = container;
    this.isLoading = false;
    this.error = null;
  }

  /**
   * Show loading state
   * @param {string} _message - Loading message (currently unused, kept for API consistency)
   */
  show(_message = 'Loading...') {
    this.isLoading = true;
    this.error = null;
    this._render();
  }

  /**
   * Hide loading state
   */
  hide() {
    this.isLoading = false;
    this.error = null;
    this.container.innerHTML = '';
  }

  /**
   * Show error message
   * @param {Error|string} error - Error object or error message
   * @param {Object} options - Options for error display
   */
  showError(error, options = {}) {
    this.isLoading = false;

    if (error instanceof Error) {
      this.error = {
        message: error.message,
        name: error.name,
        details: options.details || null,
      };
    } else {
      this.error = {
        message: String(error),
        name: options.name || 'Error',
        details: options.details || null,
      };
    }

    this._render();
  }

  /**
   * Clear error
   */
  clearError() {
    this.error = null;
    this._render();
  }

  /**
   * Render loading or error state
   * @private
   */
  _render() {
    if (this.isLoading) {
      this.container.innerHTML = `
        <div class="loading-state" role="status" aria-live="polite" aria-busy="true">
          <div class="loading-spinner"></div>
          <p>Loading...</p>
        </div>
      `;
      return;
    }

    if (this.error) {
      const errorClass = this._getErrorClass(this.error.name);
      this.container.innerHTML = `
        <div class="error-state ${errorClass}" role="alert" aria-live="assertive">
          <div class="error-state__icon" aria-hidden="true">⚠️</div>
          <div class="error-state__content">
            <h3 class="error-state__title">${this._escapeHtml(this.error.name)}</h3>
            <p class="error-state__message">${this._escapeHtml(this.error.message)}</p>
            ${this.error.details ? `<pre class="error-state__details">${this._escapeHtml(this.error.details)}</pre>` : ''}
          </div>
        </div>
      `;
      return;
    }

    this.container.innerHTML = '';
  }

  /**
   * Get CSS class for error type
   * @private
   */
  _getErrorClass(errorName) {
    const errorClasses = {
      FileReadError: 'error-state--file-read',
      DuckDBError: 'error-state--database',
      ArrowParseError: 'error-state--parse',
    };
    return errorClasses[errorName] || 'error-state--generic';
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
