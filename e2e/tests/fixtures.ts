/**
 * E2E Test Fixtures
 * Extended Playwright test fixtures with API client
 */

import { test as base } from '@playwright/test';
import { SyntonApiClient } from './helpers';

export type ApiFixtures = {
  api: SyntonApiClient;
};

export const test = base.extend<ApiFixtures>({
  api: async ({ }, use) => {
    const client = new SyntonApiClient();
    await use(client);
    // Cleanup: clear all nodes after each test
    try {
      await client.clearAll();
    } catch {
      // Ignore cleanup errors
    }
  },
});

export { expect } from '@playwright/test';
