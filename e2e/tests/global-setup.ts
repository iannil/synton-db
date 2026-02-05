/**
 * Global E2E test setup
 * Starts the synton-db-server before tests
 */

import { spawn, ChildProcess } from 'child_process';
import { FullConfig } from '@playwright/test';

let serverProcess: ChildProcess;

async function startServer(): Promise<void> {
  const serverPath = process.env.SERVER_PATH || './target/release/synton-db-server';
  const configPath = process.env.CONFIG_PATH || './e2e/test-config.toml';

  console.log(`Starting server: ${serverPath} --config ${configPath}`);

  serverProcess = spawn(serverPath, ['--config', configPath], {
    stdio: ['ignore', 'pipe', 'pipe'],
  });

  serverProcess.stdout?.on('data', (data) => {
    console.log(`[Server] ${data.toString().trim()}`);
  });

  serverProcess.stderr?.on('data', (data) => {
    console.error(`[Server Error] ${data.toString().trim()}`);
  });

  // Wait for server to be ready
  await waitForServer();
}

async function waitForServer(retries = 30): Promise<void> {
  const baseUrl = process.env.BASE_URL || 'http://127.0.0.1:8080';
  const healthUrl = `${baseUrl}/health`;

  for (let i = 0; i < retries; i++) {
    try {
      const response = await fetch(healthUrl);
      if (response.ok) {
        console.log('Server is ready');
        return;
      }
    } catch {
      // Server not ready yet
    }
    await new Promise(resolve => setTimeout(resolve, 1000));
  }

  throw new Error('Server failed to start after retries');
}

export default async function globalSetup(config: FullConfig): Promise<void> {
  // Check if server is already running (useful for local development)
  const baseUrl = process.env.BASE_URL || 'http://127.0.0.1:8080';
  try {
    const response = await fetch(`${baseUrl}/health`);
    if (response.ok) {
      console.log('Using existing server at', baseUrl);
      return;
    }
  } catch {
    // No server running, start one
  }

  await startServer();
}
