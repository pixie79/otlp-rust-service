import { test, expect } from '@playwright/test';

test.describe('Trace Tail Viewer', () => {
  test('shows trace controls on initial load', async ({ page }) => {
    await page.goto('/');
    await expect(page.getByRole('heading', { name: 'OTLP Realtime Dashboard' })).toBeVisible();
    await expect(page.getByRole('button', { name: 'Choose Directory' })).toBeVisible();
    await expect(page.getByTestId('trace-panel')).toBeVisible();
  });
});
