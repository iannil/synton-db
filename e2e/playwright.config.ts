import { defineConfig, devices } from '@playwright/test';

/**
 * SYNTON-DB E2E Testing Configuration
 */
export default defineConfig({
  testDir: './tests',
  fullyParallel: false, // Tests may share server state
  forbidOnly: !!process.env.CI,
  retries: process.env.CI ? 2 : 0,
  workers: 1, // Single worker to avoid port conflicts
  reporter: [
    ['html', { outputFolder: 'html-report', open: 'never' }],
    ['json', { outputFile: 'test-results/results.json' }],
    ['list']
  ],
  use: {
    trace: 'on-first-retry',
    screenshot: 'only-on-failure',
    video: 'retain-on-failure',
    baseURL: process.env.BASE_URL || 'http://127.0.0.1:8080',
  },
  projects: [
    {
      name: 'chromium',
      use: { ...devices['Desktop Chrome'] },
    },
  ],
  // Global setup to start the server before tests
  globalSetup: './tests/global-setup.ts',
  // Global teardown to stop the server after tests
  globalTeardown: './tests/global-teardown.ts',
});
