/**
 * Navigation component for switching between trace and metrics views
 * Implements keyboard navigation and accessibility per acceptance scenario 1
 */
export class Navigation {
  constructor(container, options = {}) {
    if (!container) {
      throw new Error('Navigation requires a container element');
    }

    this.container = container;
    this.currentView = options.currentView || 'traces';
    this.onViewChanged = options.onViewChanged || null;
  }

  /**
   * Render navigation buttons
   */
  render() {
    this.container.innerHTML = `
      <nav class="view-nav" role="tablist" aria-label="View navigation">
        <button 
          class="view-nav__button ${this.currentView === 'traces' ? 'active' : ''}" 
          id="nav-traces" 
          data-view="traces"
          role="tab"
          aria-selected="${this.currentView === 'traces'}"
          aria-controls="trace-panel"
          tabindex="${this.currentView === 'traces' ? '0' : '-1'}"
        >
          Traces
        </button>
        <button 
          class="view-nav__button ${this.currentView === 'metrics' ? 'active' : ''}" 
          id="nav-metrics" 
          data-view="metrics"
          role="tab"
          aria-selected="${this.currentView === 'metrics'}"
          aria-controls="metric-panel"
          tabindex="${this.currentView === 'metrics' ? '0' : '-1'}"
        >
          Metrics
        </button>
      </nav>
    `;

    this._attachEventHandlers();
  }

  /**
   * Switch to a view
   */
  switchToView(view) {
    if (view === this.currentView) {
      return;
    }

    this.currentView = view;
    this._updateButtons();

    if (this.onViewChanged) {
      this.onViewChanged(view);
    }
  }

  /**
   * Attach event handlers
   * @private
   */
  _attachEventHandlers() {
    const buttons = this.container.querySelectorAll('.view-nav__button');
    
    buttons.forEach((button) => {
      // Click handler
      button.addEventListener('click', () => {
        const view = button.getAttribute('data-view');
        this.switchToView(view);
      });

      // Keyboard navigation
      button.addEventListener('keydown', (e) => {
        if (e.key === 'Enter' || e.key === ' ') {
          e.preventDefault();
          const view = button.getAttribute('data-view');
          this.switchToView(view);
        } else if (e.key === 'ArrowLeft' || e.key === 'ArrowRight') {
          e.preventDefault();
          const currentIndex = Array.from(buttons).indexOf(button);
          const nextIndex = e.key === 'ArrowLeft' 
            ? (currentIndex - 1 + buttons.length) % buttons.length
            : (currentIndex + 1) % buttons.length;
          buttons[nextIndex].focus();
          const view = buttons[nextIndex].getAttribute('data-view');
          this.switchToView(view);
        }
      });
    });
  }

  /**
   * Update button states
   * @private
   */
  _updateButtons() {
    const buttons = this.container.querySelectorAll('.view-nav__button');
    
    buttons.forEach((button) => {
      const view = button.getAttribute('data-view');
      const isActive = view === this.currentView;

      if (isActive) {
        button.classList.add('active');
        button.setAttribute('aria-selected', 'true');
        button.setAttribute('tabindex', '0');
      } else {
        button.classList.remove('active');
        button.setAttribute('aria-selected', 'false');
        button.setAttribute('tabindex', '-1');
      }
    });
  }

  /**
   * Get current view
   */
  getCurrentView() {
    return this.currentView;
  }
}

