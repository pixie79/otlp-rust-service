import { describe, it, expect, vi } from 'vitest';
import { TraceQuery } from '../../src/traces/trace-query.js';

const row = (overrides = {}) => ({
  trace_id: 'a'.repeat(32),
  span_id: 'b'.repeat(16),
  parent_span_id: null,
  name: 'operation',
  service_name: 'checkout',
  start_time_unix_nano: BigInt(1700000000000000000),
  end_time_unix_nano: BigInt(1700000000000500000),
  status_code: 'ok',
  status_message: null,
  attributes: JSON.stringify({ 'service.name': 'checkout' }),
  ...overrides,
});

describe('Trace data flow integration', () => {
  it('fetches traces from multiple tables and merges chronologically', async () => {
    const executor = {
      execute: vi.fn(async (sql) => {
        if (sql.includes('table_a')) {
          return {
            rows: [
              row({
                trace_id: 'aaa111',
                start_time_unix_nano: BigInt(1700000000001000000),
              }),
            ],
          };
        }

        return {
          rows: [
            row({
              trace_id: 'bbb222',
              start_time_unix_nano: BigInt(1700000000002000000),
              status_code: 'error',
            }),
          ],
        };
      }),
    };

    const query = new TraceQuery(executor, { limit: 10 });
    const traces = await query.fetchLatestFromTables(['table_a', 'table_b'], { errorOnly: false });

    expect(executor.execute).toHaveBeenCalledTimes(2);
    expect(traces[0].traceId).toBe('bbb222');
    expect(traces[1].traceId).toBe('aaa111');
  });
});
