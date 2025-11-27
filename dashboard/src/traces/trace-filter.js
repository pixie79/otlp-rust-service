const template = () => `
  <form class="trace-filter__form">
    <div class="field-group">
      <label>
        Trace ID
        <input type="text" name="traceId" placeholder="abc123" autocomplete="off" />
      </label>
      <label>
        Service
        <input type="text" name="serviceName" placeholder="checkout" autocomplete="off" />
      </label>
      <label>
        Span Name
        <input type="text" name="spanName" placeholder="GET /checkout" autocomplete="off" />
      </label>
    </div>
    <div class="field-group">
      <label>
        Min Duration (ms)
        <input type="number" name="minDuration" min="0" />
      </label>
      <label>
        Max Duration (ms)
        <input type="number" name="maxDuration" min="0" />
      </label>
      <label class="checkbox">
        <input type="checkbox" name="errorOnly" />
        Errors only
      </label>
      <label class="checkbox">
        <input type="checkbox" name="liveTail" id="live-tail-toggle" />
        Live Tail
      </label>
    </div>
  </form>
`;

const parseNumber = (value) => {
  if (value === '' || value === null || value === undefined) {
    return null;
  }
  const parsed = Number(value);
  return Number.isNaN(parsed) ? null : parsed;
};

export class TraceFilter {
  constructor(container) {
    if (!container) {
      throw new Error('TraceFilter requires a container element');
    }
    this.container = container;
    this.container.classList.add('trace-filter');
    this.onChange = () => {};
    this.onLiveTailToggle = () => {};
    this._filters = {
      traceId: '',
      serviceName: '',
      spanName: '',
      minDuration: null,
      maxDuration: null,
      errorOnly: false,
      liveTail: false,
    };
    this._render();
  }

  _render() {
    this.container.innerHTML = template();
    this.form = this.container.querySelector('form');
    this.form.addEventListener('submit', (event) => event.preventDefault());
    this.form.addEventListener('input', () => this._handleChange());
    this.form.addEventListener('change', () => this._handleChange());

    // Handle live tail toggle separately
    const liveTailToggle = this.container.querySelector('#live-tail-toggle');
    if (liveTailToggle) {
      liveTailToggle.addEventListener('change', (e) => {
        this._filters.liveTail = e.target.checked;
        this.onLiveTailToggle?.(e.target.checked);
      });
    }
  }

  _handleChange() {
    this._filters = {
      traceId: this.form.elements.traceId.value.trim(),
      serviceName: this.form.elements.serviceName.value.trim(),
      spanName: this.form.elements.spanName.value.trim(),
      minDuration: parseNumber(this.form.elements.minDuration.value),
      maxDuration: parseNumber(this.form.elements.maxDuration.value),
      errorOnly: this.form.elements.errorOnly.checked,
      liveTail: this.form.elements.liveTail?.checked ?? false,
    };
    this.onChange?.(this.getFilters());
  }

  setFilters(filters = {}) {
    this._filters = { ...this._filters, ...filters };
    this.form.elements.traceId.value = this._filters.traceId ?? '';
    this.form.elements.serviceName.value = this._filters.serviceName ?? '';
    this.form.elements.spanName.value = this._filters.spanName ?? '';
    this.form.elements.minDuration.value = this._filters.minDuration ?? '';
    this.form.elements.maxDuration.value = this._filters.maxDuration ?? '';
    this.form.elements.errorOnly.checked = Boolean(this._filters.errorOnly);
  }

  getFilters() {
    return { ...this._filters };
  }
}
