import { describe, it, expect, vi } from 'vitest';
import { MetricQuery } from '../../src/metrics/metric-query.js';
import { createMetricEntry } from '../../src/metrics/metric-entry.js';

const buildMetricRow = (overrides = {}) => ({
  metric_name: 'request_duration',
  value: 100,
  timestamp_unix_nano: BigInt(Date.now() * 1_000_000),
  labels: JSON.stringify({ service: 'api' }),
  metric_type: 'gauge',
  unit: 'ms',
  ...overrides,
});

describe('Metric data flow integration', () => {
  it('queries metrics from multiple tables and combines results', async () => {
    const executor = {
      execute: vi.fn(async (sql, params) => {
        if (sql.includes('table_a')) {
          return {
            rows: [
              buildMetricRow({
                metric_name: 'request_duration',
                value: 100,
                timestamp_unix_nano: BigInt(Date.now() * 1_000_000),
              }),
            ],
          };
        }

        return {
          rows: [
            buildMetricRow({
              metric_name: 'request_duration',
              value: 200,
              timestamp_unix_nano: BigInt(Date.now() * 1_000_000 + 1_000_000),
            }),
          ],
        };
      }),
    };

    const query = new MetricQuery({ execute: executor.execute });
    const metrics = await query.queryMetricsFromTables(['table_a', 'table_b'], {
      metricName: 'request_duration',
    });

    expect(executor.execute).toHaveBeenCalledTimes(2);
    expect(metrics.length).toBe(2);
    expect(metrics[0].metricName).toBe('request_duration');
    expect(metrics[1].metricName).toBe('request_duration');
  });

  it('filters metrics by metric name', async () => {
    const executor = {
      execute: vi.fn(async () => ({
        rows: [
          buildMetricRow({ metric_name: 'request_duration', value: 100 }),
          buildMetricRow({ metric_name: 'error_rate', value: 0.5 }),
        ],
      })),
    };

    const query = new MetricQuery({ execute: executor.execute });
    const metrics = await query.queryMetricsFromTables(['table_a'], {
      metricName: 'request_duration',
    });

    const callArgs = executor.execute.mock.calls[0];
    expect(callArgs[0]).toContain("metric_name = ?");
    expect(callArgs[1]).toContain('request_duration');
  });

  it('filters metrics by time range', async () => {
    const now = Date.now() * 1_000_000;
    const executor = {
      execute: vi.fn(async () => ({
        rows: [
          buildMetricRow({ timestamp_unix_nano: BigInt(now - 30 * 60 * 1_000_000_000) }),
          buildMetricRow({ timestamp_unix_nano: BigInt(now) }),
        ],
      })),
    };

    const query = new MetricQuery({ execute: executor.execute });
    const metrics = await query.queryMetricsFromTables(['table_a'], {
      timeRange: {
        start: now - 60 * 60 * 1_000_000_000, // 1 hour ago
        end: now,
      },
    });

    const callArgs = executor.execute.mock.calls[0];
    expect(callArgs[0]).toContain('timestamp_unix_nano >= ?');
    expect(callArgs[0]).toContain('timestamp_unix_nano <= ?');
  });

  it('gets available metrics from multiple tables', async () => {
    const executor = {
      execute: vi.fn(async (sql) => {
        if (sql.includes('table_a')) {
          return {
            rows: [
              { metric_name: 'request_duration' },
              { metric_name: 'error_rate' },
            ],
          };
        }
        return {
          rows: [
            { metric_name: 'error_rate' },
            { metric_name: 'throughput' },
          ],
        };
      }),
    };

    const query = new MetricQuery({ execute: executor.execute });
    const metrics = await query.getAvailableMetricsFromTables(['table_a', 'table_b']);

    expect(executor.execute).toHaveBeenCalledTimes(2);
    expect(metrics).toContain('request_duration');
    expect(metrics).toContain('error_rate');
    expect(metrics).toContain('throughput');
    expect(metrics.length).toBe(3); // No duplicates
  });

  it('handles empty tables gracefully', async () => {
    const executor = {
      execute: vi.fn(async () => ({ rows: [] })),
    };

    const query = new MetricQuery({ execute: executor.execute });
    const metrics = await query.queryMetricsFromTables([], {});

    expect(metrics).toEqual([]);
    expect(executor.execute).not.toHaveBeenCalled();
  });

  it('transforms database rows to MetricEntry objects', async () => {
    const executor = {
      execute: vi.fn(async () => ({
        rows: [
          buildMetricRow({
            metric_name: 'test_metric',
            value: 42,
            unit: 'bytes',
            metric_type: 'gauge',
          }),
        ],
      })),
    };

    const query = new MetricQuery({ execute: executor.execute });
    const metrics = await query.queryMetricsFromTables(['table_a'], {});

    expect(metrics.length).toBe(1);
    expect(metrics[0]).toMatchObject({
      metricName: 'test_metric',
      value: 42,
      unit: 'bytes',
      metricType: 'gauge',
    });
  });
});

