# E2E Tests with Playwright

This directory contains end-to-end tests for the OTLP Realtime Dashboard using [Playwright](https://playwright.dev/).

## Setup

1. Install dependencies:
```bash
npm install --save-dev @playwright/test
npx playwright install
```

2. For optional AI testing tools:
```bash
# Visual regression with Percy
npm install --save-dev @percy/playwright

# Visual regression with Chromatic
npm install --save-dev chromatic

# Accessibility testing with axe-core
npm install --save-dev @axe-core/playwright

# Accessibility testing with Pa11y (alternative)
npm install --save-dev pa11y
```

## Running Tests

```bash
# Run all E2E tests
npx playwright test

# Run specific test file
npx playwright test test-accessibility.spec.js

# Run tests in headed mode
npx playwright test --headed

# Run tests with UI mode
npx playwright test --ui
```

## Test Files

- `test-accessibility.spec.js` - Basic accessibility tests (WCAG 2.1 AA)
- `test-accessibility-axe.spec.js` - Advanced accessibility tests using axe-core (optional)
- `test-cross-browser.spec.js` - Cross-browser compatibility tests
- `test-interactive-ui.spec.js` - Interactive UI feature tests
- `test-metrics-graphing.spec.js` - Metrics graphing functionality tests
- `test-visual-regression.spec.js` - Visual regression tests (optional)

## Optional AI Testing Tools

### Visual Regression (T116)

The dashboard supports visual regression testing using:

1. **Playwright Built-in**: Use `expect(page).toHaveScreenshot()` for screenshot comparison
2. **Percy**: Integrate with `@percy/playwright` for cloud-based visual testing
3. **Chromatic**: Integrate with `chromatic` for component visual testing

See `test-visual-regression.spec.js` for examples.

### Accessibility Testing (T117)

The dashboard supports accessibility testing using:

1. **axe-core**: Use `@axe-core/playwright` for automated WCAG compliance checking
2. **Pa11y**: Use `pa11y` CLI or programmatic API for accessibility audits

See `test-accessibility-axe.spec.js` for examples.

### Code Quality (T118)

For code quality analysis, use static analysis tools:

1. **ESLint**: Lint JavaScript code
2. **SonarJS**: Code quality and security analysis
3. **CodeQL**: Security vulnerability detection

These tools can be integrated into CI/CD pipelines and can analyze Playwright test coverage.

## Configuration

Create `playwright.config.js`:

```javascript
import { defineConfig, devices } from '@playwright/test';

export default defineConfig({
  testDir: './tests/e2e',
  fullyParallel: true,
  forbidOnly: !!process.env.CI,
  retries: process.env.CI ? 2 : 0,
  workers: process.env.CI ? 1 : undefined,
  reporter: 'html',
  use: {
    baseURL: 'http://localhost:8080',
    trace: 'on-first-retry',
  },
  projects: [
    {
      name: 'chromium',
      use: { ...devices['Desktop Chrome'] },
    },
    {
      name: 'firefox',
      use: { ...devices['Desktop Firefox'] },
    },
    {
      name: 'webkit',
      use: { ...devices['Desktop Safari'] },
    },
  ],
  webServer: {
    command: 'npm run serve',
    url: 'http://localhost:8080',
    reuseExistingServer: !process.env.CI,
  },
});
```

## CI/CD Integration

Example GitHub Actions workflow:

```yaml
name: E2E Tests

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions/setup-node@v3
      - run: npm ci
      - run: npx playwright install --with-deps
      - run: npx playwright test
      - uses: actions/upload-artifact@v3
        if: always()
        with:
          name: playwright-report
          path: playwright-report/
```

