import { describe, it, expect, beforeEach } from 'vitest';
import { Navigation } from '../../src/ui/navigation.js';
import { Layout } from '../../src/ui/layout.js';

describe('Navigation Integration', () => {
  let layoutContainer;
  let navContainer;

  beforeEach(() => {
    document.body.innerHTML = `
      <div id="app">
        <div id="layout-container"></div>
        <div id="nav-container"></div>
      </div>
    `;
    layoutContainer = document.getElementById('layout-container');
    navContainer = document.getElementById('nav-container');
  });

  it('synchronizes navigation and layout view switching', () => {
    const layout = new Layout(layoutContainer);
    layout.render();

    const nav = new Navigation(navContainer, {
      currentView: 'traces',
      onViewChanged: (view) => {
        layout.switchView(view);
      },
    });
    nav.render();

    // Switch to metrics via navigation
    const metricsButton = navContainer.querySelector('#nav-metrics');
    metricsButton.click();

    // Verify layout updated
    const tracePanel = layoutContainer.querySelector('#trace-panel');
    const metricPanel = layoutContainer.querySelector('#metric-panel');

    expect(tracePanel.style.display).toBe('none');
    expect(metricPanel.style.display).toBe('block');
    expect(nav.getCurrentView()).toBe('metrics');
  });

  it('maintains consistent state between navigation and layout', () => {
    const layout = new Layout(layoutContainer);
    layout.render();

    const nav = new Navigation(navContainer, {
      currentView: 'traces',
      onViewChanged: (view) => {
        layout.switchView(view);
      },
    });
    nav.render();

    // Switch views multiple times
    nav.switchToView('metrics');
    nav.switchToView('traces');
    nav.switchToView('metrics');

    // Final state should be consistent
    expect(nav.getCurrentView()).toBe('metrics');
    const metricPanel = layoutContainer.querySelector('#metric-panel');
    expect(metricPanel.style.display).toBe('block');
  });

  it('handles keyboard navigation across views', () => {
    const layout = new Layout(layoutContainer);
    layout.render();

    const nav = new Navigation(navContainer, {
      currentView: 'traces',
      onViewChanged: (view) => {
        layout.switchView(view);
      },
    });
    nav.render();

    const tracesButton = navContainer.querySelector('#nav-traces');
    tracesButton.focus();

    // Navigate with arrow keys
    const rightArrow = new KeyboardEvent('keydown', { key: 'ArrowRight' });
    tracesButton.dispatchEvent(rightArrow);

    expect(nav.getCurrentView()).toBe('metrics');
    const metricPanel = layoutContainer.querySelector('#metric-panel');
    expect(metricPanel.style.display).toBe('block');
  });
});
