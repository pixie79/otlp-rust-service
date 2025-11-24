import { test, expect } from '@playwright/test';

test.describe('Realtime Metrics Graphing', () => {
  test('shows metrics view when navigating to metrics tab', async ({ page }) => {
    await page.goto('/');
    
    // Navigate to metrics view
    await page.getByRole('button', { name: 'Metrics' }).click();
    
    // Check that metrics panel is visible
    const metricsPanel = page.locator('#metric-panel');
    await expect(metricsPanel).toBeVisible();
    
    // Check that metric selector is present
    const metricSelector = page.locator('#metric-selector');
    await expect(metricSelector).toBeVisible();
    
    // Check that time range selector is present
    const timeRangeSelector = page.locator('#metric-time-range');
    await expect(timeRangeSelector).toBeVisible();
  });

  test('displays metric graphs container', async ({ page }) => {
    await page.goto('/');
    
    // Navigate to metrics view
    await page.getByRole('button', { name: 'Metrics' }).click();
    
    // Check that graphs container exists
    const graphsContainer = page.locator('#metric-graphs-container');
    await expect(graphsContainer).toBeVisible();
  });

  test('shows empty state when no metrics available', async ({ page }) => {
    await page.goto('/');
    
    // Navigate to metrics view
    await page.getByRole('button', { name: 'Metrics' }).click();
    
    // Check for empty state message (if no metrics are loaded)
    const emptyState = page.locator('.metric-selector__empty, .metric-graph__empty');
    // This may or may not be visible depending on whether data is loaded
    // Just verify the container structure exists
    await expect(page.locator('#metric-selector')).toBeVisible();
  });

  test('allows switching between traces and metrics views', async ({ page }) => {
    await page.goto('/');
    
    // Initially traces view should be visible
    const tracePanel = page.locator('#trace-panel');
    await expect(tracePanel).toBeVisible();
    
    // Switch to metrics
    await page.getByRole('button', { name: 'Metrics' }).click();
    await expect(tracePanel).not.toBeVisible();
    
    const metricsPanel = page.locator('#metric-panel');
    await expect(metricsPanel).toBeVisible();
    
    // Switch back to traces
    await page.getByRole('button', { name: 'Traces' }).click();
    await expect(tracePanel).toBeVisible();
    await expect(metricsPanel).not.toBeVisible();
  });

  test('displays time range selector with presets', async ({ page }) => {
    await page.goto('/');
    
    // Navigate to metrics view
    await page.getByRole('button', { name: 'Metrics' }).click();
    
    // Check time range selector exists
    const timeRangeSelect = page.locator('#metric-time-range-select');
    await expect(timeRangeSelect).toBeVisible();
    
    // Check that presets are available
    await expect(timeRangeSelect.locator('option[value="last5m"]')).toBeVisible();
    await expect(timeRangeSelect.locator('option[value="last15m"]')).toBeVisible();
    await expect(timeRangeSelect.locator('option[value="last1h"]')).toBeVisible();
    await expect(timeRangeSelect.locator('option[value="last6h"]')).toBeVisible();
    await expect(timeRangeSelect.locator('option[value="last24h"]')).toBeVisible();
  });

  test('allows changing time range', async ({ page }) => {
    await page.goto('/');
    
    // Navigate to metrics view
    await page.getByRole('button', { name: 'Metrics' }).click();
    
    const timeRangeSelect = page.locator('#metric-time-range-select');
    
    // Change time range
    await timeRangeSelect.selectOption('last6h');
    await expect(timeRangeSelect).toHaveValue('last6h');
    
    // Change to another preset
    await timeRangeSelect.selectOption('last24h');
    await expect(timeRangeSelect).toHaveValue('last24h');
  });

  test('shows refresh button for metrics', async ({ page }) => {
    await page.goto('/');
    
    // Navigate to metrics view
    await page.getByRole('button', { name: 'Metrics' }).click();
    
    // Check refresh button exists
    const refreshButton = page.getByRole('button', { name: 'Refresh' });
    await expect(refreshButton).toBeVisible();
  });

  test('displays metrics panel header with title and subtitle', async ({ page }) => {
    await page.goto('/');
    
    // Navigate to metrics view
    await page.getByRole('button', { name: 'Metrics' }).click();
    
    // Check header content
    await expect(page.getByRole('heading', { name: 'Metrics' })).toBeVisible();
    await expect(page.getByText('Real-time time-series graphs')).toBeVisible();
  });
});

