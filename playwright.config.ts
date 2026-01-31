import { defineConfig } from '@playwright/test';

export default defineConfig({
  testDir: './test-browser',
  testMatch: '**/*.test.ts',
  timeout: 30000,
  webServer: {
    command: 'python3 -m http.server 8765',
    port: 8765,
    reuseExistingServer: true,
  },
  use: {
    baseURL: 'http://localhost:8765',
    headless: true,
  },
});
