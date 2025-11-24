import { describe, it, expect, beforeEach } from 'vitest';
import { Loading } from '../../../src/ui/loading.js';
import { FileReadError } from '../../../src/error.js';

describe('Loading', () => {
  let container;

  beforeEach(() => {
    document.body.innerHTML = '<div id="loading-container"></div>';
    container = document.getElementById('loading-container');
  });

  it('creates loading component with container element', () => {
    const loading = new Loading(container);
    expect(loading.container).toBe(container);
  });

  it('throws error if container is missing', () => {
    expect(() => new Loading(null)).toThrow('Loading requires a container element');
  });

  it('shows loading state', () => {
    const loading = new Loading(container);
    loading.show('Loading data...');

    expect(loading.isLoading).toBe(true);
    expect(container.innerHTML).toContain('Loading data...');
    expect(container.querySelector('[role="status"]')).toBeTruthy();
    expect(container.querySelector('[aria-busy="true"]')).toBeTruthy();
  });

  it('hides loading state', () => {
    const loading = new Loading(container);
    loading.show();
    loading.hide();

    expect(loading.isLoading).toBe(false);
    expect(container.innerHTML).toBe('');
  });

  it('shows error message', () => {
    const loading = new Loading(container);
    const error = new Error('Something went wrong');
    loading.showError(error);

    expect(loading.isLoading).toBe(false);
    expect(loading.error).toBeTruthy();
    expect(container.innerHTML).toContain('Something went wrong');
    expect(container.querySelector('[role="alert"]')).toBeTruthy();
  });

  it('shows error from string', () => {
    const loading = new Loading(container);
    loading.showError('Custom error message', { name: 'CustomError' });

    expect(loading.error.message).toBe('Custom error message');
    expect(loading.error.name).toBe('CustomError');
    expect(container.innerHTML).toContain('Custom error message');
  });

  it('shows error details when provided', () => {
    const loading = new Loading(container);
    const error = new Error('Test error');
    loading.showError(error, { details: 'Stack trace here' });

    expect(container.innerHTML).toContain('Stack trace here');
    expect(container.querySelector('.error-state__details')).toBeTruthy();
  });

  it('applies error-specific CSS classes', () => {
    const loading = new Loading(container);
    const fileError = new FileReadError('test.arrow', new Error('Read failed'));
    loading.showError(fileError);

    expect(container.querySelector('.error-state--file-read')).toBeTruthy();
  });

  it('clears error', () => {
    const loading = new Loading(container);
    loading.showError('Test error');
    loading.clearError();

    expect(loading.error).toBeNull();
    expect(container.innerHTML).toBe('');
  });

  it('escapes HTML in error messages', () => {
    const loading = new Loading(container);
    loading.showError('<script>alert("xss")</script>');

    expect(container.innerHTML).not.toContain('<script>');
    expect(container.innerHTML).toContain('&lt;script&gt;');
  });
});
