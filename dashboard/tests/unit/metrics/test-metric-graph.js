import { describe, it, expect, beforeEach, vi } from 'vitest';
import { MetricGraph } from '../../../src/metrics/metric-graph.js';
// createMetricEntry not used in this test file
// import { createMetricEntry } from '../../../src/metrics/metric-entry.js';

// Mock Plotly.js
vi.mock('plotly.js-dist-min', () => ({
  default: {
    react: vi.fn((_container, _traces, _layout, _config) => Promise.resolve()),
    extendTraces: vi.fn((_container, _updateData, _indices, _maxPoints) => {}),
    purge: vi.fn((_container) => {}),
  },
}));

const buildMetric = (overrides = {}) => ({
  metricName: 'request_duration',
  value: 100,
  timestamp: Date.now() * 1_000_000,
  labels: { service: 'api' },
  metricType: 'gauge',
  unit: 'ms',
  ...overrides,
});

describe('MetricGraph', () => {
  let container;

  beforeEach(() => {
    document.body.innerHTML = '<div id="metric-graph"></div>';
    container = document.getElementById('metric-graph');
    vi.clearAllMocks();
  });

  it('creates graph with container element', () => {
    const graph = new MetricGraph(container, { metricName: 'test_metric' });
    expect(graph.container).toBe(container);
    expect(graph.metricName).toBe('test_metric');
  });

  it('throws error if container is missing', () => {
    expect(() => new MetricGraph(null)).toThrow('MetricGraph requires a container element');
  });

  it('updates metric data and renders graph', async () => {
    const Plotly = (await import('plotly.js-dist-min')).default;
    const metrics = [
      buildMetric({ value: 100, timestamp: Date.now() * 1_000_000 }),
      buildMetric({ value: 150, timestamp: Date.now() * 1_000_000 + 1_000_000 }),
    ];

    const graph = new MetricGraph(container, { metricName: 'request_duration' });
    graph.updateMetric('request_duration', metrics);

    expect(Plotly.react).toHaveBeenCalled();
    const callArgs = Plotly.react.mock.calls[0];
    expect(callArgs[0]).toBe(container);
    expect(callArgs[1].length).toBeGreaterThan(0);
  });

  it('uses extendTraces for efficient updates when graph already rendered', async () => {
    const Plotly = (await import('plotly.js-dist-min')).default;
    const graph = new MetricGraph(container, { metricName: 'test' });

    // Initial render
    const initialMetrics = [buildMetric({ value: 100 })];
    graph.updateMetric('test', initialMetrics);
    graph.isRendered = true; // Simulate rendered state
    graph.traceIndices.set('test{}', 0);

    // Update with new data
    const newMetrics = [
      buildMetric({ value: 200, timestamp: Date.now() * 1_000_000 + 10_000_000 }),
    ];
    graph.updateMetric('test', newMetrics);

    expect(Plotly.extendTraces).toHaveBeenCalled();
  });

  it('removes metric from display', async () => {
    const Plotly = (await import('plotly.js-dist-min')).default;
    const graph = new MetricGraph(container, { metricName: 'test' });
    graph.data.set('test{}', [buildMetric()]);

    graph.removeMetric('test');

    expect(graph.data.size).toBe(0);
    expect(Plotly.react).toHaveBeenCalled();
  });

  it('sets and gets time range', () => {
    const graph = new MetricGraph(container);
    const range = { preset: 'last1h', start: 1000, end: 2000 };

    graph.setTimeRange(range);
    const retrieved = graph.getTimeRange();

    expect(retrieved.preset).toBe('last1h');
    expect(retrieved.start).toBe(1000);
    expect(retrieved.end).toBe(2000);
  });

  it('calls onTimeRangeChanged callback when time range changes', () => {
    const graph = new MetricGraph(container);
    const callback = vi.fn();
    graph.onTimeRangeChanged = callback;

    graph.setTimeRange({ preset: 'last6h' });

    expect(callback).toHaveBeenCalledWith(expect.objectContaining({ preset: 'last6h' }));
  });

  it('shows empty state when no data', async () => {
    const Plotly = (await import('plotly.js-dist-min')).default;
    const graph = new MetricGraph(container);

    graph.updateMetric('test', []);

    expect(container.innerHTML).toContain('No data available');
    expect(Plotly.purge).toHaveBeenCalled();
  });

  it('limits data points to maxDataPoints', () => {
    const graph = new MetricGraph(container, { maxDataPoints: 5 });
    const metrics = Array.from({ length: 10 }, (_, i) =>
      buildMetric({ value: i, timestamp: Date.now() * 1_000_000 + i * 1_000_000 })
    );

    graph.updateMetric('test', metrics);

    expect(graph.data.get('test{}').length).toBeLessThanOrEqual(5);
  });

  it('filters data by time range', async () => {
    const Plotly = (await import('plotly.js-dist-min')).default;
    const now = Date.now() * 1_000_000;
    const graph = new MetricGraph(container);
    graph.setTimeRange({ preset: 'last1h' });

    const metrics = [
      buildMetric({ timestamp: now - 2 * 60 * 60 * 1_000_000_000 }), // 2 hours ago
      buildMetric({ timestamp: now - 30 * 60 * 1_000_000_000 }), // 30 min ago
      buildMetric({ timestamp: now }), // now
    ];

    graph.updateMetric('test', metrics);

    const callArgs = Plotly.react.mock.calls[0];
    const traces = callArgs[1];
    // Should filter out the 2-hour-old metric
    expect(traces[0].x.length).toBeLessThanOrEqual(2);
  });

  it('destroys graph and cleans up', async () => {
    const Plotly = (await import('plotly.js-dist-min')).default;
    const graph = new MetricGraph(container);
    graph.data.set('test{}', [buildMetric()]);
    graph.traceIndices.set('test{}', 0);
    graph.isRendered = true;

    graph.destroy();

    expect(Plotly.purge).toHaveBeenCalledWith(container);
    expect(graph.data.size).toBe(0);
    expect(graph.traceIndices.size).toBe(0);
    expect(graph.isRendered).toBe(false);
  });
});
