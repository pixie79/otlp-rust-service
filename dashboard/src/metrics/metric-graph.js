import Plotly from 'plotly.js-dist-min';
import { groupMetricsBySeries, sortByTimestampAsc } from './metric-entry.js';

/**
 * MetricGraph component for displaying time-series metrics using Plotly.js
 * Implements the MetricGraph interface from contracts/ui-components.md
 */
export class MetricGraph {
  constructor(container, options = {}) {
    if (!container) {
      throw new Error('MetricGraph requires a container element');
    }

    this.container = container;
    this.metricName = options.metricName || 'metric';
    this.maxDataPoints = options.maxDataPoints || 10000;
    this.data = new Map(); // Map of series key to MetricEntry[]
    this.timeRange = {
      start: null,
      end: null,
      preset: 'last1h',
    };
    this.onTimeRangeChanged = null;
    this.plotlyLayout = {
      title: this.metricName,
      xaxis: {
        title: 'Time',
        type: 'date',
      },
      yaxis: {
        title: 'Value',
      },
      hovermode: 'closest',
      showlegend: true,
      margin: { l: 60, r: 30, t: 50, b: 50 },
    };
    this.plotlyConfig = {
      displayModeBar: true,
      displaylogo: false,
      modeBarButtonsToRemove: ['lasso2d', 'select2d'],
    };
    this.allDataPoints = []; // Track all data points for limit management
  }

  /**
   * Add or update metric data for a specific metric name
   * Uses Plotly.js extendTraces for efficient real-time updates
   * @param {string} metricName - Name of the metric
   * @param {Array} data - Array of MetricEntry objects
   */
  updateMetric(metricName, data) {
    if (!data || data.length === 0) {
      return;
    }

    // Group metrics by series (metric name + labels)
    const grouped = groupMetricsBySeries(data);
    const hasExistingData = this.data.size > 0;

    // Update data for each series
    const newDataBySeries = new Map();
    for (const [seriesKey, metrics] of grouped.entries()) {
      const existing = this.data.get(seriesKey) || [];
      
      // Filter out duplicates (same timestamp)
      const existingTimestamps = new Set(existing.map((m) => m.timestamp));
      const newMetrics = metrics.filter((m) => !existingTimestamps.has(m.timestamp));
      
      if (newMetrics.length === 0) continue;

      const combined = [...existing, ...newMetrics].sort(sortByTimestampAsc);

      // Limit data points to prevent memory issues
      if (combined.length > this.maxDataPoints) {
        const keep = combined.slice(-this.maxDataPoints);
        this.data.set(seriesKey, keep);
        newDataBySeries.set(seriesKey, keep);
      } else {
        this.data.set(seriesKey, combined);
        newDataBySeries.set(seriesKey, combined);
      }
    }

    // Use extendTraces for efficient updates if graph already exists
    if (hasExistingData && this.container.querySelector('.js-plotly-plot')) {
      this._extendTraces(newDataBySeries);
    } else {
      this._render();
    }
  }

  /**
   * Extend existing traces with new data using Plotly.js extendTraces
   * More efficient than full re-render for real-time updates
   * @private
   */
  _extendTraces(newDataBySeries) {
    if (newDataBySeries.size === 0 || !this.isRendered) {
      this._render();
      return;
    }

    const timeRange = this._calculateTimeRange(Date.now() * 1_000_000);
    const updateData = [];
    const traceIndices = [];
    const maxPoints = [];

    // Prepare update data for each series that has new data
    let traceIndex = 0;
    for (const [seriesKey, seriesData] of newDataBySeries.entries()) {
      const storedIndex = this.traceIndices.get(seriesKey);
      if (storedIndex === undefined) {
        // New series, need full render
        this._render();
        return;
      }

      // Filter by time range
      const filtered = seriesData.filter((m) => {
        if (timeRange.start && m.timestamp < timeRange.start) return false;
        if (timeRange.end && m.timestamp > timeRange.end) return false;
        return true;
      });

      if (filtered.length === 0) continue;

      // Get existing data length from stored data
      const existing = this.data.get(seriesKey) || [];
      const existingLength = existing.length;
      const newLength = filtered.length;
      const pointsToAdd = Math.max(0, newLength - existingLength);

      if (pointsToAdd > 0) {
        const newPoints = filtered.slice(-pointsToAdd);
        const x = newPoints.map((m) => new Date(m.timestamp / 1_000_000));
        const y = newPoints.map((m) => m.value);

        updateData.push({
          x: [x],
          y: [y],
        });
        traceIndices.push(storedIndex);
        maxPoints.push(this.maxDataPoints);
      }
    }

    if (updateData.length > 0) {
      Plotly.extendTraces(this.container, updateData, traceIndices, maxPoints);
    }
  }

  /**
   * Remove metric from display
   * @param {string} metricName - Name of the metric to remove
   */
  removeMetric(metricName) {
    const keysToRemove = [];
    for (const key of this.data.keys()) {
      if (key.startsWith(metricName)) {
        keysToRemove.push(key);
      }
    }
    for (const key of keysToRemove) {
      this.data.delete(key);
    }
    this._render();
  }

  /**
   * Set time range for the graph
   * @param {Object} range - TimeRange object with start, end, preset
   */
  setTimeRange(range) {
    this.timeRange = { ...this.timeRange, ...range };
    this._render();
    if (this.onTimeRangeChanged) {
      this.onTimeRangeChanged(this.timeRange);
    }
  }

  /**
   * Get current time range
   * @returns {Object} Current TimeRange
   */
  getTimeRange() {
    return { ...this.timeRange };
  }

  /**
   * Render the graph using Plotly.js
   * @private
   */
  _render() {
    if (this.data.size === 0) {
      // Show empty state
      Plotly.purge(this.container);
      this.container.innerHTML = '<div class="metric-graph__empty"><p>No data available</p></div>';
      return;
    }

    const traces = [];
    const now = Date.now() * 1_000_000;
    const timeRange = this._calculateTimeRange(now);

    // Create a trace for each series
    for (const [seriesKey, metrics] of this.data.entries()) {
      // Filter by time range
      const filtered = metrics.filter((m) => {
        if (timeRange.start && m.timestamp < timeRange.start) return false;
        if (timeRange.end && m.timestamp > timeRange.end) return false;
        return true;
      });

      if (filtered.length === 0) continue;

      const x = filtered.map((m) => new Date(m.timestamp / 1_000_000));
      const y = filtered.map((m) => m.value);
      const text = filtered.map((m) => this._formatTooltip(m));

      traces.push({
        name: seriesKey,
        x,
        y,
        text,
        type: 'scatter',
        mode: 'lines+markers',
        hovertemplate: '<b>%{fullData.name}</b><br>%{text}<extra></extra>',
        line: { width: 2 },
        marker: { size: 4 },
      });
    }

    const layout = {
      ...this.plotlyLayout,
      title: this.metricName,
      xaxis: {
        ...this.plotlyLayout.xaxis,
        range: timeRange.start && timeRange.end ? [new Date(timeRange.start / 1_000_000), new Date(timeRange.end / 1_000_000)] : undefined,
      },
    };

    Plotly.react(this.container, traces, layout, this.plotlyConfig).then(() => {
      // Store trace indices for extendTraces
      this.traceIndices.clear();
      traces.forEach((trace, index) => {
        this.traceIndices.set(trace.name, index);
      });
      this.isRendered = true;
    });
  }

  /**
   * Calculate time range based on preset or explicit start/end
   * @private
   */
  _calculateTimeRange(now) {
    if (this.timeRange.start && this.timeRange.end) {
      return { start: this.timeRange.start, end: this.timeRange.end };
    }

    const preset = this.timeRange.preset || 'last1h';
    const ranges = {
      last5m: 5 * 60 * 1_000_000_000,
      last15m: 15 * 60 * 1_000_000_000,
      last1h: 60 * 60 * 1_000_000_000,
      last6h: 6 * 60 * 60 * 1_000_000_000,
      last24h: 24 * 60 * 60 * 1_000_000_000,
    };

    const duration = ranges[preset] || ranges.last1h;
    return {
      start: now - duration,
      end: now,
    };
  }

  /**
   * Format tooltip text for a metric entry
   * @private
   */
  _formatTooltip(metric) {
    const time = new Date(metric.timestamp / 1_000_000).toLocaleString();
    const labels = Object.entries(metric.labels)
      .map(([k, v]) => `${k}=${v}`)
      .join(', ');
    return `Time: ${time}<br>Value: ${metric.value}${metric.unit ? ` ${metric.unit}` : ''}${labels ? `<br>Labels: ${labels}` : ''}`;
  }

  /**
   * Get the current number of data points
   * @returns {number} Total number of data points across all series
   */
  getPointCount() {
    let count = 0;
    this.data.forEach((entries) => {
      count += entries.length;
    });
    return count;
  }

  /**
   * Remove the oldest data points
   * @param {number} count - Number of points to remove
   */
  removeOldestPoints(count) {
    if (count <= 0) return;

    // Remove oldest points from each series
    this.data.forEach((entries, seriesKey) => {
      if (entries.length > 0) {
        // Sort by timestamp ascending to get oldest first
        entries.sort((a, b) => a.timestamp - b.timestamp);
        const toRemove = Math.min(count, entries.length);
        entries.splice(0, toRemove);
      }
    });

    // Re-render if already rendered
    if (this.isRendered) {
      this._updatePlotlyTraces();
    }
  }

  /**
   * Destroy the graph and clean up
   */
  destroy() {
    Plotly.purge(this.container);
    this.data.clear();
    this.traceIndices.clear();
    this.isRendered = false;
  }
}

