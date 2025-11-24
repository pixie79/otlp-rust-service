/**
 * Search component for searching traces and metrics
 * Implements search by trace ID or metric name per acceptance scenario 2
 */
export class Search {
  constructor(container, options = {}) {
    if (!container) {
      throw new Error('Search requires a container element');
    }

    this.container = container;
    this.placeholder = options.placeholder || 'Search...';
    this.onSearch = options.onSearch || null;
    this.onClear = options.onClear || null;
    this.currentQuery = '';
  }

  /**
   * Render search input
   */
  render() {
    this.container.innerHTML = `
      <div class="search-container" role="search" aria-label="Search">
        <input
          type="search"
          id="search-input"
          class="search-input"
          placeholder="${this._escapeHtml(this.placeholder)}"
          aria-label="Search input"
          autocomplete="off"
        />
        <button
          type="button"
          id="search-clear"
          class="search-clear"
          aria-label="Clear search"
          style="display: none;"
        >
          ×
        </button>
      </div>
    `;

    this._attachEventHandlers();
  }

  /**
   * Get current search query
   */
  getQuery() {
    return this.currentQuery;
  }

  /**
   * Set search query
   */
  setQuery(query) {
    this.currentQuery = query;
    const input = this.container.querySelector('#search-input');
    if (input) {
      input.value = query;
      this._updateClearButton();
    }
  }

  /**
   * Clear search
   */
  clear() {
    this.setQuery('');
    if (this.onClear) {
      this.onClear();
    }
  }

  /**
   * Focus search input
   */
  focus() {
    const input = this.container.querySelector('#search-input');
    input?.focus();
  }

  /**
   * Attach event handlers
   * @private
   */
  _attachEventHandlers() {
    const input = this.container.querySelector('#search-input');
    const clearButton = this.container.querySelector('#search-clear');

    if (input) {
      // Search on input
      input.addEventListener('input', (e) => {
        this.currentQuery = e.target.value.trim();
        this._updateClearButton();

        if (this.onSearch) {
          this.onSearch(this.currentQuery);
        }
      });

      // Search on Enter
      input.addEventListener('keydown', (e) => {
        if (e.key === 'Enter') {
          e.preventDefault();
          if (this.onSearch) {
            this.onSearch(this.currentQuery);
          }
        } else if (e.key === 'Escape') {
          this.clear();
          input.blur();
        }
      });
    }

    if (clearButton) {
      clearButton.addEventListener('click', () => {
        this.clear();
        input?.focus();
      });
    }
  }

  /**
   * Update clear button visibility
   * @private
   */
  _updateClearButton() {
    const clearButton = this.container.querySelector('#search-clear');
    if (clearButton) {
      clearButton.style.display = this.currentQuery ? 'block' : 'none';
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
