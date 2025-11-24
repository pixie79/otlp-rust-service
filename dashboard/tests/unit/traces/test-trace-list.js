import { randomUUID } from 'crypto';
import { describe, it, expect, beforeEach, vi } from 'vitest';
import { TraceList } from '../../../src/traces/trace-list.js';

const buildTrace = (overrides = {}) => ({
  traceId: randomUUID().replace(/-/g, '').slice(0, 32),
  spanId: randomUUID().replace(/-/g, '').slice(0, 16),
  parentSpanId: null,
  name: 'span-operation',
  serviceName: 'orders-api',
  startTime: Date.now() * 1_000_000,
  endTime: Date.now() * 1_000_000 + 5_000_000,
  duration: 5,
  statusCode: 'ok',
  statusMessage: null,
  attributes: {},
  error: false,
  ...overrides,
});

describe('TraceList', () => {
  let container;

  beforeEach(() => {
    document.body.innerHTML = '<div id="trace-list"></div>';
    container = document.getElementById('trace-list');
  });

  it('renders traces with virtualization window', () => {
    const traces = Array.from({ length: 200 }, (_, idx) => buildTrace({ name: `span-${idx}` }));
    const list = new TraceList(container, { rowHeight: 20, windowSize: 10 });

    list.setTraces(traces);
    const renderedRows = container.querySelectorAll('.trace-row');
    expect(renderedRows.length).toBeLessThanOrEqual(list.windowSize + list.virtualBuffer * 2);
    expect(renderedRows[0].querySelector('.span-name').textContent).toContain('span-');
  });

  it('applies filters and updates filtered trace set', () => {
    const traces = [
      buildTrace({ traceId: 'aaa', serviceName: 'checkout', error: false }),
      buildTrace({ traceId: 'bbb', serviceName: 'payments', error: true }),
    ];
    const list = new TraceList(container);
    list.setTraces(traces);
    list.applyFilters({ serviceName: 'pay', errorOnly: true });

    expect(list.filteredTraces.length).toBe(1);
    expect(list.filteredTraces[0].traceId).toBe('bbb');
  });

  it('tracks selection and exposes selected trace', () => {
    const traces = [buildTrace({ traceId: 'select-me' })];
    const list = new TraceList(container);
    list.setTraces(traces);

    const onTraceSelected = vi.fn();
    list.onTraceSelected = onTraceSelected;

    const row = container.querySelector('.trace-row');
    row.click();

    expect(list.getSelectedTrace().traceId).toBe('select-me');
    expect(onTraceSelected).toHaveBeenCalledWith(expect.objectContaining({ traceId: 'select-me' }));
  });

  it('binds to worker client to receive realtime batches', () => {
    const worker = {
      subscribe: vi.fn((event, cb) => {
        if (event === 'TRACE_BATCH') {
          cb({ traces: [buildTrace({ traceId: 'worker-trace' })] });
        }
        return () => {};
      }),
    };

    const list = new TraceList(container);
    list.bindWorker(worker);

    expect(worker.subscribe).toHaveBeenCalledWith('TRACE_BATCH', expect.any(Function));
    expect(list.traces[0].traceId).toBe('worker-trace');
  });
});
