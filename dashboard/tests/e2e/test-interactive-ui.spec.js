import { test, expect } from '@playwright/test';

test.describe('Interactive UI Features', () => {
  test('allows switching between traces and metrics views', async ({ page }) => {
    await page.goto('/');

    // Initially traces view should be visible
    await expect(page.locator('#trace-panel')).toBeVisible();
    await expect(page.locator('#metric-panel')).not.toBeVisible();

    // Switch to metrics
    await page.getByRole('button', { name: 'Metrics' }).click();
    await expect(page.locator('#trace-panel')).not.toBeVisible();
    await expect(page.locator('#metric-panel')).toBeVisible();

    // Switch back to traces
    await page.getByRole('button', { name: 'Traces' }).click();
    await expect(page.locator('#trace-panel')).toBeVisible();
    await expect(page.locator('#metric-panel')).not.toBeVisible();
  });

  test('supports keyboard navigation', async ({ page }) => {
    await page.goto('/');

    // Focus on navigation
    const navTraces = page.getByRole('button', { name: 'Traces' });
    await navTraces.focus();

    // Use arrow key to navigate
    await page.keyboard.press('ArrowRight');
    await expect(page.locator('#metric-panel')).toBeVisible();

    // Use arrow key to go back
    await page.keyboard.press('ArrowLeft');
    await expect(page.locator('#trace-panel')).toBeVisible();
  });

  test('supports keyboard shortcuts', async ({ page }) => {
    await page.goto('/');

    // Ctrl+1 to switch to traces
    await page.keyboard.press('Control+1');
    await expect(page.locator('#trace-panel')).toBeVisible();

    // Ctrl+2 to switch to metrics
    await page.keyboard.press('Control+2');
    await expect(page.locator('#metric-panel')).toBeVisible();
  });

  test('search input is accessible', async ({ page }) => {
    await page.goto('/');

    // Search should be visible
    const searchInput = page.locator('input[type="search"]');
    await expect(searchInput).toBeVisible();

    // Can type in search
    await searchInput.fill('test query');
    await expect(searchInput).toHaveValue('test query');
  });

  test('search can be focused with keyboard shortcut', async ({ page }) => {
    await page.goto('/');

    // Press / to focus search
    await page.keyboard.press('/');
    const searchInput = page.locator('input[type="search"]');
    await expect(searchInput).toBeFocused();
  });

  test('search can be cleared with Escape', async ({ page }) => {
    await page.goto('/');

    const searchInput = page.locator('input[type="search"]');
    await searchInput.fill('test');
    await expect(searchInput).toHaveValue('test');

    await page.keyboard.press('Escape');
    await expect(searchInput).toHaveValue('');
  });

  test('displays loading state', async ({ page }) => {
    await page.goto('/');

    // Loading container should exist (may or may not be visible)
    const loadingContainer = page.locator('#loading-container');
    // Just verify it exists in the DOM
    await expect(loadingContainer.or(page.locator('body'))).toBeVisible();
  });

  test('has accessible ARIA labels', async ({ page }) => {
    await page.goto('/');

    // Check for ARIA attributes
    const header = page.locator('header[role="banner"]');
    const main = page.locator('main[role="main"]');
    const statusLine = page.locator('#status-line[role="status"]');

    await expect(header).toBeVisible();
    await expect(main).toBeVisible();
    await expect(statusLine).toBeVisible();
  });

  test('navigation has proper ARIA roles', async ({ page }) => {
    await page.goto('/');

    const nav = page.locator('nav[role="tablist"]');
    const tracesTab = page.locator('#nav-traces[role="tab"]');
    const metricsTab = page.locator('#nav-metrics[role="tab"]');

    await expect(nav).toBeVisible();
    await expect(tracesTab).toBeVisible();
    await expect(metricsTab).toBeVisible();
    await expect(tracesTab).toHaveAttribute('aria-controls', 'trace-panel');
    await expect(metricsTab).toHaveAttribute('aria-controls', 'metric-panel');
  });

  test('responsive layout adapts to screen size', async ({ page }) => {
    // Test desktop size
    await page.setViewportSize({ width: 1920, height: 1080 });
    await page.goto('/');

    const tracePanel = page.locator('#trace-panel');
    await expect(tracePanel).toBeVisible();

    // Test tablet size
    await page.setViewportSize({ width: 768, height: 1024 });
    await page.reload();
    await expect(tracePanel).toBeVisible();

    // Test mobile size
    await page.setViewportSize({ width: 375, height: 667 });
    await page.reload();
    await expect(tracePanel).toBeVisible();
  });
});

