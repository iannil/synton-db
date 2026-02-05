/**
 * Global E2E test teardown
 * Stops the synton-db-server after tests
 */

import { FullConfig } from '@playwright/test';

export default async function globalTeardown(config: FullConfig): Promise<void> {
  // The server process will be killed automatically when the Node process exits
  // If we started it, we could clean it up here, but for simplicity we rely on
  // the OS to clean up child processes

  console.log('E2E tests completed');
}
