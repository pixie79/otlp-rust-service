import { describe, it, expect, beforeEach, vi } from 'vitest';
import { TraceFilter } from '../../../src/traces/trace-filter.js';

describe('TraceFilter', () => {
  let container;

  beforeEach(() => {
    document.body.innerHTML = '<div id="trace-filter"></div>';
    container = document.getElementById('trace-filter');
  });

  it('emits filter changes when inputs change', () => {
    const onChange = vi.fn();
    const filter = new TraceFilter(container);
    filter.onChange = onChange;

    container.querySelector('input[name="traceId"]').value = 'abc';
    container.querySelector('input[name="traceId"]').dispatchEvent(new Event('input'));

    expect(onChange).toHaveBeenCalledWith(expect.objectContaining({ traceId: 'abc' }));
  });

  it('supports setting filters programmatically', () => {
    const filter = new TraceFilter(container);
    filter.setFilters({ serviceName: 'checkout', errorOnly: true });

    expect(filter.getFilters()).toMatchObject({ serviceName: 'checkout', errorOnly: true });
    expect(container.querySelector('input[name="serviceName"]').value).toBe('checkout');
    expect(container.querySelector('input[name="errorOnly"]').checked).toBe(true);
  });
});
