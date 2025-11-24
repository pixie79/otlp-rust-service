/**
 * E2E tests for accessibility using axe-core with Playwright
 * 
 * To use:
 * 1. Install: npm install --save-dev @axe-core/playwright
 * 2. Import and use in tests
 * 
 * Alternative: Use Pa11y with Playwright
 * 1. Install: npm install --save-dev pa11y
 * 2. Use pa11y CLI or programmatic API with Playwright
 */

import { test, expect } from '@playwright/test';
import AxeBuilder from '@axe-core/playwright';

test.describe('Accessibility Tests with axe-core', () => {
  test('should not have any automatically detectable accessibility issues', async ({ page }) => {
    await page.goto('http://localhost:8080');
    await page.waitForSelector('h1:has-text("OTLP Realtime Dashboard")');

    // Run axe-core accessibility scan
    const accessibilityScanResults = await new AxeBuilder({ page })
      .withTags(['wcag2a', 'wcag2aa', 'wcag21aa', 'best-practice'])
      .analyze();

    // Check for violations
    expect(accessibilityScanResults.violations).toEqual([]);
  });

  test('should meet WCAG 2.1 AA standards', async ({ page }) => {
    await page.goto('http://localhost:8080');
    await page.waitForSelector('h1:has-text("OTLP Realtime Dashboard")');

    const accessibilityScanResults = await new AxeBuilder({ page })
      .withTags(['wcag2aa', 'wcag21aa'])
      .analyze();

    // Filter and report violations
    const violations = accessibilityScanResults.violations;
    if (violations.length > 0) {
      console.error('WCAG 2.1 AA violations found:', violations);
    }

    expect(violations).toEqual([]);
  });

  test('should have proper ARIA labels on interactive elements', async ({ page }) => {
    await page.goto('http://localhost:8080');
    await page.waitForSelector('h1:has-text("OTLP Realtime Dashboard")');

    const accessibilityScanResults = await new AxeBuilder({ page })
      .withTags(['aria'])
      .analyze();

    expect(accessibilityScanResults.violations).toEqual([]);
  });

  test('should have proper color contrast', async ({ page }) => {
    await page.goto('http://localhost:8080');
    await page.waitForSelector('h1:has-text("OTLP Realtime Dashboard")');

    const accessibilityScanResults = await new AxeBuilder({ page })
      .withTags(['color-contrast'])
      .analyze();

    expect(accessibilityScanResults.violations).toEqual([]);
  });

  test('should be keyboard navigable', async ({ page }) => {
    await page.goto('http://localhost:8080');
    await page.waitForSelector('h1:has-text("OTLP Realtime Dashboard")');

    const accessibilityScanResults = await new AxeBuilder({ page })
      .withTags(['keyboard'])
      .analyze();

    expect(accessibilityScanResults.violations).toEqual([]);
  });

  test('should have proper heading hierarchy', async ({ page }) => {
    await page.goto('http://localhost:8080');
    await page.waitForSelector('h1:has-text("OTLP Realtime Dashboard")');

    const accessibilityScanResults = await new AxeBuilder({ page })
      .withTags(['heading-order'])
      .analyze();

    expect(accessibilityScanResults.violations).toEqual([]);
  });

  test('should test accessibility in traces view', async ({ page }) => {
    await page.goto('http://localhost:8080');
    await page.waitForSelector('h1:has-text("OTLP Realtime Dashboard")');
    
    // Switch to traces view
    await page.getByRole('tab', { name: /traces/i }).click();
    await page.waitForSelector('[data-testid="trace-panel"]');

    const accessibilityScanResults = await new AxeBuilder({ page })
      .withTags(['wcag2aa', 'wcag21aa'])
      .analyze();

    expect(accessibilityScanResults.violations).toEqual([]);
  });

  test('should test accessibility in metrics view', async ({ page }) => {
    await page.goto('http://localhost:8080');
    await page.waitForSelector('h1:has-text("OTLP Realtime Dashboard")');
    
    // Switch to metrics view
    await page.getByRole('tab', { name: /metrics/i }).click();
    await page.waitForSelector('[role="tabpanel"][aria-labelledby="nav-metrics"]');

    const accessibilityScanResults = await new AxeBuilder({ page })
      .withTags(['wcag2aa', 'wcag21aa'])
      .analyze();

    expect(accessibilityScanResults.violations).toEqual([]);
  });

  test('should test accessibility in settings panel', async ({ page }) => {
    await page.goto('http://localhost:8080');
    await page.waitForSelector('h1:has-text("OTLP Realtime Dashboard")');
    
    // Open settings
    await page.getByRole('button', { name: /settings/i }).click();
    await page.waitForSelector('#settings-container');

    const accessibilityScanResults = await new AxeBuilder({ page })
      .withTags(['wcag2aa', 'wcag21aa'])
      .analyze();

    expect(accessibilityScanResults.violations).toEqual([]);
  });
});

// Example with Pa11y (alternative to axe-core)
/*
import pa11y from 'pa11y';

test('should pass Pa11y accessibility tests', async ({ page }) => {
  await page.goto('http://localhost:8080');
  await page.waitForSelector('h1:has-text("OTLP Realtime Dashboard")');

  const url = page.url();
  const results = await pa11y(url, {
    standard: 'WCAG2AA',
    includeWarnings: true,
  });

  expect(results.issues.filter(issue => issue.type === 'error')).toEqual([]);
});
*/

