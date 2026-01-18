import { defineConfig } from '@playwright/test';

export default defineConfig({
  testDir: './e2e',
  timeout: 60000,
  retries: process.env.CI ? 2 : 0,
  reporter: process.env.CI ? 'github' : 'list',
  use: {
    trace: 'on-first-retry',
    screenshot: 'only-on-failure',
    baseURL: 'http://localhost:1420',
  },
  projects: [
    {
      name: 'Desktop App',
      use: {
        browserName: 'chromium',
      },
    },
  ],
  webServer: {
    // Use Vite dev server for frontend-only testing
    // For full E2E with Tauri backend, use: pnpm tauri dev
    command: 'pnpm dev',
    port: 1420,
    timeout: 60000,
    reuseExistingServer: !process.env.CI,
  },
});
