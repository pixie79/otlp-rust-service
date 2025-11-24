import { describe, it, expect, beforeEach, vi } from 'vitest';
import { Navigation } from '../../../src/ui/navigation.js';

describe('Navigation', () => {
  let container;

  beforeEach(() => {
    document.body.innerHTML = '<div id="nav-container"></div>';
    container = document.getElementById('nav-container');
  });

  it('creates navigation with container element', () => {
    const nav = new Navigation(container);
    expect(nav.container).toBe(container);
  });

  it('throws error if container is missing', () => {
    expect(() => new Navigation(null)).toThrow('Navigation requires a container element');
  });

  it('renders navigation buttons', () => {
    const nav = new Navigation(container);
    nav.render();

    const buttons = container.querySelectorAll('.view-nav__button');
    expect(buttons.length).toBe(2);
    expect(buttons[0].textContent).toBe('Traces');
    expect(buttons[1].textContent).toBe('Metrics');
  });

  it('includes ARIA attributes for accessibility', () => {
    const nav = new Navigation(container);
    nav.render();

    const navElement = container.querySelector('nav[role="tablist"]');
    const tracesButton = container.querySelector('#nav-traces');
    const metricsButton = container.querySelector('#nav-metrics');

    expect(navElement).toBeTruthy();
    expect(navElement.getAttribute('aria-label')).toBe('View navigation');
    expect(tracesButton.getAttribute('role')).toBe('tab');
    expect(tracesButton.getAttribute('aria-controls')).toBe('trace-panel');
    expect(metricsButton.getAttribute('role')).toBe('tab');
    expect(metricsButton.getAttribute('aria-controls')).toBe('metric-panel');
  });

  it('switches to a view when button is clicked', () => {
    const callback = vi.fn();
    const nav = new Navigation(container, {
      currentView: 'traces',
      onViewChanged: callback,
    });
    nav.render();

    const metricsButton = container.querySelector('#nav-metrics');
    metricsButton.click();

    expect(nav.getCurrentView()).toBe('metrics');
    expect(callback).toHaveBeenCalledWith('metrics');
  });

  it('handles keyboard navigation with arrow keys', () => {
    const callback = vi.fn();
    const nav = new Navigation(container, {
      currentView: 'traces',
      onViewChanged: callback,
    });
    nav.render();

    const tracesButton = container.querySelector('#nav-traces');
    tracesButton.focus();

    // Press right arrow
    const rightArrowEvent = new KeyboardEvent('keydown', { key: 'ArrowRight' });
    tracesButton.dispatchEvent(rightArrowEvent);

    expect(nav.getCurrentView()).toBe('metrics');
    expect(callback).toHaveBeenCalledWith('metrics');
  });

  it('handles Enter and Space keys for activation', () => {
    const callback = vi.fn();
    const nav = new Navigation(container, {
      currentView: 'traces',
      onViewChanged: callback,
    });
    nav.render();

    const metricsButton = container.querySelector('#nav-metrics');
    metricsButton.focus();

    // Press Enter
    const enterEvent = new KeyboardEvent('keydown', { key: 'Enter', preventDefault: vi.fn() });
    metricsButton.dispatchEvent(enterEvent);

    expect(nav.getCurrentView()).toBe('metrics');
  });

  it('updates button states when switching views', () => {
    const nav = new Navigation(container, { currentView: 'traces' });
    nav.render();

    nav.switchToView('metrics');

    const tracesButton = container.querySelector('#nav-traces');
    const metricsButton = container.querySelector('#nav-metrics');

    expect(tracesButton.classList.contains('active')).toBe(false);
    expect(metricsButton.classList.contains('active')).toBe(true);
    expect(tracesButton.getAttribute('aria-selected')).toBe('false');
    expect(metricsButton.getAttribute('aria-selected')).toBe('true');
  });

  it('does not call callback if switching to same view', () => {
    const callback = vi.fn();
    const nav = new Navigation(container, {
      currentView: 'traces',
      onViewChanged: callback,
    });
    nav.render();

    nav.switchToView('traces');

    expect(callback).not.toHaveBeenCalled();
  });
});

