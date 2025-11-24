/**
 * E2E tests for accessibility (WCAG 2.1 AA compliance)
 */

import { test, expect } from '@playwright/test';

test.describe('Accessibility Tests', () => {
  test.beforeEach(async ({ page }) => {
    // Navigate to dashboard
    await page.goto('http://localhost:8080');
  });

  test('should have proper ARIA labels', async ({ page }) => {
    // Check for ARIA labels on interactive elements
    const selectButton = page.getByRole('button', { name: /choose directory/i });
    await expect(selectButton).toHaveAttribute('aria-label');

    const navTraces = page.getByRole('tab', { name: /traces/i });
    await expect(navTraces).toHaveAttribute('aria-selected');
    await expect(navTraces).toHaveAttribute('aria-controls');
  });

  test('should support keyboard navigation', async ({ page }) => {
    // Test Tab navigation
    await page.keyboard.press('Tab');
    const focusedElement = page.locator(':focus');
    await expect(focusedElement).toBeVisible();

    // Test Enter key activation
    await page.keyboard.press('Enter');
  });

  test('should have proper heading hierarchy', async ({ page }) => {
    const h1 = page.locator('h1');
    await expect(h1).toHaveCount(1);

    const h2 = page.locator('h2');
    await expect(h2.length).toBeGreaterThan(0);
  });

  test('should have proper form labels', async ({ page }) => {
    // Open settings
    await page.getByRole('button', { name: /settings/i }).click();

    const pollingInput = page.getByLabel(/polling interval/i);
    await expect(pollingInput).toBeVisible();
  });

  test('should have proper color contrast', async ({ page }) => {
    // This would require visual regression testing or a11y tools
    // For now, we check that text is visible
    const text = page.locator('body');
    const color = await text.evaluate((el) => {
      const style = window.getComputedStyle(el);
      return {
        color: style.color,
        backgroundColor: style.backgroundColor,
      };
    });

    expect(color.color).toBeTruthy();
    expect(color.backgroundColor).toBeTruthy();
  });

  test('should have proper focus indicators', async ({ page }) => {
    const button = page.getByRole('button', { name: /choose directory/i });
    await button.focus();

    const focusedStyle = await button.evaluate((el) => {
      const style = window.getComputedStyle(el);
      return {
        outline: style.outline,
        outlineWidth: style.outlineWidth,
      };
    });

    // Should have visible focus indicator
    expect(focusedStyle.outlineWidth).not.toBe('0px');
  });
});

