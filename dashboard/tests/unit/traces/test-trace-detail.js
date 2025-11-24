import { describe, it, expect, beforeEach } from 'vitest';
import { TraceDetail } from '../../../src/traces/trace-detail.js';

const sampleTrace = {
  traceId: 'abc123',
  spanId: 'def456',
  parentSpanId: null,
  name: 'GET /checkout',
  serviceName: 'checkout',
  startTime: 1700000000000000000,
  endTime: 1700000000005000000,
  duration: 5,
  statusCode: 'error',
  statusMessage: 'boom',
  attributes: { 'service.name': 'checkout', 'http.method': 'GET' },
  events: [{ name: 'exception', attributes: { message: 'boom' } }],
  links: [],
  error: true,
};

describe('TraceDetail', () => {
  let container;

  beforeEach(() => {
    document.body.innerHTML = '<div id="trace-detail"></div>';
    container = document.getElementById('trace-detail');
  });

  it('renders trace details with attributes and events', () => {
    const detail = new TraceDetail(container);
    detail.showTrace(sampleTrace);

    expect(container.querySelector('.trace-detail__title').textContent).toContain('GET /checkout');
    expect(container.querySelectorAll('.attribute-item').length).toBeGreaterThan(0);
    expect(container.querySelector('.status-chip').textContent).toContain('ERROR');
  });

  it('clears details when clear is called', () => {
    const detail = new TraceDetail(container);
    detail.showTrace(sampleTrace);
    detail.clear();

    expect(container.textContent).toContain('Select a trace');
  });
});
