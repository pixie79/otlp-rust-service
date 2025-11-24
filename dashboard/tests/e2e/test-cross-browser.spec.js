/**
 * E2E tests for cross-browser compatibility
 * Tests core functionality across different browsers
 */

import { test, expect, devices } from '@playwright/test';

// Test on different browser engines
const browsers = [
  { name: 'chromium', ...devices['Desktop Chrome'] },
  { name: 'firefox', ...devices['Desktop Firefox'] },
  { name: 'webkit', ...devices['Desktop Safari'] },
];

for (const browser of browsers) {
  test.describe(`${browser.name} Browser Tests`, () => {
    test.use({ ...browser });

    test('should load dashboard', async ({ page }) => {
      await page.goto('http://localhost:8080');
      await expect(page.locator('h1')).toContainText('OTLP Realtime Dashboard');
    });

    test('should check browser compatibility', async ({ page }) => {
      await page.goto('http://localhost:8080');

      // Check that WebAssembly is available
      const wasmAvailable = await page.evaluate(() => typeof WebAssembly !== 'undefined');
      expect(wasmAvailable).toBe(true);

      // Check that Workers are available
      const workersAvailable = await page.evaluate(() => typeof Worker !== 'undefined');
      expect(workersAvailable).toBe(true);

      // Check that localStorage is available
      const localStorageAvailable = await page.evaluate(() => typeof Storage !== 'undefined');
      expect(localStorageAvailable).toBe(true);
    });

    test('should handle directory selection', async ({ page, context }) => {
      await page.goto('http://localhost:8080');

      // Grant file system access (if supported)
      await context.grantPermissions(['read']);

      const selectButton = page.getByRole('button', { name: /choose directory/i });
      await expect(selectButton).toBeVisible();
    });

    test('should render trace list', async ({ page }) => {
      await page.goto('http://localhost:8080');

      const tracePanel = page.getByRole('tabpanel', { name: /traces/i });
      await expect(tracePanel).toBeVisible();
    });

    test('should render metrics view', async ({ page }) => {
      await page.goto('http://localhost:8080');

      // Switch to metrics view
      await page.getByRole('tab', { name: /metrics/i }).click();

      const metricPanel = page.getByRole('tabpanel', { name: /metrics/i });
      await expect(metricPanel).toBeVisible();
    });
  });
}
