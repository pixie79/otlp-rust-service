const escapeHtml = (value) =>
  String(value ?? '')
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;')
    .replace(/'/g, '&#39;');

const emptyState = () => `
  <div class="trace-detail__empty">
    <p>Select a trace to view details.</p>
  </div>
`;

export class TraceDetail {
  constructor(container) {
    if (!container) {
      throw new Error('TraceDetail requires a container element');
    }
    this.container = container;
    this.container.classList.add('trace-detail');
    this.clear();
  }

  showTrace(trace) {
    if (!trace) {
      this.clear();
      return;
    }

    const attributeItems = Object.entries(trace.attributes ?? {}).map(
      ([key, value]) => `
        <div class="attribute-item">
          <span class="attribute-key">${escapeHtml(key)}</span>
          <span class="attribute-value">${escapeHtml(value)}</span>
        </div>
      `
    );

    const eventItems = (trace.events ?? []).map(
      (event) => `
        <div class="event-item">
          <div class="event-name">${escapeHtml(event.name)}</div>
          <pre>${escapeHtml(JSON.stringify(event.attributes ?? {}, null, 2))}</pre>
        </div>
      `
    );

    const duration =
      typeof trace.duration === 'number' ? trace.duration : Number(trace.duration ?? 0);

    this.container.innerHTML = `
      <div class="trace-detail__header">
        <div>
          <h3 class="trace-detail__title">${escapeHtml(trace.name)}</h3>
          <p class="trace-detail__subtitle">${escapeHtml(trace.serviceName)}</p>
        </div>
        <span class="status-chip ${escapeHtml(trace.statusCode)}">${escapeHtml(
          trace.statusCode.toUpperCase()
        )}</span>
      </div>
      <div class="trace-detail__meta">
        <div><strong>Trace ID:</strong> ${escapeHtml(trace.traceId)}</div>
        <div><strong>Span ID:</strong> ${escapeHtml(trace.spanId)}</div>
        <div><strong>Duration:</strong> ${duration.toFixed(2)} ms</div>
        <div><strong>Start:</strong> ${new Date(trace.startTime / 1_000_000).toLocaleString()}</div>
      </div>
      <section>
        <h4>Attributes</h4>
        <div class="attribute-grid">
          ${attributeItems.join('')}
        </div>
      </section>
      ${
        eventItems.length
          ? `<section>
              <h4>Events</h4>
              <div class="events-stack">${eventItems.join('')}</div>
            </section>`
          : ''
      }
    `;
  }

  clear() {
    this.container.innerHTML = emptyState();
  }
}
