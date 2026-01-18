import { test as base, Page } from '@playwright/test';

// Mock responses for Tauri commands
const mockResponses: Record<string, unknown> = {
  get_session_state: {
    is_active: false,
    session: null,
    state: 'Idle',
  },
  get_preferences: {
    focusDurationMinutes: 25,
    shortBreakMinutes: 5,
    longBreakMinutes: 15,
    autoStartBreaks: false,
    soundEnabled: true,
    theme: 'system',
  },
  get_todays_session_count: {
    sessionsToday: 0,
    dailyLimit: 3,
    isUnlimited: false,
  },
  check_permissions: {
    hosts_file_writable: true,
    hosts_file_error: null,
    process_monitoring_available: true,
    process_monitoring_error: null,
    overall_status: 'fully_functional',
  },
  get_blocklist: [],
  get_achievements: [],
  start_focus_session: {
    id: 'test-session-1',
    startTime: new Date().toISOString(),
    plannedDurationSeconds: 1500,
    state: 'Focus',
  },
  stop_session: { success: true },
};

// Inject Tauri mocks before each navigation
async function injectTauriMocks(page: Page) {
  await page.addInitScript(() => {
    // Mock __TAURI__ global
    (window as unknown as { __TAURI__: unknown }).__TAURI__ = {
      core: {
        invoke: async (cmd: string, args?: unknown) => {
          console.log(`[Tauri Mock] invoke: ${cmd}`, args);
          const mockData = (window as unknown as { __TAURI_MOCKS__: Record<string, unknown> }).__TAURI_MOCKS__;
          if (mockData && cmd in mockData) {
            return mockData[cmd];
          }
          console.warn(`[Tauri Mock] No mock for command: ${cmd}`);
          return null;
        },
      },
      event: {
        listen: async (event: string, handler: unknown) => {
          console.log(`[Tauri Mock] listen: ${event}`);
          return () => {};
        },
        emit: async (event: string, payload: unknown) => {
          console.log(`[Tauri Mock] emit: ${event}`, payload);
        },
      },
    };

    // Also mock @tauri-apps/api modules
    (window as unknown as { __TAURI_INTERNALS__: { invoke: unknown } }).__TAURI_INTERNALS__ = {
      invoke: async (cmd: string, args?: unknown) => {
        const mockData = (window as unknown as { __TAURI_MOCKS__: Record<string, unknown> }).__TAURI_MOCKS__;
        if (mockData && cmd in mockData) {
          return mockData[cmd];
        }
        return null;
      },
    };
  });
}

// Extend base test with Tauri mocking
export const test = base.extend<{ tauriMocks: Record<string, unknown> }>({
  tauriMocks: [mockResponses, { option: true }],

  page: async ({ page, tauriMocks }, use) => {
    // Inject mock data before navigation
    await page.addInitScript((mocks) => {
      (window as unknown as { __TAURI_MOCKS__: Record<string, unknown> }).__TAURI_MOCKS__ = mocks;
    }, tauriMocks);

    await injectTauriMocks(page);

    await use(page);
  },
});

export { expect } from '@playwright/test';
