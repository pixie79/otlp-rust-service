import { describe, it, expect, beforeEach } from 'vitest';
import { Layout } from '../../../src/ui/layout.js';

describe('Layout', () => {
  let container;

  beforeEach(() => {
    document.body.innerHTML = '<div id="layout-container"></div>';
    container = document.getElementById('layout-container');
  });

  it('creates layout with container element', () => {
    const layout = new Layout(container);
    expect(layout.container).toBe(container);
  });

  it('throws error if container is missing', () => {
    expect(() => new Layout(null)).toThrow('Layout requires a container element');
  });

  it('renders main layout structure', () => {
    const layout = new Layout(container);
    layout.render();

    expect(container.querySelector('header')).toBeTruthy();
    expect(container.querySelector('main')).toBeTruthy();
    expect(container.querySelector('#trace-panel')).toBeTruthy();
    expect(container.querySelector('#metric-panel')).toBeTruthy();
  });

  it('includes ARIA labels for accessibility', () => {
    const layout = new Layout(container);
    layout.render();

    const header = container.querySelector('header[role="banner"]');
    const main = container.querySelector('main[role="main"]');
    const statusLine = container.querySelector('#status-line[role="status"]');

    expect(header).toBeTruthy();
    expect(main).toBeTruthy();
    expect(statusLine).toBeTruthy();
    expect(statusLine.getAttribute('aria-live')).toBe('polite');
  });

  it('switches between views', () => {
    const layout = new Layout(container);
    layout.render();

    layout.switchView('metrics');

    const tracePanel = container.querySelector('#trace-panel');
    const metricPanel = container.querySelector('#metric-panel');
    const navTraces = container.querySelector('#nav-traces');
    const navMetrics = container.querySelector('#nav-metrics');

    expect(tracePanel.style.display).toBe('none');
    expect(metricPanel.style.display).toBe('block');
    expect(navTraces?.getAttribute('aria-selected')).toBe('false');
    expect(navMetrics?.getAttribute('aria-selected')).toBe('true');
  });

  it('updates status badge', () => {
    const layout = new Layout(container);
    layout.render();

    layout.setStatus('Ready');

    const statusLine = container.querySelector('#status-line');
    expect(statusLine.innerHTML).toContain('Ready');
    expect(statusLine.getAttribute('aria-label')).toContain('Ready');
  });

  it('gets section container by ID', () => {
    const layout = new Layout(container);
    layout.render();

    const tracePanel = layout.getSection('trace-panel');
    expect(tracePanel).toBeTruthy();
    expect(tracePanel.id).toBe('trace-panel');
  });

  it('escapes HTML in status text', () => {
    const layout = new Layout(container);
    layout.render();

    layout.setStatus('<script>alert("xss")</script>');

    const statusLine = container.querySelector('#status-line');
    expect(statusLine.innerHTML).not.toContain('<script>');
    expect(statusLine.innerHTML).toContain('&lt;script&gt;');
  });
});

