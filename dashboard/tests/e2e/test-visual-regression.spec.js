/**
 * E2E tests for visual regression testing using Playwright
 * Optional: Can be integrated with Percy, Chromatic, or use Playwright's built-in screenshot comparison
 * 
 * To use with Percy:
 * 1. Install: npm install --save-dev @percy/playwright
 * 2. Configure percy.config.js
 * 3. Run: npx percy exec -- playwright test
 * 
 * To use with Chromatic:
 * 1. Install: npm install --save-dev chromatic
 * 2. Configure chromatic.config.js
 * 3. Run: npx chromatic --playwright
 * 
 * To use Playwright's built-in screenshot comparison:
 * 1. Use expect(page).toHaveScreenshot() in tests
 * 2. Screenshots are stored in test-results/
 */

import { test, expect } from '@playwright/test';

test.describe('Visual Regression Tests', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('http://localhost:8080');
  });

  test('should match dashboard home screen', async ({ page }) => {
    // Wait for dashboard to load
    await page.waitForSelector('h1:has-text("OTLP Realtime Dashboard")');
    
    // Take screenshot and compare (Playwright built-in)
    await expect(page).toHaveScreenshot('dashboard-home.png', {
      fullPage: true,
    });
  });

  test('should match traces view', async ({ page }) => {
    await page.goto('http://localhost:8080');
    await page.waitForSelector('h1:has-text("OTLP Realtime Dashboard")');
    
    // Switch to traces view
    await page.getByRole('tab', { name: /traces/i }).click();
    await page.waitForSelector('[data-testid="trace-panel"]');
    
    // Take screenshot
    await expect(page).toHaveScreenshot('dashboard-traces-view.png', {
      fullPage: true,
    });
  });

  test('should match metrics view', async ({ page }) => {
    await page.goto('http://localhost:8080');
    await page.waitForSelector('h1:has-text("OTLP Realtime Dashboard")');
    
    // Switch to metrics view
    await page.getByRole('tab', { name: /metrics/i }).click();
    await page.waitForSelector('[role="tabpanel"][aria-labelledby="nav-metrics"]');
    
    // Take screenshot
    await expect(page).toHaveScreenshot('dashboard-metrics-view.png', {
      fullPage: true,
    });
  });

  test('should match settings panel', async ({ page }) => {
    await page.goto('http://localhost:8080');
    await page.waitForSelector('h1:has-text("OTLP Realtime Dashboard")');
    
    // Open settings
    await page.getByRole('button', { name: /settings/i }).click();
    await page.waitForSelector('#settings-container');
    
    // Take screenshot
    await expect(page).toHaveScreenshot('dashboard-settings.png', {
      fullPage: true,
    });
  });

  test('should match trace detail pane', async ({ page }) => {
    await page.goto('http://localhost:8080');
    await page.waitForSelector('h1:has-text("OTLP Realtime Dashboard")');
    
    // If there are traces, select one
    const traceRow = page.locator('.trace-row').first();
    if (await traceRow.count() > 0) {
      await traceRow.click();
      await page.waitForSelector('#trace-detail');
      
      // Take screenshot of detail pane
      const detailPane = page.locator('#trace-detail');
      await expect(detailPane).toHaveScreenshot('trace-detail-pane.png');
    }
  });

  // Component-level visual tests
  test('should match navigation buttons', async ({ page }) => {
    await page.goto('http://localhost:8080');
    await page.waitForSelector('.view-nav');
    
    const nav = page.locator('.view-nav');
    await expect(nav).toHaveScreenshot('navigation-buttons.png');
  });

  test('should match search bar', async ({ page }) => {
    await page.goto('http://localhost:8080');
    await page.waitForSelector('#search-container');
    
    const searchBar = page.locator('#search-container');
    await expect(searchBar).toHaveScreenshot('search-bar.png');
  });
});

// Example with Percy integration (uncomment if using Percy)
/*
import { percySnapshot } from '@percy/playwright';

test('should match dashboard with Percy', async ({ page }) => {
  await page.goto('http://localhost:8080');
  await page.waitForSelector('h1:has-text("OTLP Realtime Dashboard")');
  await percySnapshot(page, 'Dashboard Home');
});
*/

// Example with Chromatic integration (uncomment if using Chromatic)
/*
import { test as base } from '@playwright/test';
import { chromatic } from 'chromatic/playwright';

const test = base.extend(chromatic);

test('should match dashboard with Chromatic', async ({ page, chromatic }) => {
  await page.goto('http://localhost:8080');
  await page.waitForSelector('h1:has-text("OTLP Realtime Dashboard")');
  await chromatic.snapshot(page, 'Dashboard Home');
});
*/

