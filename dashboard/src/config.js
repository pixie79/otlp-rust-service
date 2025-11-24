const DEFAULT_CONFIG = Object.freeze({
  pollingIntervalMs: 1_000,
  maxTraces: 10_000,
  maxGraphPoints: 10_000,
  maxLoadedFiles: 100,
  traceRetentionMinutes: 60,
  metricRetentionMinutes: 60,
});

const numberGuard = (value, fallback, { min = 0, max = Number.MAX_SAFE_INTEGER } = {}) => {
  if (typeof value !== 'number' || Number.isNaN(value)) {
    return fallback;
  }
  return Math.min(Math.max(value, min), max);
};

const normalizeConfig = (partial) => ({
  pollingIntervalMs: numberGuard(partial.pollingIntervalMs, DEFAULT_CONFIG.pollingIntervalMs, {
    min: 250,
    max: 10_000,
  }),
  maxTraces: numberGuard(partial.maxTraces, DEFAULT_CONFIG.maxTraces, {
    min: 100,
    max: 100_000,
  }),
  maxGraphPoints: numberGuard(partial.maxGraphPoints, DEFAULT_CONFIG.maxGraphPoints, {
    min: 1_000,
    max: 100_000,
  }),
  maxLoadedFiles: numberGuard(partial.maxLoadedFiles, DEFAULT_CONFIG.maxLoadedFiles, {
    min: 10,
    max: 10_000,
  }),
  traceRetentionMinutes: numberGuard(partial.traceRetentionMinutes, DEFAULT_CONFIG.traceRetentionMinutes, {
    min: 1,
    max: 24 * 60,
  }),
  metricRetentionMinutes: numberGuard(
    partial.metricRetentionMinutes,
    DEFAULT_CONFIG.metricRetentionMinutes,
    {
      min: 1,
      max: 24 * 60,
    },
  ),
});

export const createConfig = (overrides = {}) =>
  Object.freeze({
    ...DEFAULT_CONFIG,
    ...normalizeConfig({ ...DEFAULT_CONFIG, ...overrides }),
  });

export class ConfigManager {
  constructor(initialOverrides = {}) {
    this._config = createConfig(initialOverrides);
    this._listeners = new Set();
  }

  get(key) {
    return this._config[key];
  }

  getAll() {
    return { ...this._config };
  }

  update(partial = {}) {
    this._config = createConfig({ ...this._config, ...partial });
    this._emit();
    return this.getAll();
  }

  onChange(listener) {
    if (typeof listener !== 'function') {
      throw new Error('ConfigManager.onChange requires a callback function');
    }
    this._listeners.add(listener);
    return () => this._listeners.delete(listener);
  }

  _emit() {
    for (const listener of this._listeners) {
      listener(this.getAll());
    }
  }
}

export const configManager = new ConfigManager();
