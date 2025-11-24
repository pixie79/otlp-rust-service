import { describe, it, expect, beforeEach, vi } from 'vitest';
import { MetricSelector } from '../../../src/metrics/metric-selector.js';

describe('MetricSelector', () => {
  let container;

  beforeEach(() => {
    document.body.innerHTML = '<div id="metric-selector"></div>';
    container = document.getElementById('metric-selector');
  });

  it('creates selector with container element', () => {
    const selector = new MetricSelector(container);
    expect(selector.container).toBe(container);
  });

  it('throws error if container is missing', () => {
    expect(() => new MetricSelector(null)).toThrow('MetricSelector requires a container element');
  });

  it('sets available metrics and renders them', () => {
    const selector = new MetricSelector(container);
    const metrics = ['request_duration', 'error_rate', 'throughput'];

    selector.setAvailableMetrics(metrics);

    expect(selector.availableMetrics).toEqual(metrics);
    expect(container.innerHTML).toContain('request_duration');
    expect(container.innerHTML).toContain('error_rate');
    expect(container.innerHTML).toContain('throughput');
  });

  it('shows empty state when no metrics available', () => {
    const selector = new MetricSelector(container);

    selector.setAvailableMetrics([]);

    expect(container.innerHTML).toContain('No metrics available');
  });

  it('gets selected metrics', () => {
    const selector = new MetricSelector(container);
    selector.setAvailableMetrics(['metric1', 'metric2', 'metric3']);
    selector.selectedMetrics.add('metric1');
    selector.selectedMetrics.add('metric3');

    const selected = selector.getSelectedMetrics();

    expect(selected).toEqual(['metric1', 'metric3']);
  });

  it('sets selected metrics', () => {
    const selector = new MetricSelector(container);
    selector.setAvailableMetrics(['metric1', 'metric2', 'metric3']);

    selector.setSelectedMetrics(['metric1', 'metric2']);

    expect(selector.getSelectedMetrics()).toEqual(['metric1', 'metric2']);
    const checkboxes = container.querySelectorAll('input[type="checkbox"]:checked');
    expect(checkboxes.length).toBe(2);
  });

  it('calls onSelectionChanged callback when selection changes', () => {
    const selector = new MetricSelector(container);
    const callback = vi.fn();
    selector.onSelectionChanged = callback;
    selector.setAvailableMetrics(['metric1', 'metric2']);

    const checkbox = container.querySelector('input[type="checkbox"]');
    checkbox.checked = true;
    checkbox.dispatchEvent(new Event('change'));

    expect(callback).toHaveBeenCalledWith(expect.arrayContaining(['metric1']));
  });

  it('select all button selects all metrics', () => {
    const selector = new MetricSelector(container);
    const callback = vi.fn();
    selector.onSelectionChanged = callback;
    selector.setAvailableMetrics(['metric1', 'metric2', 'metric3']);

    const selectAllButton = container.querySelector('#select-all-metrics');
    selectAllButton.click();

    expect(selector.getSelectedMetrics()).toEqual(['metric1', 'metric2', 'metric3']);
    expect(callback).toHaveBeenCalledWith(['metric1', 'metric2', 'metric3']);
  });

  it('deselect all button deselects all metrics', () => {
    const selector = new MetricSelector(container);
    const callback = vi.fn();
    selector.onSelectionChanged = callback;
    selector.setAvailableMetrics(['metric1', 'metric2']);
    selector.setSelectedMetrics(['metric1', 'metric2']);

    const deselectAllButton = container.querySelector('#deselect-all-metrics');
    deselectAllButton.click();

    expect(selector.getSelectedMetrics()).toEqual([]);
    expect(callback).toHaveBeenCalledWith([]);
  });

  it('escapes HTML in metric names to prevent XSS', () => {
    const selector = new MetricSelector(container);
    const metrics = ['<script>alert("xss")</script>', 'normal_metric'];

    selector.setAvailableMetrics(metrics);

    expect(container.innerHTML).not.toContain('<script>');
    expect(container.innerHTML).toContain('&lt;script&gt;');
    expect(container.innerHTML).toContain('normal_metric');
  });
});
