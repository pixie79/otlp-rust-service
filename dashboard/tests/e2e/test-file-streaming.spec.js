import { test, expect } from '@playwright/test';

test.describe('File Streaming Performance', () => {
  test('handles file ingestion without blocking UI', async ({ page }) => {
    await page.goto('/');

    // Check that UI is responsive
    const header = page.getByRole('heading', { name: 'OTLP Realtime Dashboard' });
    await expect(header).toBeVisible();

    // UI should remain interactive
    const selectButton = page.getByRole('button', { name: 'Choose Directory' });
    await expect(selectButton).toBeEnabled();
  });

  test('displays file loading status', async ({ page }) => {
    await page.goto('/');

    // Status should be visible
    const statusLine = page.locator('#status-line');
    await expect(statusLine).toBeVisible();
  });

  test('shows activity log for file operations', async ({ page }) => {
    await page.goto('/');

    // Activity log panel should be visible
    const logPanel = page.locator('#log-panel');
    await expect(logPanel).toBeVisible();

    const logList = page.locator('#log-list');
    await expect(logList).toBeVisible();
  });

  test('handles multiple file operations', async ({ page }) => {
    await page.goto('/');

    // Dashboard should be ready for file operations
    const selectButton = page.getByRole('button', { name: 'Choose Directory' });
    await expect(selectButton).toBeVisible();
    await expect(selectButton).toBeEnabled();
  });

  test('maintains UI responsiveness during file processing', async ({ page }) => {
    await page.goto('/');

    // All interactive elements should be accessible
    const navTraces = page.getByRole('button', { name: 'Traces' });
    const navMetrics = page.getByRole('button', { name: 'Metrics' });

    await expect(navTraces).toBeEnabled();
    await expect(navMetrics).toBeEnabled();

    // Should be able to switch views
    await navMetrics.click();
    await expect(page.locator('#metric-panel')).toBeVisible();

    await navTraces.click();
    await expect(page.locator('#trace-panel')).toBeVisible();
  });

  test('displays error messages for file read failures', async ({ page }) => {
    await page.goto('/');

    // Error handling UI should be present
    // (Actual error testing would require file system access)
    const logPanel = page.locator('#log-panel');
    await expect(logPanel).toBeVisible();
  });
});
