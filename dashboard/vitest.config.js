import { defineConfig } from 'vitest/config';

export default defineConfig({
  test: {
    include: ['tests/unit/**/*.js', 'tests/integration/**/*.js'],
    exclude: ['tests/e2e/**'],
    environment: 'jsdom',
    globals: true,
    reporters: ['default'],
    coverage: {
      reporter: ['text', 'lcov'],
      provider: 'v8',
      reportsDirectory: './coverage/unit',
    },
  },
});
