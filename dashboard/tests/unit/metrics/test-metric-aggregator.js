import { describe, it, expect } from 'vitest';
import { MetricAggregator } from '../../../src/metrics/metric-aggregator.js';
import { createMetricEntry } from '../../../src/metrics/metric-entry.js';

const buildMetric = (value, timestamp = Date.now() * 1_000_000) =>
  createMetricEntry({
    metric_name: 'test_metric',
    value,
    timestamp_unix_nano: timestamp,
    metric_type: 'histogram',
  });

describe('MetricAggregator', () => {
  describe('aggregateHistogram', () => {
    it('aggregates histogram metrics with sum, count, min, max, avg, p50, p95, p99', () => {
      const metrics = [
        buildMetric(10),
        buildMetric(20),
        buildMetric(30),
        buildMetric(40),
        buildMetric(50),
        buildMetric(60),
        buildMetric(70),
        buildMetric(80),
        buildMetric(90),
        buildMetric(100),
      ];

      const result = MetricAggregator.aggregateHistogram(metrics);

      expect(result.sum).toBe(550);
      expect(result.count).toBe(10);
      expect(result.min).toBe(10);
      expect(result.max).toBe(100);
      expect(result.avg).toBe(55);
      expect(result.p50).toBeGreaterThanOrEqual(50);
      expect(result.p95).toBeGreaterThanOrEqual(90);
      expect(result.p99).toBeGreaterThanOrEqual(95);
    });

    it('returns zero values for empty array', () => {
      const result = MetricAggregator.aggregateHistogram([]);

      expect(result.sum).toBe(0);
      expect(result.count).toBe(0);
      expect(result.min).toBe(0);
      expect(result.max).toBe(0);
      expect(result.avg).toBe(0);
      expect(result.p50).toBe(0);
      expect(result.p95).toBe(0);
      expect(result.p99).toBe(0);
    });

    it('handles single metric', () => {
      const metrics = [buildMetric(42)];
      const result = MetricAggregator.aggregateHistogram(metrics);

      expect(result.sum).toBe(42);
      expect(result.count).toBe(1);
      expect(result.min).toBe(42);
      expect(result.max).toBe(42);
      expect(result.avg).toBe(42);
    });

    it('filters out NaN values', () => {
      const metrics = [buildMetric(10), buildMetric(NaN), buildMetric(30)];
      const result = MetricAggregator.aggregateHistogram(metrics);

      expect(result.count).toBe(2);
      expect(result.sum).toBe(40);
    });
  });

  describe('aggregateCounter', () => {
    it('sums counter metric values', () => {
      const metrics = [
        buildMetric(100, Date.now() * 1_000_000),
        buildMetric(200, Date.now() * 1_000_000 + 1_000_000),
        buildMetric(300, Date.now() * 1_000_000 + 2_000_000),
      ];

      const result = MetricAggregator.aggregateCounter(metrics);

      expect(result).toBe(600);
    });

    it('returns 0 for empty array', () => {
      const result = MetricAggregator.aggregateCounter([]);
      expect(result).toBe(0);
    });

    it('handles zero values', () => {
      const metrics = [buildMetric(0), buildMetric(0)];
      const result = MetricAggregator.aggregateCounter(metrics);
      expect(result).toBe(0);
    });
  });

  describe('aggregateGauge', () => {
    it('returns latest value by default', () => {
      const now = Date.now() * 1_000_000;
      const metrics = [
        buildMetric(100, now - 2_000_000_000),
        buildMetric(200, now - 1_000_000_000),
        buildMetric(300, now),
      ];

      const result = MetricAggregator.aggregateGauge(metrics);

      expect(result).toBe(300);
    });

    it('returns average when method is avg', () => {
      const metrics = [buildMetric(10), buildMetric(20), buildMetric(30)];

      const result = MetricAggregator.aggregateGauge(metrics, 'avg');

      expect(result).toBe(20);
    });

    it('returns 0 for empty array', () => {
      const result = MetricAggregator.aggregateGauge([]);
      expect(result).toBe(0);
    });
  });

  describe('aggregateSummary', () => {
    it('aggregates summary metrics with sum, count, min, max, avg', () => {
      const metrics = [
        buildMetric(10),
        buildMetric(20),
        buildMetric(30),
        buildMetric(40),
        buildMetric(50),
      ];

      const result = MetricAggregator.aggregateSummary(metrics);

      expect(result.sum).toBe(150);
      expect(result.count).toBe(5);
      expect(result.min).toBe(10);
      expect(result.max).toBe(50);
      expect(result.avg).toBe(30);
    });

    it('returns zero values for empty array', () => {
      const result = MetricAggregator.aggregateSummary([]);

      expect(result.sum).toBe(0);
      expect(result.count).toBe(0);
      expect(result.min).toBe(0);
      expect(result.max).toBe(0);
      expect(result.avg).toBe(0);
    });
  });

  describe('aggregate', () => {
    it('aggregates histogram type', () => {
      const metrics = [buildMetric(10), buildMetric(20)];
      const result = MetricAggregator.aggregate(metrics, 'histogram');

      expect(result).toHaveProperty('sum');
      expect(result).toHaveProperty('p50');
      expect(result).toHaveProperty('p95');
    });

    it('aggregates counter type', () => {
      const metrics = [buildMetric(10), buildMetric(20)];
      const result = MetricAggregator.aggregate(metrics, 'counter');

      expect(result).toBe(30);
    });

    it('aggregates gauge type', () => {
      const metrics = [buildMetric(10), buildMetric(20)];
      const result = MetricAggregator.aggregate(metrics, 'gauge');

      expect(typeof result).toBe('number');
    });

    it('aggregates summary type', () => {
      const metrics = [buildMetric(10), buildMetric(20)];
      const result = MetricAggregator.aggregate(metrics, 'summary');

      expect(result).toHaveProperty('sum');
      expect(result).toHaveProperty('avg');
    });

    it('defaults to gauge aggregation for unknown type', () => {
      const metrics = [buildMetric(10), buildMetric(20)];
      const result = MetricAggregator.aggregate(metrics, 'unknown');

      expect(typeof result).toBe('number');
    });
  });
});
